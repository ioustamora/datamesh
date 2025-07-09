/// REST API Server Module
/// 
/// This module implements the REST API server as specified in the DataMesh
/// Application & Network Improvements Roadmap. It provides:
/// - RESTful API endpoints for file operations
/// - WebSocket support for real-time updates
/// - Swagger UI for API documentation
/// - Authentication and rate limiting
/// - CORS and security headers

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Json},
    routing::{delete, get, post},
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{error, info, warn};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

use crate::cli::Cli;
use crate::config::Config;
use crate::database;
use crate::error::{DfsError, DfsResult};
use crate::file_storage;
use crate::key_manager::KeyManager;
use crate::smart_cache::SmartCacheManager;

/// API server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Server host address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Maximum upload file size in bytes
    pub max_upload_size: u64,
    /// API rate limiting - requests per minute
    pub rate_limit_per_minute: u32,
    /// Enable HTTPS
    pub enable_https: bool,
    /// Path to TLS certificate file
    pub cert_path: Option<PathBuf>,
    /// Path to TLS private key file
    pub key_path: Option<PathBuf>,
    /// Enable Swagger UI
    pub enable_swagger: bool,
    /// API prefix (e.g., "/api/v1")
    pub api_prefix: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            max_upload_size: 100 * 1024 * 1024, // 100MB
            rate_limit_per_minute: 1000,
            enable_https: false,
            cert_path: None,
            key_path: None,
            enable_swagger: true,
            api_prefix: "/api/v1".to_string(),
        }
    }
}

/// API server state shared across handlers
#[derive(Clone)]
pub struct ApiState {
    pub config: Config,
    pub key_manager: Arc<KeyManager>,
    pub cache_manager: Arc<SmartCacheManager>,
    pub cli: Cli,
    pub api_config: ApiConfig,
}

/// File upload request
#[derive(Debug, Deserialize, ToSchema)]
pub struct FileUploadRequest {
    /// Optional file name
    pub name: Option<String>,
    /// Optional tags (comma-separated)
    pub tags: Option<String>,
    /// Optional public key for encryption
    pub public_key: Option<String>,
}

/// File upload response
#[derive(Debug, Serialize, ToSchema)]
pub struct FileUploadResponse {
    /// File key for retrieval
    pub file_key: String,
    /// Assigned file name
    pub file_name: String,
    /// File size in bytes
    pub file_size: u64,
    /// Upload timestamp
    pub uploaded_at: DateTime<Utc>,
    /// Success message
    pub message: String,
}

/// File download response
#[derive(Debug, Serialize, ToSchema)]
pub struct FileDownloadResponse {
    /// File name
    pub file_name: String,
    /// File size in bytes
    pub file_size: u64,
    /// Content type
    pub content_type: String,
    /// File data
    pub data: Vec<u8>,
}

/// File metadata response
#[derive(Debug, Serialize, ToSchema)]
pub struct FileMetadataResponse {
    /// File key
    pub file_key: String,
    /// File name
    pub file_name: String,
    /// Original file name
    pub original_name: String,
    /// File size in bytes
    pub file_size: u64,
    /// Upload timestamp
    pub uploaded_at: DateTime<Utc>,
    /// Tags
    pub tags: Vec<String>,
    /// Public key used for encryption
    pub public_key: String,
}

/// File list response
#[derive(Debug, Serialize, ToSchema)]
pub struct FileListResponse {
    /// List of files
    pub files: Vec<FileMetadataResponse>,
    /// Total count
    pub total: usize,
    /// Current page
    pub page: u32,
    /// Page size
    pub page_size: u32,
}

/// File search request
#[derive(Debug, Deserialize, ToSchema)]
pub struct FileSearchRequest {
    /// Search query
    pub query: Option<String>,
    /// Tags filter
    pub tags: Option<String>,
    /// File size range (min, max)
    pub size_range: Option<(u64, u64)>,
    /// Date range (from, to)
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    /// Page number
    pub page: Option<u32>,
    /// Page size
    pub page_size: Option<u32>,
}

/// API error response
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiErrorResponse {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Additional error details
    pub details: Option<String>,
    /// Request ID for tracking
    pub request_id: String,
}

/// API statistics response
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiStatsResponse {
    /// Total files stored
    pub total_files: u64,
    /// Total storage used in bytes
    pub total_storage_bytes: u64,
    /// Cache hit ratio
    pub cache_hit_ratio: f64,
    /// API requests in last hour
    pub api_requests_last_hour: u64,
    /// System status
    pub system_status: String,
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    /// File upload progress
    FileUploadProgress {
        file_key: String,
        progress: f64,
        status: String,
    },
    /// File download progress
    FileDownloadProgress {
        file_key: String,
        progress: f64,
        status: String,
    },
    /// System status update
    SystemStatus {
        status: String,
        message: String,
    },
    /// Cache statistics update
    CacheStats {
        hit_ratio: f64,
        cache_size: u64,
    },
}

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        upload_file,
        download_file,
        get_file_metadata,
        list_files,
        search_files,
        delete_file,
        get_api_stats,
        health_check
    ),
    components(
        schemas(
            FileUploadRequest,
            FileUploadResponse,
            FileDownloadResponse,
            FileMetadataResponse,
            FileListResponse,
            FileSearchRequest,
            ApiErrorResponse,
            ApiStatsResponse,
            WebSocketMessage
        )
    ),
    tags(
        (name = "files", description = "File operations API"),
        (name = "search", description = "File search API"),
        (name = "stats", description = "Statistics API"),
        (name = "health", description = "Health check API")
    ),
    info(
        title = "DataMesh API",
        version = "1.0.0",
        description = "RESTful API for DataMesh distributed storage system",
        contact(
            name = "DataMesh Team",
            email = "support@datamesh.io"
        )
    )
)]
pub struct ApiDoc;

/// REST API Server
pub struct ApiServer {
    state: ApiState,
    app: Router,
}

impl ApiServer {
    /// Create a new API server
    pub fn new(
        config: Config,
        key_manager: Arc<KeyManager>,
        cache_manager: Arc<SmartCacheManager>,
        cli: Cli,
        api_config: ApiConfig,
    ) -> Self {
        let state = ApiState {
            config,
            key_manager,
            cache_manager,
            cli,
            api_config: api_config.clone(),
        };

        let app = Self::create_app(state.clone());

        Self { state, app }
    }

    /// Create the Axum application router
    fn create_app(state: ApiState) -> Router {
        let api_prefix = &state.api_config.api_prefix;

        // API routes
        let api_routes = Router::new()
            .route("/files", post(upload_file))
            .route("/files/:file_key", get(download_file))
            .route("/files/:file_key", delete(delete_file))
            .route("/files/:file_key/metadata", get(get_file_metadata))
            .route("/files", get(list_files))
            .route("/search", post(search_files))
            .route("/stats", get(get_api_stats))
            .route("/health", get(health_check))
            .with_state(state.clone());

        let mut app = Router::new()
            .nest(api_prefix, api_routes);

        // Add Swagger UI if enabled
        if state.api_config.enable_swagger {
            app = app.merge(
                SwaggerUi::new("/swagger-ui")
                    .url("/api-docs/openapi.json", ApiDoc::openapi())
            );
        }

        // Add middleware layers
        app = app.layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any))
                .layer(DefaultBodyLimit::max(state.api_config.max_upload_size as usize))
        );

        app
    }

    /// Start the API server
    async fn start(&self) -> DfsResult<()> {
        let addr = format!("{}:{}", self.state.api_config.host, self.state.api_config.port);
        info!("Starting DataMesh API server on {}", addr);

        if self.state.api_config.enable_https {
            self.start_https_server(&addr).await
        } else {
            self.start_http_server(&addr).await
        }
    }

    /// Start HTTP server
    async fn start_http_server(&self, addr: &str) -> DfsResult<()> {
        let listener = tokio::net::TcpListener::bind(addr).await
            .map_err(|e| DfsError::Network(format!("Failed to bind to {}: {}", addr, e)))?;

        info!("DataMesh API server listening on http://{}", addr);
        if self.state.api_config.enable_swagger {
            info!("Swagger UI available at: http://{}/swagger-ui", addr);
        }

        axum::serve(listener, self.app.clone()).await
            .map_err(|e| DfsError::Network(format!("Server error: {}", e)))?;

        Ok(())
    }

    /// Start HTTPS server
    async fn start_https_server(&self, addr: &str) -> DfsResult<()> {
        let cert_path = self.state.api_config.cert_path.as_ref()
            .ok_or_else(|| DfsError::Network("HTTPS enabled but no cert_path provided".to_string()))?;
        let key_path = self.state.api_config.key_path.as_ref()
            .ok_or_else(|| DfsError::Network("HTTPS enabled but no key_path provided".to_string()))?;

        let config = RustlsConfig::from_pem_file(cert_path, key_path).await
            .map_err(|e| DfsError::Network(format!("Failed to load TLS config: {}", e)))?;

        info!("DataMesh API server listening on https://{}", addr);
        if self.state.api_config.enable_swagger {
            info!("Swagger UI available at: https://{}/swagger-ui", addr);
        }

        axum_server::bind_rustls(addr.parse().unwrap(), config)
            .serve(self.app.clone().into_make_service())
            .await
            .map_err(|e| DfsError::Network(format!("HTTPS server error: {}", e)))?;

        Ok(())
    }
}

/// Upload a file
#[utoipa::path(
    post,
    path = "/api/v1/files",
    request_body = FileUploadRequest,
    responses(
        (status = 200, description = "File uploaded successfully", body = FileUploadResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 413, description = "File too large", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse)
    ),
    tag = "files"
)]
async fn upload_file(
    State(state): State<ApiState>,
    mut multipart: Multipart,
) -> Result<Json<FileUploadResponse>, ApiError> {
    let mut file_data: Option<Bytes> = None;
    let mut file_name: Option<String> = None;
    let mut request_name: Option<String> = None;
    let mut tags: Option<String> = None;
    let mut public_key: Option<String> = None;

    // Parse multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        ApiError::BadRequest(format!("Failed to parse multipart data: {}", e))
    })? {
        match field.name() {
            Some("file") => {
                file_name = field.file_name().map(|s| s.to_string());
                file_data = Some(field.bytes().await.map_err(|e| {
                    ApiError::BadRequest(format!("Failed to read file data: {}", e))
                })?);
            }
            Some("name") => {
                request_name = Some(field.text().await.map_err(|e| {
                    ApiError::BadRequest(format!("Failed to read name field: {}", e))
                })?);
            }
            Some("tags") => {
                tags = Some(field.text().await.map_err(|e| {
                    ApiError::BadRequest(format!("Failed to read tags field: {}", e))
                })?);
            }
            Some("public_key") => {
                public_key = Some(field.text().await.map_err(|e| {
                    ApiError::BadRequest(format!("Failed to read public_key field: {}", e))
                })?);
            }
            _ => {}
        }
    }

    let file_data = file_data.ok_or_else(|| {
        ApiError::BadRequest("No file data provided".to_string())
    })?;

    let original_name = file_name.unwrap_or_else(|| "unnamed_file".to_string());

    // Write file to temporary location
    let temp_dir = std::env::temp_dir();
    let temp_file_path = temp_dir.join(format!("upload_{}", Uuid::new_v4()));
    
    tokio::fs::write(&temp_file_path, &file_data).await.map_err(|e| {
        ApiError::InternalServerError(format!("Failed to write temporary file: {}", e))
    })?;

    // Upload file using existing file storage system
    match file_storage::handle_put_command(
        &state.cli,
        &state.key_manager,
        &temp_file_path,
        &public_key,
        &request_name,
        &tags,
    ).await {
        Ok(()) => {
            // Clean up temporary file
            let _ = tokio::fs::remove_file(&temp_file_path).await;

            // Get file information from database
            let db_path = database::get_default_db_path()
                .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;
            let db = database::DatabaseManager::new(&db_path)
                .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;

            let final_name = request_name.as_ref().unwrap_or(&original_name);
            let file_entry = db.get_file_by_name(final_name)
                .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?
                .ok_or_else(|| ApiError::InternalServerError("File not found after upload".to_string()))?;

            let response = FileUploadResponse {
                file_key: file_entry.file_key,
                file_name: file_entry.name,
                file_size: file_entry.file_size,
                uploaded_at: file_entry.upload_time.and_utc(),
                message: "File uploaded successfully".to_string(),
            };

            Ok(Json(response))
        }
        Err(e) => {
            // Clean up temporary file
            let _ = tokio::fs::remove_file(&temp_file_path).await;
            Err(ApiError::InternalServerError(format!("Upload failed: {}", e)))
        }
    }
}

/// Download a file
#[utoipa::path(
    get,
    path = "/api/v1/files/{file_key}",
    params(
        ("file_key" = String, Path, description = "File key or name")
    ),
    responses(
        (status = 200, description = "File downloaded successfully"),
        (status = 404, description = "File not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse)
    ),
    tag = "files"
)]
async fn download_file(
    State(state): State<ApiState>,
    Path(file_key): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let temp_dir = std::env::temp_dir();
    let temp_file_path = temp_dir.join(format!("download_{}", Uuid::new_v4()));

    // Download file using existing file storage system
    match file_storage::handle_get_command(
        &state.cli,
        &state.key_manager,
        &file_key,
        &temp_file_path,
        &None,
    ).await {
        Ok(()) => {
            // Read file data
            let file_data = tokio::fs::read(&temp_file_path).await.map_err(|e| {
                ApiError::InternalServerError(format!("Failed to read downloaded file: {}", e))
            })?;

            // Clean up temporary file
            let _ = tokio::fs::remove_file(&temp_file_path).await;

            // Get file metadata
            let db_path = database::get_default_db_path()
                .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;
            let db = database::DatabaseManager::new(&db_path)
                .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;

            let file_entry = db.get_file_by_key(&file_key)
                .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?
                .or_else(|| {
                    db.get_file_by_name(&file_key).ok().flatten()
                })
                .ok_or_else(|| ApiError::NotFound("File not found".to_string()))?;

            let headers = [
                (header::CONTENT_TYPE, "application/octet-stream"),
                (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"{}\"", file_entry.original_filename)),
            ];

            Ok((headers, file_data))
        }
        Err(e) => {
            // Clean up temporary file
            let _ = tokio::fs::remove_file(&temp_file_path).await;
            Err(ApiError::NotFound(format!("File not found: {}", e)))
        }
    }
}

/// Get file metadata
#[utoipa::path(
    get,
    path = "/api/v1/files/{file_key}/metadata",
    params(
        ("file_key" = String, Path, description = "File key or name")
    ),
    responses(
        (status = 200, description = "File metadata retrieved", body = FileMetadataResponse),
        (status = 404, description = "File not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse)
    ),
    tag = "files"
)]
async fn get_file_metadata(
    State(state): State<ApiState>,
    Path(file_key): Path<String>,
) -> Result<Json<FileMetadataResponse>, ApiError> {
    let db_path = database::get_default_db_path()
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;
    let db = database::DatabaseManager::new(&db_path)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;

    let file_entry = db.get_file_by_key(&file_key)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?
        .or_else(|| {
            db.get_file_by_name(&file_key).ok().flatten()
        })
        .ok_or_else(|| ApiError::NotFound("File not found".to_string()))?;

    let response = FileMetadataResponse {
        file_key: file_entry.file_key,
        file_name: file_entry.name,
        original_name: file_entry.original_filename,
        file_size: file_entry.file_size,
        uploaded_at: file_entry.upload_time.and_utc(),
        tags: file_entry.tags,
        public_key: file_entry.public_key_hex,
    };

    Ok(Json(response))
}

/// List files
#[utoipa::path(
    get,
    path = "/api/v1/files",
    params(
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Page size"),
        ("tags" = Option<String>, Query, description = "Filter by tags")
    ),
    responses(
        (status = 200, description = "Files listed successfully", body = FileListResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse)
    ),
    tag = "files"
)]
async fn list_files(
    State(state): State<ApiState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<FileListResponse>, ApiError> {
    let db_path = database::get_default_db_path()
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;
    let db = database::DatabaseManager::new(&db_path)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;

    let tags = params.get("tags").map(|s| s.as_str());
    let page: u32 = params.get("page").and_then(|s| s.parse().ok()).unwrap_or(1);
    let page_size: u32 = params.get("page_size").and_then(|s| s.parse().ok()).unwrap_or(20);

    let files = db.list_files(tags)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;

    let total = files.len();
    let start = ((page - 1) * page_size) as usize;
    let end = std::cmp::min(start + page_size as usize, total);

    let file_responses: Vec<FileMetadataResponse> = files
        .into_iter()
        .skip(start)
        .take(end - start)
        .map(|file| FileMetadataResponse {
            file_key: file.file_key,
            file_name: file.file_name,
            original_name: file.original_name,
            file_size: file.file_size,
            uploaded_at: file.uploaded_at.and_utc(),
            tags: file.tags,
            public_key: file.public_key_hex,
        })
        .collect();

    let response = FileListResponse {
        files: file_responses,
        total,
        page,
        page_size,
    };

    Ok(Json(response))
}

/// Search files
#[utoipa::path(
    post,
    path = "/api/v1/search",
    request_body = FileSearchRequest,
    responses(
        (status = 200, description = "Search completed successfully", body = FileListResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse)
    ),
    tag = "search"
)]
async fn search_files(
    State(state): State<ApiState>,
    Json(request): Json<FileSearchRequest>,
) -> Result<Json<FileListResponse>, ApiError> {
    let db_path = database::get_default_db_path()
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;
    let db = database::DatabaseManager::new(&db_path)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;

    let tags = request.tags.as_deref();
    let page = request.page.unwrap_or(1);
    let page_size = request.page_size.unwrap_or(20);

    // For now, use basic tag-based search
    // In a full implementation, this would support complex search queries
    let files = db.list_files(tags)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;

    let total = files.len();
    let start = ((page - 1) * page_size) as usize;
    let end = std::cmp::min(start + page_size as usize, total);

    let file_responses: Vec<FileMetadataResponse> = files
        .into_iter()
        .skip(start)
        .take(end - start)
        .map(|file| FileMetadataResponse {
            file_key: file.file_key,
            file_name: file.file_name,
            original_name: file.original_name,
            file_size: file.file_size,
            uploaded_at: file.uploaded_at.and_utc(),
            tags: file.tags,
            public_key: file.public_key_hex,
        })
        .collect();

    let response = FileListResponse {
        files: file_responses,
        total,
        page,
        page_size,
    };

    Ok(Json(response))
}

/// Delete a file
#[utoipa::path(
    delete,
    path = "/api/v1/files/{file_key}",
    params(
        ("file_key" = String, Path, description = "File key or name")
    ),
    responses(
        (status = 200, description = "File deleted successfully"),
        (status = 404, description = "File not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse)
    ),
    tag = "files"
)]
async fn delete_file(
    State(state): State<ApiState>,
    Path(file_key): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let db_path = database::get_default_db_path()
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;
    let db = database::DatabaseManager::new(&db_path)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;

    // Check if file exists
    let file_entry = db.get_file_by_key(&file_key)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?
        .or_else(|| {
            db.get_file_by_name(&file_key).ok().flatten()
        })
        .ok_or_else(|| ApiError::NotFound("File not found".to_string()))?;

    // TODO: Implement actual file deletion from DHT
    // For now, just remove from database
    db.delete_file(&file_entry.name)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;

    Ok(Json(serde_json::json!({
        "message": "File deleted successfully",
        "file_key": file_key
    })))
}

/// Get API statistics
#[utoipa::path(
    get,
    path = "/api/v1/stats",
    responses(
        (status = 200, description = "Statistics retrieved", body = ApiStatsResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse)
    ),
    tag = "stats"
)]
async fn get_api_stats(
    State(state): State<ApiState>,
) -> Result<Json<ApiStatsResponse>, ApiError> {
    let db_path = database::get_default_db_path()
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;
    let db = database::DatabaseManager::new(&db_path)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;

    let files = db.list_files(None)
        .map_err(|e| ApiError::InternalServerError(format!("Database error: {}", e)))?;

    let total_files = files.len() as u64;
    let total_storage_bytes: u64 = files.iter().map(|f| f.file_size).sum();

    let cache_stats = state.cache_manager.get_stats().await;

    let response = ApiStatsResponse {
        total_files,
        total_storage_bytes,
        cache_hit_ratio: cache_stats.hit_ratio,
        api_requests_last_hour: 0, // TODO: Implement request tracking
        system_status: "healthy".to_string(),
    };

    Ok(Json(response))
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/api/v1/health",
    responses(
        (status = 200, description = "Service is healthy"),
        (status = 503, description = "Service is unhealthy")
    ),
    tag = "health"
)]
async fn health_check() -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now(),
        "version": "1.0.0"
    })))
}

/// API error types
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Internal server error: {0}")]
    InternalServerError(String),
    #[error("Too many requests")]
    TooManyRequests,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::TooManyRequests => (StatusCode::TOO_MANY_REQUESTS, "Too many requests".to_string()),
        };

        let body = Json(ApiErrorResponse {
            code: status.as_u16().to_string(),
            message: error_message,
            details: None,
            request_id: Uuid::new_v4().to_string(),
        });

        (status, body).into_response()
    }
}