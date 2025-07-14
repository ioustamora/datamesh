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

use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Json},
    routing::{delete, get, post, put},
    Router,
};
// use axum_server::tls_rustls::RustlsConfig;
use axum::http::Request;
use axum::middleware::{self, Next};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

// Additional imports for file operations (removed duplicate HashMap import)

use crate::bootstrap_admin::BootstrapAdministrationService;
use crate::cli::Cli;
use crate::config::Config;
use crate::database;
use crate::error::{DfsError, DfsResult};
use crate::file_storage;
use crate::actor_file_storage::ActorFileStorage;
use crate::governance::{AuthService, UserRegistry};
use crate::governance_service::GovernanceService;
use crate::key_manager::KeyManager;
use crate::smart_cache::SmartCacheManager;
use crate::storage_economy::{StorageEconomyService, UserStorageProfile, UserStorageStatistics, StorageTier, EconomyTransaction};
use crate::websocket::{websocket_handler, WebSocketManager};

/// API error types for HTTP responses
#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    InternalServerError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            ApiError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        
        let body = Json(serde_json::json!({
            "error": message,
            "status": status.as_u16()
        }));
        
        (status, body).into_response()
    }
}

/// JWT authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// JWT signing secret (should be loaded from environment)
    pub secret: String,
    /// JWT issuer
    pub issuer: String,
    /// JWT audience
    pub audience: String,
    /// Token expiry in hours
    pub expiry_hours: u64,
    /// Clock skew allowance in seconds
    pub leeway_seconds: u64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: std::env::var("DATAMESH_JWT_SECRET")
                .unwrap_or_else(|_| {
                    eprintln!("WARNING: DATAMESH_JWT_SECRET not set. Using insecure default for development only!");
                    "INSECURE_DEFAULT_SECRET_CHANGE_IN_PRODUCTION".to_string()
                }),
            issuer: "datamesh.local".to_string(),
            audience: "datamesh-api".to_string(),
            expiry_hours: 24,
            leeway_seconds: 30,
        }
    }
}

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
    /// Minimum TLS version (1.2 or 1.3)
    pub min_tls_version: String,
    /// Enable Swagger UI
    pub enable_swagger: bool,
    /// API prefix (e.g., "/api/v1")
    pub api_prefix: String,
    /// JWT configuration
    pub jwt: JwtConfig,
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
            min_tls_version: "1.3".to_string(),
            enable_swagger: true,
            api_prefix: "/api/v1".to_string(),
            jwt: JwtConfig::default(),
        }
    }
}

/// API server state shared across handlers
#[derive(Clone)]
pub struct ApiState {
    pub config: Config,
    pub key_manager: Arc<KeyManager>,
    pub cache_manager: Arc<SmartCacheManager>,
    pub governance_service: Arc<GovernanceService>,
    pub bootstrap_admin: Arc<BootstrapAdministrationService>,
    pub auth_service: Arc<AuthService>,
    pub user_registry: Arc<UserRegistry>,
    pub cli: Cli,
    pub api_config: ApiConfig,
    pub websocket_manager: Arc<WebSocketManager>,
    pub file_storage: Arc<ActorFileStorage>,
    pub storage_economy: Arc<StorageEconomyService>,
}

/// Extract user ID from Authorization header
async fn extract_user_id(
    headers: &HeaderMap,
    state: &ApiState,
) -> Result<crate::governance::UserId, ApiError> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .ok_or_else(|| ApiError::Unauthorized("Missing authorization header".to_string()))?
        .to_str()
        .map_err(|_| ApiError::Unauthorized("Invalid authorization header format".to_string()))?;

    // Extract Bearer token
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::Unauthorized("Invalid authorization scheme".to_string()))?;

    state
        .auth_service
        .get_user_id_from_token(token)
        .map_err(|e| ApiError::Unauthorized(format!("Invalid token: {}", e)))
}

/// Verify user authentication and get user account
async fn authenticate_user(
    headers: &HeaderMap,
    state: &ApiState,
) -> Result<crate::governance::UserAccount, ApiError> {
    let user_id = extract_user_id(headers, state).await?;

    state
        .user_registry
        .get_user(&user_id)
        .map_err(|e| ApiError::InternalServerError(format!("Failed to get user: {}", e)))?
        .ok_or_else(|| ApiError::Unauthorized("User not found".to_string()))
}

/// Login request
#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    /// User email
    pub email: String,
    /// User password
    pub password: String,
}

/// Registration request
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    /// User email
    pub email: String,
    /// User password
    pub password: String,
    /// User public key
    pub public_key: String,
}

/// Authentication response
#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    /// JWT access token
    pub access_token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Token expiration in seconds
    pub expires_in: u64,
    /// User information
    pub user: UserInfo,
}

/// User information
#[derive(Debug, Serialize, ToSchema)]
pub struct UserInfo {
    /// User ID
    pub user_id: String,
    /// User email
    pub email: String,
    /// Account type
    pub account_type: String,
    /// Registration date
    pub registration_date: DateTime<Utc>,
    /// Verification status
    pub verification_status: String,
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

/// Security headers middleware
async fn add_security_headers(request: Request<axum::body::Body>, next: Next) -> impl IntoResponse {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
    );
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; connect-src 'self'"),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(header::SERVER, HeaderValue::from_static("DataMesh"));

    response
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
    SystemStatus { status: String, message: String },
    /// Cache statistics update
    CacheStats { hit_ratio: f64, cache_size: u64 },
}

/// Governance status response
#[derive(Debug, Serialize, ToSchema)]
pub struct GovernanceStatusResponse {
    /// Whether governance is enabled
    pub enabled: bool,
    /// Total number of operators
    pub total_operators: usize,
    /// Active operators
    pub active_operators: usize,
    /// Network health status
    pub network_healthy: bool,
    /// Can reach consensus
    pub can_reach_consensus: bool,
}

/// Operator registration request
#[derive(Debug, Deserialize, ToSchema)]
pub struct ApiOperatorRegistrationRequest {
    /// Legal name of the operator
    pub legal_name: String,
    /// Contact email
    pub contact_email: String,
    /// Jurisdiction
    pub jurisdiction: String,
    /// Stake amount
    pub stake_amount: u64,
    /// Proposed services
    pub proposed_services: Vec<String>,
    /// Technical contact
    pub technical_contact: String,
    /// Service level agreement
    pub service_level_agreement: String,
    /// Peer ID
    pub peer_id: String,
}

/// Operator response
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiOperatorResponse {
    /// Operator ID
    pub operator_id: String,
    /// Peer ID
    pub peer_id: String,
    /// Stake amount
    pub stake: u64,
    /// Jurisdiction
    pub jurisdiction: String,
    /// Governance weight
    pub governance_weight: f64,
    /// Reputation score
    pub reputation_score: f64,
    /// Services provided
    pub services: Vec<String>,
    /// Registration date
    pub registration_date: DateTime<Utc>,
    /// Last active
    pub last_active: DateTime<Utc>,
}

/// Service registration request
#[derive(Debug, Deserialize, ToSchema)]
pub struct ApiServiceRegistrationRequest {
    /// Service type
    pub service_type: String,
    /// Service configuration
    pub service_config: serde_json::Value,
}

/// Admin action request
#[derive(Debug, Deserialize, ToSchema)]
pub struct ApiAdminActionRequest {
    /// Action type
    pub action_type: String,
    /// Target
    pub target: String,
    /// Reason
    pub reason: String,
}

/// Admin action response
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiAdminActionResponse {
    /// Action ID
    pub action_id: String,
    /// Operator ID
    pub operator_id: String,
    /// Action type
    pub action_type: String,
    /// Target
    pub target: String,
    /// Reason
    pub reason: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Network health response
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiNetworkHealthResponse {
    /// Total operators
    pub total_operators: usize,
    /// Online operators
    pub online_operators: usize,
    /// Online percentage
    pub online_percentage: f64,
    /// Total governance weight
    pub total_governance_weight: f64,
    /// Online governance weight
    pub online_governance_weight: f64,
    /// Can reach consensus
    pub can_reach_consensus: bool,
}

/// Economy status response
#[derive(Debug, Serialize, ToSchema)]
pub struct EconomyStatusResponse {
    /// Overall economy health
    pub health: String,
    /// Total contributors
    pub total_contributors: u64,
    /// Total storage contributed
    pub total_storage_contributed: u64,
    /// Active verifications
    pub active_verifications: u64,
    /// Network utilization
    pub network_utilization: f64,
}

/// User economy profile response
#[derive(Debug, Serialize, ToSchema)]
pub struct UserEconomyProfileResponse {
    /// User ID
    pub user_id: String,
    /// Current storage tier
    pub tier: String,
    /// Current storage usage
    pub current_usage: u64,
    /// Maximum storage allowed
    pub max_storage: u64,
    /// Upload quota used
    pub upload_quota_used: u64,
    /// Upload quota limit
    pub upload_quota_limit: u64,
    /// Download quota used
    pub download_quota_used: u64,
    /// Download quota limit
    pub download_quota_limit: u64,
    /// Reputation score
    pub reputation_score: f64,
    /// Violations count
    pub violations_count: usize,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Whether user can contribute
    pub can_contribute: bool,
}

/// Storage contribution request
#[derive(Debug, Deserialize, ToSchema)]
pub struct StorageContributionRequest {
    /// Storage path
    pub storage_path: String,
    /// Amount to contribute in bytes
    pub amount: u64,
}

/// Storage tier upgrade request
#[derive(Debug, Deserialize, ToSchema)]
pub struct StorageTierUpgradeRequest {
    /// Target tier
    pub target_tier: String,
    /// Payment method
    pub payment_method: String,
    /// Additional storage amount
    pub additional_storage: Option<u64>,
}

/// Challenge response request
#[derive(Debug, Deserialize, ToSchema)]
pub struct ChallengeResponseRequest {
    /// Challenge ID
    pub challenge_id: String,
    /// Response data
    pub response_data: String,
}

/// Storage tiers response
#[derive(Debug, Serialize, ToSchema)]
pub struct StorageTiersResponse {
    /// Available tiers
    pub tiers: Vec<StorageTierInfo>,
}

/// Storage tier information
#[derive(Debug, Serialize, ToSchema)]
pub struct StorageTierInfo {
    /// Tier name
    pub name: String,
    /// Maximum storage
    pub max_storage: u64,
    /// Upload quota
    pub upload_quota: u64,
    /// Download quota
    pub download_quota: u64,
    /// Monthly cost
    pub monthly_cost: Option<f64>,
    /// Description
    pub description: String,
}

/// Economy transaction response
#[derive(Debug, Serialize, ToSchema)]
pub struct EconomyTransactionResponse {
    /// Transaction ID
    pub transaction_id: String,
    /// Transaction type
    pub transaction_type: String,
    /// Amount
    pub amount: u64,
    /// Description
    pub description: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Status
    pub status: String,
}

/// Quota status response
#[derive(Debug, Serialize, ToSchema)]
pub struct QuotaStatusResponse {
    /// Storage quota used
    pub storage_used: u64,
    /// Storage quota limit
    pub storage_limit: u64,
    /// Upload quota used
    pub upload_quota_used: u64,
    /// Upload quota limit
    pub upload_quota_limit: u64,
    /// Download quota used
    pub download_quota_used: u64,
    /// Download quota limit
    pub download_quota_limit: u64,
    /// Next reset time
    pub next_reset: DateTime<Utc>,
}

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        upload_file,
        download_file,
        get_file_metadata,
        list_files,
        delete_file,
        login,
        register,
        get_current_user,
        update_profile,
        change_password,
        refresh_token,
        logout,
        get_system_metrics,
        get_storage_metrics,
        get_network_metrics,
        get_proposals,
        submit_proposal,
        vote_on_proposal,
        get_user_settings,
        update_user_settings,
        get_users,
        get_system_health,
        get_economy_status,
        get_user_economy_profile,
        update_economy_profile,
        start_storage_contribution,
        get_contribution_status,
        stop_storage_contribution,
        get_storage_tiers,
        upgrade_storage_tier,
        get_verification_status,
        respond_to_challenge,
        get_economy_transactions,
        get_quota_status
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
            WebSocketMessage,
            GovernanceStatusResponse,
            ApiOperatorRegistrationRequest,
            ApiOperatorResponse,
            ApiServiceRegistrationRequest,
            ApiAdminActionRequest,
            ApiAdminActionResponse,
            ApiNetworkHealthResponse,
            UserInfo,
            UpdateProfileRequest,
            ChangePasswordRequest,
            SystemMetricsResponse,
            StorageMetricsResponse,
            NetworkMetricsResponse,
            ProposalResponse,
            SubmitProposalRequest,
            VoteRequest,
            UserSettingsResponse,
            UpdateUserSettingsRequest,
            UsersListResponse,
            SystemHealthResponse,
            EconomyStatusResponse,
            UserEconomyProfileResponse,
            StorageContributionRequest,
            StorageTierUpgradeRequest,
            ChallengeResponseRequest,
            StorageTiersResponse,
            StorageTierInfo,
            EconomyTransactionResponse,
            QuotaStatusResponse
        )
    ),
    tags(
        (name = "files", description = "File operations API"),
        (name = "search", description = "File search API"),
        (name = "stats", description = "Statistics API"),
        (name = "health", description = "Health check API"),
        (name = "governance", description = "Governance API"),
        (name = "admin", description = "Administration API"),
        (name = "auth", description = "Authentication API"),
        (name = "analytics", description = "Analytics API"),
        (name = "settings", description = "User settings API"),
        (name = "economy", description = "Storage Economy API")
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
    pub async fn new(
        config: Config,
        key_manager: Arc<KeyManager>,
        cache_manager: Arc<SmartCacheManager>,
        governance_service: Arc<GovernanceService>,
        bootstrap_admin: Arc<BootstrapAdministrationService>,
        cli: Cli,
        api_config: ApiConfig,
    ) -> Result<Self, DfsError> {
        // Initialize authentication components with secure configuration
        let auth_service = Arc::new(AuthService::new(&api_config.jwt)?);
        let user_registry = Arc::new(UserRegistry::new());
        
        // Initialize WebSocket manager
        let websocket_manager = Arc::new(WebSocketManager::new());
        
        // Initialize distributed file storage
        let file_storage = Arc::new(
            ActorFileStorage::new(&cli, &config)
                .await
                .map_err(|e| DfsError::Storage(format!("Failed to initialize file storage: {}", e)))?
        );

        // Initialize storage economy service
        let storage_economy = Arc::new(
            StorageEconomyService::new(&config)
                .await
                .map_err(|e| DfsError::Storage(format!("Failed to initialize storage economy: {}", e)))?
        );

        let state = ApiState {
            config,
            key_manager,
            cache_manager,
            governance_service,
            bootstrap_admin,
            auth_service,
            user_registry,
            cli,
            api_config: api_config.clone(),
            websocket_manager,
            file_storage,
            storage_economy,
        };

        let app = Self::create_app(state.clone());

        Ok(Self { state, app })
    }

    /// Create the Axum application router
    fn create_app(state: ApiState) -> Router {
        let api_prefix = &state.api_config.api_prefix;

        // API routes
        let api_routes = Router::new()
            // Authentication endpoints
            .route("/auth/login", post(login))
            .route("/auth/register", post(register))
            .route("/auth/me", get(get_current_user))
            .route("/auth/profile", put(update_profile))
            .route("/auth/password", put(change_password))
            .route("/auth/refresh", post(refresh_token))
            .route("/auth/logout", post(logout))
            // File endpoints
            .route("/files", post(upload_file))
            .route("/files/:file_key", get(download_file))
            .route("/files/:file_key", delete(delete_file))
            .route("/files/:file_key/metadata", get(get_file_metadata))
            .route("/files", get(list_files))
            // .route("/search", post(search_files)) // TODO: Implement search_files
            // .route("/stats", get(get_api_stats)) // TODO: Implement get_api_stats
            // .route("/health", get(health_check)) // TODO: Implement health_check
            // Analytics endpoints
            .route("/analytics/system", get(get_system_metrics))
            .route("/analytics/storage", get(get_storage_metrics))
            .route("/analytics/network", get(get_network_metrics))
            // Economy endpoints
            .route("/economy/status", get(get_economy_status))
            .route("/economy/profile", get(get_user_economy_profile))
            .route("/economy/profile", put(update_economy_profile))
            .route("/economy/contribute", post(start_storage_contribution))
            .route("/economy/contribute", get(get_contribution_status))
            .route("/economy/contribute", delete(stop_storage_contribution))
            .route("/economy/tiers", get(get_storage_tiers))
            .route("/economy/upgrade", post(upgrade_storage_tier))
            .route("/economy/verification", get(get_verification_status))
            .route("/economy/verification/challenge", post(respond_to_challenge))
            .route("/economy/transactions", get(get_economy_transactions))
            .route("/economy/quota", get(get_quota_status))
            // Governance endpoints
            // .route("/governance/status", get(get_governance_status)) // TODO: Implement get_governance_status
            // .route("/governance/operators", get(list_operators)) // TODO: Implement list_operators
            // .route("/governance/operators/:operator_id", get(get_operator)) // TODO: Implement get_operator
            // .route(
            //     "/governance/operators/:operator_id/dashboard",
            //     get(get_operator_dashboard),
            // ) // TODO: Implement get_operator_dashboard
            // .route("/governance/network/health", get(get_network_health)) // TODO: Implement get_network_health
            .route("/governance/proposals", get(get_proposals))
            .route("/governance/proposals", post(submit_proposal))
            .route("/governance/proposals/:proposal_id/vote", post(vote_on_proposal))
            // Settings endpoints
            .route("/settings", get(get_user_settings))
            .route("/settings", put(update_user_settings))
            // Admin endpoints (temporarily disabled)
            // .route("/admin/operators", post(register_operator))
            .route("/admin/users", get(get_users))
            .route("/admin/health", get(get_system_health))
            // .route(
            //     "/admin/operators/:operator_id/services",
            //     post(register_service),
            // )
            // .route(
            //     "/admin/operators/:operator_id/services/:service_id/heartbeat",
            //     post(update_service_heartbeat),
            // )
            // .route("/admin/actions", post(execute_admin_action))
            // .route("/admin/actions", get(list_admin_actions))
            // .route("/admin/cleanup/operators", post(cleanup_inactive_operators))
            // Settings endpoints
            .route("/settings", get(get_user_settings))
            .route("/settings", put(update_user_settings))
            // User management endpoints
            .route("/admin/users", get(get_users))
            // System health endpoint
            .route("/admin/health", get(get_system_health))
            // WebSocket endpoint
            .route("/ws", get(websocket_handler))
            .with_state(state.clone());

        let mut app = Router::new().nest(api_prefix, api_routes);

        // Add Swagger UI if enabled
        if state.api_config.enable_swagger {
            app = app.merge(
                SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()),
            );
        }

        // Add middleware layers with security headers
        app = app.layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                // Security headers
                .layer(middleware::from_fn(add_security_headers))
                // CORS with more restrictive settings
                .layer(
                    CorsLayer::new()
                        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
                        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap()) // Vite dev server
                        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
                        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::UPGRADE, header::CONNECTION])
                        .allow_credentials(true),
                )
                .layer(DefaultBodyLimit::max(
                    state.api_config.max_upload_size as usize,
                )),
        );

        app
    }

    /// Start the API server
    pub async fn start(&self) -> DfsResult<()> {
        let addr = format!(
            "{}:{}",
            self.state.api_config.host, self.state.api_config.port
        );
        info!("Starting DataMesh API server on {}", addr);

        if self.state.api_config.enable_https {
            self.start_https_server(&addr).await
        } else {
            self.start_http_server(&addr).await
        }
    }

    /// Start HTTP server
    async fn start_http_server(&self, addr: &str) -> DfsResult<()> {
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| DfsError::Network(format!("Failed to bind to {}: {}", addr, e)))?;

        info!("DataMesh API server listening on http://{}", addr);
        if self.state.api_config.enable_swagger {
            info!("Swagger UI available at: http://{}/swagger-ui", addr);
        }

        axum::serve(listener, self.app.clone())
            .await
            .map_err(|e| DfsError::Network(format!("Server error: {}", e)))?;

        Ok(())
    }

    /// Start HTTPS server with TLS configuration
    async fn start_https_server(&self, addr: &str) -> DfsResult<()> {
        let cert_path = self.state.api_config.cert_path.as_ref().ok_or_else(|| {
            DfsError::Config("HTTPS enabled but no cert_path specified".to_string())
        })?;
        let key_path = self.state.api_config.key_path.as_ref().ok_or_else(|| {
            DfsError::Config("HTTPS enabled but no key_path specified".to_string())
        })?;

        let _config = axum_server::tls_rustls::RustlsConfig::from_pem_file(cert_path, key_path)
            .await
            .map_err(|e| DfsError::Network(format!("Failed to load TLS config: {}", e)))?;

        info!("DataMesh API server listening on https://{}", addr);
        if self.state.api_config.enable_swagger {
            info!("Swagger UI available at: https://{}/swagger-ui", addr);
        }

        // TODO: Fix axum_server compatibility issue
        // axum_server::bind_rustls(addr.parse().unwrap(), config)
        //     .serve(self.app.clone().into_make_service())
        //     .await
        //     .map_err(|e| DfsError::Network(format!("HTTPS server error: {}", e)))?;

        Err(DfsError::Config(
            "HTTPS server temporarily disabled due to compatibility issue".to_string(),
        ))
    }
}

/// Login endpoint
#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials", body = ApiErrorResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse)
    ),
    tag = "auth"
)]
async fn login(
    State(state): State<ApiState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let user = state
        .user_registry
        .authenticate_user(&request.email, &request.password)
        .map_err(|e| ApiError::Unauthorized(format!("Authentication failed: {}", e)))?;

    let token = state
        .auth_service
        .generate_token(&user)
        .map_err(|e| ApiError::InternalServerError(format!("Token generation failed: {}", e)))?;

    let account_type = match user.account_type {
        crate::governance::AccountType::Free { .. } => "free",
        crate::governance::AccountType::Premium { .. } => "premium",
        crate::governance::AccountType::Enterprise { .. } => "enterprise",
    };

    let verification_status = match user.verification_status {
        crate::governance::VerificationStatus::Unverified => "unverified",
        crate::governance::VerificationStatus::EmailVerified => "email_verified",
        crate::governance::VerificationStatus::IdentityVerified => "identity_verified",
    };

    let response = AuthResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: 24 * 3600, // 24 hours
        user: UserInfo {
            user_id: user.user_id.to_string(),
            email: user.email,
            account_type: account_type.to_string(),
            registration_date: user.registration_date,
            verification_status: verification_status.to_string(),
        },
    };

    Ok(Json(response))
}

/// Registration endpoint
#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "Registration successful", body = AuthResponse),
        (status = 400, description = "Registration failed", body = ApiErrorResponse),
        (status = 409, description = "Email already exists", body = ApiErrorResponse)
    ),
    tag = "auth"
)]
async fn register(
    State(state): State<ApiState>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let user = state
        .user_registry
        .register_user(request.email, request.password, request.public_key)
        .map_err(|e| match e {
            crate::error::DfsError::Authentication(msg) if msg.contains("already registered") => {
                ApiError::Conflict("Email already registered".to_string())
            }
            _ => ApiError::BadRequest(format!("Registration failed: {}", e)),
        })?;

    let token = state
        .auth_service
        .generate_token(&user)
        .map_err(|e| ApiError::InternalServerError(format!("Token generation failed: {}", e)))?;

    let account_type = match user.account_type {
        crate::governance::AccountType::Free { .. } => "free",
        crate::governance::AccountType::Premium { .. } => "premium",
        crate::governance::AccountType::Enterprise { .. } => "enterprise",
    };

    let verification_status = match user.verification_status {
        crate::governance::VerificationStatus::Unverified => "unverified",
        crate::governance::VerificationStatus::EmailVerified => "email_verified",
        crate::governance::VerificationStatus::IdentityVerified => "identity_verified",
    };

    let response = AuthResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: 24 * 3600, // 24 hours
        user: UserInfo {
            user_id: user.user_id.to_string(),
            email: user.email,
            account_type: account_type.to_string(),
            registration_date: user.registration_date,
            verification_status: verification_status.to_string(),
        },
    };

    Ok(Json(response))
}

/// Get current user profile
#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    responses(
        (status = 200, description = "User profile retrieved", body = UserInfo),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse)
    ),
    tag = "auth"
)]
async fn get_current_user(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<UserInfo>, ApiError> {
    let user_account = authenticate_user(&headers, &state).await?;
    
    let account_type = match user_account.account_type {
        crate::governance::AccountType::Free { .. } => "free",
        crate::governance::AccountType::Premium { .. } => "premium",
        crate::governance::AccountType::Enterprise { .. } => "enterprise",
    };

    let verification_status = match user_account.verification_status {
        crate::governance::VerificationStatus::Unverified => "unverified",
        crate::governance::VerificationStatus::EmailVerified => "email_verified",
        crate::governance::VerificationStatus::IdentityVerified => "identity_verified",
    };

    let user_info = UserInfo {
        user_id: user_account.user_id.to_string(),
        email: user_account.email,
        account_type: account_type.to_string(),
        registration_date: user_account.registration_date,
        verification_status: verification_status.to_string(),
    };

    Ok(Json(user_info))
}

/// Update user profile
#[utoipa::path(
    put,
    path = "/api/v1/auth/profile",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated successfully", body = UserInfo),
        (status = 400, description = "Invalid request", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse)
    ),
    tag = "auth"
)]
async fn update_profile(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<Json<UserInfo>, ApiError> {
    let user_account = authenticate_user(&headers, &state).await?;
    
    // Update user profile in registry
    let updated_user = state
        .user_registry
        .update_user_profile(&user_account.user_id, request.email, request.display_name)
        .map_err(|e| ApiError::InternalServerError(format!("Failed to update profile: {}", e)))?;

    let account_type = match updated_user.account_type {
        crate::governance::AccountType::Free { .. } => "free",
        crate::governance::AccountType::Premium { .. } => "premium",
        crate::governance::AccountType::Enterprise { .. } => "enterprise",
    };

    let verification_status = match updated_user.verification_status {
        crate::governance::VerificationStatus::Unverified => "unverified",
        crate::governance::VerificationStatus::EmailVerified => "email_verified",
        crate::governance::VerificationStatus::IdentityVerified => "identity_verified",
    };

    let user_info = UserInfo {
        user_id: updated_user.user_id.to_string(),
        email: updated_user.email,
        account_type: account_type.to_string(),
        registration_date: updated_user.registration_date,
        verification_status: verification_status.to_string(),
    };

    Ok(Json(user_info))
}

/// Change password
#[utoipa::path(
    put,
    path = "/api/v1/auth/password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed successfully", body = serde_json::Value),
        (status = 400, description = "Invalid password", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse)
    ),
    tag = "auth"
)]
async fn change_password(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = authenticate_user(&headers, &state).await?;
    
    // Verify current password
    let password_valid = state.user_registry.verify_password(&user.user_id, &request.current_password)
        .map_err(|e| ApiError::InternalServerError(format!("Failed to verify password: {}", e)))?;
    
    if !password_valid {
        return Err(ApiError::BadRequest("Current password is incorrect".to_string()));
    }
    
    // Update password
    state.user_registry.update_password(&user.user_id, &request.new_password)
        .map_err(|e| ApiError::InternalServerError(format!("Failed to update password: {}", e)))?;
    
    Ok(Json(serde_json::json!({
        "message": "Password changed successfully"
    })))
}

/// Refresh token
#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Token refreshed", body = AuthResponse),
        (status = 401, description = "Invalid refresh token", body = ApiErrorResponse)
    ),
    tag = "auth"
)]
async fn refresh_token(
    State(state): State<ApiState>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Validate refresh token and get user
    let user_id = state.auth_service.validate_refresh_token(&request.refresh_token)
        .map_err(|e| ApiError::Unauthorized(format!("Invalid refresh token: {}", e)))?;
    
    let user = state.user_registry.get_user(&user_id)
        .map_err(|e| ApiError::InternalServerError(format!("Failed to get user: {}", e)))?
        .ok_or_else(|| ApiError::Unauthorized("User not found".to_string()))?;
    
    // Generate new access token
    let token = state.auth_service.generate_token(&user_id)
        .map_err(|e| ApiError::InternalServerError(format!("Failed to generate token: {}", e)))?;
    
    let account_type = match user.account_type {
        crate::governance::AccountType::Free { .. } => "free",
        crate::governance::AccountType::Premium { .. } => "premium",
        crate::governance::AccountType::Enterprise { .. } => "enterprise",
    };

    let verification_status = match user.verification_status {
        crate::governance::VerificationStatus::Unverified => "unverified",
        crate::governance::VerificationStatus::EmailVerified => "email_verified",
        crate::governance::VerificationStatus::IdentityVerified => "identity_verified",
    };

    let response = AuthResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: 24 * 3600, // 24 hours
        user: UserInfo {
            user_id: user.user_id.to_string(),
            email: user.email,
            account_type: account_type.to_string(),
            registration_date: user.registration_date,
            verification_status: verification_status.to_string(),
        },
    };

    Ok(Json(response))
}

/// Logout endpoint
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    responses(
        (status = 200, description = "Logout successful"),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse)
    ),
    tag = "auth"
)]
async fn logout(
    State(_state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Validate token exists
    let _token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| ApiError::Unauthorized("Missing or invalid token".to_string()))?;

    // In a production system, you would:
    // 1. Add token to blacklist
    // 2. Invalidate refresh tokens
    // 3. Clear session data

    Ok(Json(serde_json::json!({
        "message": "Logout successful"
    })))
}

/// Get user settings
#[utoipa::path(
    get,
    path = "/api/v1/settings",
    responses(
        (status = 200, description = "User settings retrieved", body = UserSettingsResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse)
    ),
    tag = "settings"
)]
async fn get_user_settings(
    State(_state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<UserSettingsResponse>, ApiError> {
    let _user_account = authenticate_user(&headers, &_state).await?;
    
    // Mock settings response
    let settings = UserSettingsResponse {
        theme: "dark".to_string(),
        language: "en".to_string(),
        notifications_enabled: true,
        email_notifications: true,
        auto_delete_days: 0,
        privacy_mode: false,
    };

    Ok(Json(settings))
}

/// Update user settings
#[utoipa::path(
    put,
    path = "/api/v1/settings",
    request_body = UpdateUserSettingsRequest,
    responses(
        (status = 200, description = "Settings updated successfully", body = UserSettingsResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse)
    ),
    tag = "settings"
)]
async fn update_user_settings(
    State(_state): State<ApiState>,
    headers: HeaderMap,
    Json(request): Json<UpdateUserSettingsRequest>,
) -> Result<Json<UserSettingsResponse>, ApiError> {
    let _user_account = authenticate_user(&headers, &_state).await?;
    
    // In a real implementation, you would update the user's settings
    let settings = UserSettingsResponse {
        theme: request.theme.unwrap_or("dark".to_string()),
        language: request.language.unwrap_or("en".to_string()),
        notifications_enabled: request.notifications_enabled.unwrap_or(true),
        email_notifications: request.email_notifications.unwrap_or(true),
        auto_delete_days: request.auto_delete_days.unwrap_or(0),
        privacy_mode: request.privacy_mode.unwrap_or(false),
    };

    Ok(Json(settings))
}

/// Get users (admin)
#[utoipa::path(
    get,
    path = "/api/v1/admin/users",
    params(
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Page size"),
        ("filter" = Option<String>, Query, description = "Filter users")
    ),
    responses(
        (status = 200, description = "Users retrieved", body = UsersListResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse)
    ),
    tag = "admin"
)]
async fn get_users(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<UsersListResponse>, ApiError> {
    let user_account = authenticate_user(&headers, &state).await?;
    
    // Check admin permissions
    if !matches!(user_account.account_type, crate::governance::AccountType::Enterprise { .. }) {
        return Err(ApiError::Forbidden("Admin access required".to_string()));
    }

    let page: u32 = params.get("page").and_then(|s| s.parse().ok()).unwrap_or(1);
    let page_size: u32 = params.get("page_size").and_then(|s| s.parse().ok()).unwrap_or(20);
    let _filter = params.get("filter");

    // Mock user list
    let users = vec![
        AdminUserResponse {
            user_id: uuid::Uuid::new_v4().to_string(),
            email: "user1@example.com".to_string(),
            account_type: "free".to_string(),
            verification_status: "email_verified".to_string(),
            registration_date: chrono::Utc::now() - chrono::Duration::days(30),
            last_activity: chrono::Utc::now() - chrono::Duration::hours(2),
            reputation_score: 85.0,
            storage_used: 1024 * 1024 * 500, // 500MB
            files_count: 25,
        },
        AdminUserResponse {
            user_id: uuid::Uuid::new_v4().to_string(),
            email: "user2@example.com".to_string(),
            account_type: "premium".to_string(),
            verification_status: "identity_verified".to_string(),
            registration_date: chrono::Utc::now() - chrono::Duration::days(60),
            last_activity: chrono::Utc::now() - chrono::Duration::days(1),
            reputation_score: 92.5,
            storage_used: 1024 * 1024 * 1024 * 5, // 5GB
            files_count: 150,
        },
    ];

    let response = UsersListResponse {
        users,
        total: 2,
        page,
        page_size,
    };

    Ok(Json(response))
}

/// Get system health (admin)
#[utoipa::path(
    get,
    path = "/api/v1/admin/health",
    responses(
        (status = 200, description = "System health retrieved", body = SystemHealthResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse)
    ),
    tag = "admin"
)]
async fn get_system_health(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<SystemHealthResponse>, ApiError> {
    let user_account = authenticate_user(&headers, &state).await?;
    
    // Check admin permissions
    if !matches!(user_account.account_type, crate::governance::AccountType::Enterprise { .. }) {
        return Err(ApiError::Forbidden("Admin access required".to_string()));
    }

    let network_health = state.bootstrap_admin.check_network_health();
    let cache_stats = state.cache_manager.get_stats().await;

    let response = SystemHealthResponse {
        overall_status: "healthy".to_string(),
        uptime_seconds: 3600 * 24 * 7, // 1 week
        cpu_usage: 45.2,
        memory_usage: 62.8,
        disk_usage: 78.5,
        network_status: if network_health.online_percentage > 80.0 { "healthy" } else { "degraded" }.to_string(),
        database_status: "healthy".to_string(),
        cache_status: if cache_stats.hit_ratio > 0.8 { "healthy" } else { "degraded" }.to_string(),
        active_connections: network_health.online_operators as u64,
        total_requests_last_hour: 1250,
        error_rate: 0.02,
        timestamp: chrono::Utc::now(),
    };

    Ok(Json(response))
}

/// Update profile request
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProfileRequest {
    /// User email
    pub email: String,
    /// Display name
    pub display_name: Option<String>,
}

/// Change password request
#[derive(Debug, Deserialize, ToSchema)]
pub struct ChangePasswordRequest {
    /// Current password
    pub current_password: String,
    /// New password
    pub new_password: String,
}

/// Refresh token request
#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshTokenRequest {
    /// Refresh token
    pub refresh_token: String,
}

/// User settings response
#[derive(Debug, Serialize, ToSchema)]
pub struct UserSettingsResponse {
    /// UI theme
    pub theme: String,
    /// Language preference
    pub language: String,
    /// Notifications enabled
    pub notifications_enabled: bool,
    /// Email notifications
    pub email_notifications: bool,
    /// Auto delete days
    pub auto_delete_days: u32,
    /// Privacy mode
    pub privacy_mode: bool,
}

/// Update user settings request
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserSettingsRequest {
    /// UI theme
    pub theme: Option<String>,
    /// Language preference
    pub language: Option<String>,
    /// Notifications enabled
    pub notifications_enabled: Option<bool>,
    /// Email notifications
    pub email_notifications: Option<bool>,
    /// Auto delete days
    pub auto_delete_days: Option<u32>,
    /// Privacy mode
    pub privacy_mode: Option<bool>,
}

/// System metrics response
#[derive(Debug, Serialize, ToSchema)]
pub struct SystemMetricsResponse {
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage percentage
    pub memory_usage: f64,
    /// Disk usage percentage
    pub disk_usage: f64,
    /// Network throughput
    pub network_throughput: f64,
    /// Active connections
    pub active_connections: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Storage metrics response
#[derive(Debug, Serialize, ToSchema)]
pub struct StorageMetricsResponse {
    /// Total storage capacity
    pub total_capacity: u64,
    /// Used storage
    pub used_storage: u64,
    /// Available storage
    pub available_storage: u64,
    /// Files count
    pub files_count: u64,
    /// Average file size
    pub average_file_size: f64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Network metrics response
#[derive(Debug, Serialize, ToSchema)]
pub struct NetworkMetricsResponse {
    /// Total peers
    pub total_peers: u64,
    /// Connected peers
    pub connected_peers: u64,
    /// Network latency
    pub network_latency: f64,
    /// Bandwidth usage
    pub bandwidth_usage: f64,
    /// Packet loss rate
    pub packet_loss_rate: f64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Proposal response
#[derive(Debug, Serialize, ToSchema)]
pub struct ProposalResponse {
    /// Proposal ID
    pub id: String,
    /// Proposal title
    pub title: String,
    /// Proposal description
    pub description: String,
    /// Proposal type
    pub proposal_type: String,
    /// Proposal status
    pub status: String,
    /// Votes for
    pub votes_for: u64,
    /// Votes against
    pub votes_against: u64,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Expires at
    pub expires_at: DateTime<Utc>,
}

/// Submit proposal request
#[derive(Debug, Deserialize, ToSchema)]
pub struct SubmitProposalRequest {
    /// Proposal title
    pub title: String,
    /// Proposal description
    pub description: String,
    /// Proposal type
    pub proposal_type: String,
    /// Proposal data
    pub data: serde_json::Value,
}

/// Vote request
#[derive(Debug, Deserialize, ToSchema)]
pub struct VoteRequest {
    /// Vote (true for yes, false for no)
    pub vote: bool,
    /// Vote weight
    pub weight: Option<f64>,
}

// Moved UsersListResponse definition to end of file to avoid duplicates

/// System health response
#[derive(Debug, Serialize, ToSchema)]
pub struct SystemHealthResponse {
    /// Overall health status
    pub status: String,
    /// Database health
    pub database_health: String,
    /// Network health
    pub network_health: String,
    /// Storage health
    pub storage_health: String,
    /// Websocket connections
    pub websocket_connections: u64,
    /// Uptime seconds
    pub uptime_seconds: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// File upload endpoint
#[utoipa::path(
    post,
    path = "/api/v1/files",
    request_body(content = String, description = "File content", content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "File uploaded successfully", body = FileUploadResponse),
        (status = 400, description = "Invalid file", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 413, description = "File too large", body = ApiErrorResponse)
    ),
    tag = "files"
)]
async fn upload_file(
    State(state): State<ApiState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<FileUploadResponse>, ApiError> {
    // Verify authentication
    let user = authenticate_user(&headers, &state).await?;

    // Process multipart form
    let mut file_data = None;
    let mut file_name = None;
    let mut content_type = None;
    let mut tags = None;
    let mut public_key = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        ApiError::BadRequest(format!("Failed to process multipart data: {}", e))
    })? {
        let field_name = field.name().unwrap_or("").to_string();
        
        match field_name.as_str() {
            "file" => {
                file_name = field.file_name().map(|s| s.to_string());
                content_type = field.content_type().map(|s| s.to_string());
                
                file_data = Some(field.bytes().await.map_err(|e| {
                    ApiError::BadRequest(format!("Failed to read file data: {}", e))
                })?);
            }
            "tags" => {
                if let Ok(tag_data) = field.text().await {
                    tags = Some(tag_data);
                }
            }
            "public_key" => {
                if let Ok(pk_data) = field.text().await {
                    public_key = Some(pk_data);
                }
            }
            _ => {} // Ignore unknown fields
        }
    }

    let file_data = file_data.ok_or_else(|| {
        ApiError::BadRequest("No file data provided".to_string())
    })?;

    let original_file_name = file_name.unwrap_or_else(|| "unnamed_file".to_string());
    
    // Create temporary file for storage
    let temp_dir = std::env::temp_dir();
    let temp_file_path = temp_dir.join(format!("upload_{}", Uuid::new_v4()));
    
    // Write file data to temporary location
    tokio::fs::write(&temp_file_path, &file_data).await.map_err(|e| {
        ApiError::InternalServerError(format!("Failed to write temporary file: {}", e))
    })?;

    // TODO: Send progress update via WebSocket
    let _temp_file_key = temp_file_path.file_name().unwrap().to_string_lossy().to_string();
    // state.websocket_manager.send_file_upload_progress(
    //     temp_file_key.clone(),
    //     25.0,
    //     "Encrypting and sharding file".to_string(),
    // ).await;

    // Store file using distributed storage
    let file_key = match state.file_storage.store_file(
        &temp_file_path,
        &public_key,
        &Some(original_file_name.clone()),
        &tags,
        &state.key_manager,
    ).await {
        Ok(key) => key,
        Err(e) => {
            // Clean up temporary file
            let _ = tokio::fs::remove_file(&temp_file_path).await;
            return Err(ApiError::InternalServerError(format!("Failed to store file: {}", e)));
        }
    };

    // Clean up temporary file
    let _ = tokio::fs::remove_file(&temp_file_path).await;

    // TODO: Send completion update via WebSocket
    // state.websocket_manager.send_file_upload_progress(
    //     file_key.clone(),
    //     100.0,
    //     "Upload complete".to_string(),
    // ).await;

    let response = FileUploadResponse {
        file_key: file_key.clone(),
        file_name: original_file_name,
        file_size: file_data.len() as u64,
        uploaded_at: Utc::now(),
        message: "File uploaded successfully to distributed storage".to_string(),
    };

    Ok(Json(response))
}

/// File download endpoint
#[utoipa::path(
    get,
    path = "/api/v1/files/{file_key}",
    params(
        ("file_key" = String, Path, description = "File key")
    ),
    responses(
        (status = 200, description = "File downloaded successfully", body = Vec<u8>),
        (status = 404, description = "File not found", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse)
    ),
    tag = "files"
)]
async fn download_file(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(file_key): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    // Verify authentication
    let _user = authenticate_user(&headers, &state).await?;

    // TODO: Send progress update via WebSocket
    // state.websocket_manager.send_file_download_progress(
    //     file_key.clone(),
    //     25.0,
    //     "Retrieving file from distributed storage".to_string(),
    // ).await;

    // Create temporary output file
    let temp_dir = std::env::temp_dir();
    let output_path = temp_dir.join(format!("download_{}", Uuid::new_v4()));

    // TODO: Send progress update
    // state.websocket_manager.send_file_download_progress(
    //     file_key.clone(),
    //     50.0,
    //     "Reconstructing file from chunks".to_string(),
    // ).await;

    // Retrieve file using distributed storage
    let file_data = match state.file_storage.retrieve_file(
        &file_key,
        &output_path,
        &None, // private_key - using default
        &state.key_manager,
    ).await {
        Ok(_) => {
            // Read the reconstructed file
            tokio::fs::read(&output_path).await.map_err(|e| {
                ApiError::InternalServerError(format!("Failed to read reconstructed file: {}", e))
            })?
        }
        Err(e) => {
            return Err(ApiError::NotFound(format!("File not found in distributed storage: {}", e)));
        }
    };

    // Clean up temporary file
    let _ = tokio::fs::remove_file(&output_path).await;

    // Get file metadata for proper headers
    let (file_name, content_type) = match state.file_storage.get_file_metadata(&file_key).await {
        Ok(metadata) => {
            let file_name = metadata.file_name.clone();
            (metadata.file_name, detect_content_type(&file_name))
        }
        Err(_) => {
            (format!("file_{}", &file_key[..8]), "application/octet-stream".to_string())
        }
    };

    // TODO: Send completion update via WebSocket
    // state.websocket_manager.send_file_download_progress(
    //     file_key.clone(),
    //     100.0,
    //     "Download complete".to_string(),
    // ).await;

    // Return file with appropriate headers
    let content_disposition = format!("attachment; filename=\"{}\"", file_name);
    let headers = [
        (header::CONTENT_TYPE, content_type.as_str()),
        (header::CONTENT_DISPOSITION, content_disposition.as_str()),
        (header::CACHE_CONTROL, "no-cache"),
    ];

    Ok((headers, file_data))
}

/// Detect content type based on file extension
fn detect_content_type(filename: &str) -> String {
    match std::path::Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .as_deref()
    {
        Some("txt") => "text/plain".to_string(),
        Some("html") => "text/html".to_string(),
        Some("css") => "text/css".to_string(),
        Some("js") => "application/javascript".to_string(),
        Some("json") => "application/json".to_string(),
        Some("pdf") => "application/pdf".to_string(),
        Some("png") => "image/png".to_string(),
        Some("jpg") | Some("jpeg") => "image/jpeg".to_string(),
        Some("gif") => "image/gif".to_string(),
        Some("svg") => "image/svg+xml".to_string(),
        Some("mp4") => "video/mp4".to_string(),
        Some("mp3") => "audio/mpeg".to_string(),
        Some("zip") => "application/zip".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

/// Get system metrics
#[utoipa::path(
    get,
    path = "/api/v1/analytics/system",
    responses(
        (status = 200, description = "System metrics retrieved", body = SystemMetricsResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse)
    ),
    tag = "analytics"
)]
async fn get_system_metrics(
    State(_state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<SystemMetricsResponse>, ApiError> {
    let _user = authenticate_user(&headers, &_state).await?;
    
    // Mock system metrics
    let metrics = SystemMetricsResponse {
        cpu_usage: 45.2,
        memory_usage: 67.8,
        disk_usage: 23.1,
        network_throughput: 1024.5,
        active_connections: 42,
        timestamp: Utc::now(),
    };
    
    Ok(Json(metrics))
}

/// Get storage metrics
#[utoipa::path(
    get,
    path = "/api/v1/analytics/storage",
    responses(
        (status = 200, description = "Storage metrics retrieved", body = StorageMetricsResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse)
    ),
    tag = "analytics"
)]
async fn get_storage_metrics(
    State(_state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<StorageMetricsResponse>, ApiError> {
    let _user = authenticate_user(&headers, &_state).await?;
    
    // Mock storage metrics
    let metrics = StorageMetricsResponse {
        total_capacity: 1000000000000, // 1TB
        used_storage: 450000000000,    // 450GB
        available_storage: 550000000000, // 550GB
        files_count: 15420,
        average_file_size: 29200.5,
        timestamp: Utc::now(),
    };
    
    Ok(Json(metrics))
}

/// Get network metrics
#[utoipa::path(
    get,
    path = "/api/v1/analytics/network",
    responses(
        (status = 200, description = "Network metrics retrieved", body = NetworkMetricsResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse)
    ),
    tag = "analytics"
)]
async fn get_network_metrics(
    State(_state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<NetworkMetricsResponse>, ApiError> {
    let _user = authenticate_user(&headers, &_state).await?;
    
    // Mock network metrics
    let metrics = NetworkMetricsResponse {
        total_peers: 156,
        connected_peers: 142,
        network_latency: 45.3,
        bandwidth_usage: 78.2,
        packet_loss_rate: f64::from_bits(0x3f80000000000000u64), // 0.05
        timestamp: Utc::now(),
    };
    
    Ok(Json(metrics))
}

/// Get governance proposals
#[utoipa::path(
    get,
    path = "/api/v1/governance/proposals",
    responses(
        (status = 200, description = "Proposals retrieved", body = Vec<ProposalResponse>),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse)
    ),
    tag = "governance"
)]
async fn get_proposals(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ProposalResponse>>, ApiError> {
    let _user = authenticate_user(&headers, &state).await?;
    
    // Get proposals from governance service
    let proposals = state.governance_service.get_proposals().await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to get proposals: {}", e)))?;
    
    Ok(Json(proposals))
}

/// Submit governance proposal
#[utoipa::path(
    post,
    path = "/api/v1/governance/proposals",
    request_body = SubmitProposalRequest,
    responses(
        (status = 201, description = "Proposal submitted", body = ProposalResponse),
        (status = 400, description = "Invalid proposal", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse)
    ),
    tag = "governance"
)]
async fn submit_proposal(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(request): Json<SubmitProposalRequest>,
) -> Result<Json<ProposalResponse>, ApiError> {
    let user = authenticate_user(&headers, &state).await?;
    
    // Submit proposal to governance service
    let proposal = state.governance_service.submit_proposal(
        &user.user_id,
        request.title,
        request.description,
        request.proposal_type,
        request.data,
    ).await
    .map_err(|e| ApiError::InternalServerError(format!("Failed to submit proposal: {}", e)))?;
    
    // Send WebSocket notification
    state.websocket_manager.send_governance_update(
        "proposal_submitted".to_string(),
        serde_json::json!({
            "proposal_id": proposal.id,
            "title": proposal.title,
            "submitter": user.user_id.to_string()
        }),
    ).await;
    
    Ok(Json(proposal))
}

/// Vote on proposal
#[utoipa::path(
    post,
    path = "/api/v1/governance/vote/{proposal_id}",
    params(
        ("proposal_id" = String, Path, description = "Proposal ID")
    ),
    request_body = VoteRequest,
    responses(
        (status = 200, description = "Vote recorded", body = serde_json::Value),
        (status = 400, description = "Invalid vote", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 404, description = "Proposal not found", body = ApiErrorResponse)
    ),
    tag = "governance"
)]
async fn vote_on_proposal(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(proposal_id): Path<String>,
    Json(request): Json<VoteRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user = authenticate_user(&headers, &state).await?;
    
    // Record vote in governance service
    state.governance_service.vote_on_proposal(
        &proposal_id,
        &user.user_id,
        request.vote,
        request.weight.unwrap_or(1.0),
    ).await
    .map_err(|e| ApiError::InternalServerError(format!("Failed to record vote: {}", e)))?;
    
    // Send WebSocket notification
    state.websocket_manager.send_governance_update(
        "vote_cast".to_string(),
        serde_json::json!({
            "proposal_id": proposal_id,
            "voter": user.user_id.to_string(),
            "vote": request.vote
        }),
    ).await;
    
    Ok(Json(serde_json::json!({
        "message": "Vote recorded successfully",
        "proposal_id": proposal_id,
        "vote": request.vote
    })))
}

/// List files endpoint
#[utoipa::path(
    get,
    path = "/api/v1/files",
    params(
        ("page" = Option<u32>, Query, description = "Page number"),
        ("page_size" = Option<u32>, Query, description = "Page size"),
        ("tags" = Option<String>, Query, description = "Filter by tags")
    ),
    responses(
        (status = 200, description = "Files retrieved", body = FileListResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse)
    ),
    tag = "files"
)]
async fn list_files(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<FileListResponse>, ApiError> {
    let _user = authenticate_user(&headers, &state).await?;
    
    // Extract pagination parameters
    let page: u32 = params.get("page").and_then(|s| s.parse().ok()).unwrap_or(1);
    let page_size: u32 = params.get("page_size").and_then(|s| s.parse().ok()).unwrap_or(20);
    let tags = params.get("tags");
    
    // Get files from distributed storage
    let file_metadata_list = state.file_storage.list_files(tags.map(|t| t.as_str())).await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to list files: {}", e)))?;
    
    // Convert to API response format
    let files: Vec<FileMetadataResponse> = file_metadata_list.into_iter().map(|metadata| {
        FileMetadataResponse {
            file_key: blake3::hash(metadata.file_name.as_bytes()).to_hex().to_string(),
            file_name: metadata.file_name.clone(),
            original_name: metadata.file_name,
            file_size: metadata.file_size,
            uploaded_at: metadata.upload_time.with_timezone(&Utc),
            tags: metadata.tags,
            public_key: metadata.public_key_hex,
        }
    }).collect();
    
    // Apply pagination
    let total = files.len();
    let start = ((page - 1) * page_size) as usize;
    let end = std::cmp::min(start + page_size as usize, total);
    let paginated_files = if start < total {
        files[start..end].to_vec()
    } else {
        vec![]
    };
    
    let response = FileListResponse {
        files: paginated_files,
        total: total,
        page,
        page_size,
    };
    
    Ok(Json(response))
}

/// Get file metadata endpoint
#[utoipa::path(
    get,
    path = "/api/v1/files/{file_key}/metadata",
    params(
        ("file_key" = String, Path, description = "File key")
    ),
    responses(
        (status = 200, description = "File metadata retrieved", body = FileMetadataResponse),
        (status = 404, description = "File not found", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse)
    ),
    tag = "files"
)]
async fn get_file_metadata(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(file_key): Path<String>,
) -> Result<Json<FileMetadataResponse>, ApiError> {
    let _user = authenticate_user(&headers, &state).await?;
    
    // Get metadata from distributed storage
    let metadata = state.file_storage.get_file_metadata(&file_key).await
        .map_err(|e| match e {
            crate::error::DfsError::FileNotFound(_) => ApiError::NotFound("File not found".to_string()),
            _ => ApiError::InternalServerError(format!("Failed to get file metadata: {}", e)),
        })?;
    
    let response = FileMetadataResponse {
        file_key: file_key.clone(),
        file_name: metadata.file_name.clone(),
        original_name: metadata.file_name,
        file_size: metadata.file_size,
        uploaded_at: metadata.upload_time.with_timezone(&Utc),
        tags: metadata.tags,
        public_key: metadata.public_key_hex,
    };
    
    Ok(Json(response))
}

/// Delete file endpoint
#[utoipa::path(
    delete,
    path = "/api/v1/files/{file_key}",
    params(
        ("file_key" = String, Path, description = "File key")
    ),
    responses(
        (status = 200, description = "File deleted", body = serde_json::Value),
        (status = 404, description = "File not found", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse)
    ),
    tag = "files"
)]
async fn delete_file(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(file_key): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let _user = authenticate_user(&headers, &state).await?;
    
    // Delete from distributed storage
    state.file_storage.delete_file(&file_key).await
        .map_err(|e| match e {
            crate::error::DfsError::FileNotFound(_) => ApiError::NotFound("File not found".to_string()),
            _ => ApiError::InternalServerError(format!("Failed to delete file: {}", e)),
        })?;
    
    Ok(Json(serde_json::json!({
        "message": "File deleted successfully from distributed storage",
        "file_key": file_key
    })))
}

/// Users list response
#[derive(Debug, Serialize, ToSchema)]
pub struct UsersListResponse {
    /// Users
    pub users: Vec<AdminUserResponse>,
    /// Total count
    pub total: u64,
    /// Current page
    pub page: u32,
    /// Page size
    pub page_size: u32,
}

/// Admin user response
#[derive(Debug, Serialize, ToSchema)]
pub struct AdminUserResponse {
    /// User ID
    pub user_id: String,
    /// User email
    pub email: String,
    /// Account type
    pub account_type: String,
    /// Verification status
    pub verification_status: String,
    /// Registration date
    pub registration_date: chrono::DateTime<chrono::Utc>,
    /// Last activity
    pub last_activity: chrono::DateTime<chrono::Utc>,
    /// Reputation score
    pub reputation_score: f64,
    /// Storage used
    pub storage_used: u64,
    /// Files count
    pub files_count: u32,
}

/// System health response detailed
#[derive(Debug, Serialize, ToSchema)]
pub struct SystemHealthResponseDetailed {
    /// Overall health status
    pub overall_status: String,
    /// Uptime seconds
    pub uptime_seconds: u64,
    /// CPU usage
    pub cpu_usage: f64,
    /// Memory usage
    pub memory_usage: f64,
    /// Disk usage
    pub disk_usage: f64,
    /// Network status
    pub network_status: String,
    /// Database status
    pub database_status: String,
    /// Cache status
    pub cache_status: String,
    /// Active connections
    pub active_connections: u64,
    /// Total requests last hour
    pub total_requests_last_hour: u64,
    /// Error rate
    pub error_rate: f64,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// ECONOMY API HANDLERS
// ============================================================================

/// Get economy status endpoint
#[utoipa::path(
    get,
    path = "/api/v1/economy/status",
    responses(
        (status = 200, description = "Economy status retrieved", body = EconomyStatusResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse)
    ),
    tag = "economy"
)]
async fn get_economy_status(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<EconomyStatusResponse>, ApiError> {
    let _user = authenticate_user(&headers, &state).await?;
    
    let status = state.storage_economy.get_economy_status().await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to get economy status: {}", e)))?;
    
    let response = EconomyStatusResponse {
        health: "healthy".to_string(),
        total_contributors: status.total_contributors,
        total_storage_contributed: status.total_storage_contributed,
        active_verifications: status.active_verifications,
        network_utilization: status.network_utilization,
    };
    
    Ok(Json(response))
}

/// Get user economy profile endpoint
#[utoipa::path(
    get,
    path = "/api/v1/economy/profile",
    responses(
        (status = 200, description = "User economy profile retrieved", body = UserEconomyProfileResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 404, description = "Profile not found", body = ApiErrorResponse)
    ),
    tag = "economy"
)]
async fn get_user_economy_profile(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<UserEconomyProfileResponse>, ApiError> {
    let user_id = extract_user_id(&headers, &state).await?;
    
    let stats = state.storage_economy.get_user_storage_statistics(&user_id).await
        .map_err(|e| match e {
            crate::error::DfsError::UserNotFound(_) => ApiError::NotFound("User profile not found".to_string()),
            _ => ApiError::InternalServerError(format!("Failed to get user profile: {}", e)),
        })?;
    
    let response = UserEconomyProfileResponse {
        user_id: stats.user_id,
        tier: stats.storage_tier,
        current_usage: stats.current_usage,
        max_storage: stats.max_storage,
        upload_quota_used: stats.upload_quota_used,
        upload_quota_limit: stats.upload_quota_limit,
        download_quota_used: stats.download_quota_used,
        download_quota_limit: stats.download_quota_limit,
        reputation_score: stats.reputation_score,
        violations_count: stats.violations_count,
        last_activity: stats.last_activity,
        can_contribute: stats.can_contribute,
    };
    
    Ok(Json(response))
}

/// Update economy profile endpoint
#[utoipa::path(
    put,
    path = "/api/v1/economy/profile",
    request_body = UserEconomyProfileResponse,
    responses(
        (status = 200, description = "Profile updated", body = UserEconomyProfileResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse)
    ),
    tag = "economy"
)]
async fn update_economy_profile(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(request): Json<UserEconomyProfileResponse>,
) -> Result<Json<UserEconomyProfileResponse>, ApiError> {
    let user_id = extract_user_id(&headers, &state).await?;
    
    // Update user profile via storage economy service
    let updated_profile = state.storage_economy.update_user_profile(&user_id, &request.tier).await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to update profile: {}", e)))?;
    
    // Return updated profile
    get_user_economy_profile(State(state), headers).await
}

/// Start storage contribution endpoint
#[utoipa::path(
    post,
    path = "/api/v1/economy/contribute",
    request_body = StorageContributionRequest,
    responses(
        (status = 200, description = "Storage contribution started", body = serde_json::Value),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse)
    ),
    tag = "economy"
)]
async fn start_storage_contribution(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(request): Json<StorageContributionRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = extract_user_id(&headers, &state).await?;
    
    let result = state.storage_economy.start_storage_contribution(
        &user_id,
        &std::path::PathBuf::from(&request.storage_path),
        request.amount
    ).await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to start contribution: {}", e)))?;
    
    Ok(Json(serde_json::json!({
        "status": "started",
        "contribution_id": result.contribution_id,
        "message": "Storage contribution started successfully"
    })))
}

/// Get contribution status endpoint
#[utoipa::path(
    get,
    path = "/api/v1/economy/contribute",
    responses(
        (status = 200, description = "Contribution status retrieved", body = serde_json::Value),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse)
    ),
    tag = "economy"
)]
async fn get_contribution_status(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = extract_user_id(&headers, &state).await?;
    
    let status = state.storage_economy.get_contribution_status(&user_id).await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to get contribution status: {}", e)))?;
    
    Ok(Json(serde_json::json!({
        "active": status.active,
        "contributed_amount": status.contributed_amount,
        "verified_amount": status.verified_amount,
        "last_verification": status.last_verification,
        "status": status.status
    })))
}

/// Stop storage contribution endpoint
#[utoipa::path(
    delete,
    path = "/api/v1/economy/contribute",
    responses(
        (status = 200, description = "Storage contribution stopped", body = serde_json::Value),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse)
    ),
    tag = "economy"
)]
async fn stop_storage_contribution(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = extract_user_id(&headers, &state).await?;
    
    state.storage_economy.stop_storage_contribution(&user_id).await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to stop contribution: {}", e)))?;
    
    Ok(Json(serde_json::json!({
        "status": "stopped",
        "message": "Storage contribution stopped successfully"
    })))
}

/// Get storage tiers endpoint
#[utoipa::path(
    get,
    path = "/api/v1/economy/tiers",
    responses(
        (status = 200, description = "Storage tiers retrieved", body = StorageTiersResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse)
    ),
    tag = "economy"
)]
async fn get_storage_tiers(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<StorageTiersResponse>, ApiError> {
    let _user = authenticate_user(&headers, &state).await?;
    
    let tiers = vec![
        StorageTierInfo {
            name: "Free".to_string(),
            max_storage: 1024 * 1024 * 1024, // 1GB
            upload_quota: 100 * 1024 * 1024, // 100MB
            download_quota: 500 * 1024 * 1024, // 500MB
            monthly_cost: None,
            description: "Basic free tier with limited storage".to_string(),
        },
        StorageTierInfo {
            name: "Contributor".to_string(),
            max_storage: 4 * 1024 * 1024 * 1024, // 4GB
            upload_quota: 1024 * 1024 * 1024, // 1GB
            download_quota: 2 * 1024 * 1024 * 1024, // 2GB
            monthly_cost: None,
            description: "Contribute storage to earn additional space".to_string(),
        },
        StorageTierInfo {
            name: "Premium".to_string(),
            max_storage: 100 * 1024 * 1024 * 1024, // 100GB
            upload_quota: 10 * 1024 * 1024 * 1024, // 10GB
            download_quota: 20 * 1024 * 1024 * 1024, // 20GB
            monthly_cost: Some(9.99),
            description: "Premium tier with high storage limits".to_string(),
        },
        StorageTierInfo {
            name: "Enterprise".to_string(),
            max_storage: 1024 * 1024 * 1024 * 1024, // 1TB
            upload_quota: 100 * 1024 * 1024 * 1024, // 100GB
            download_quota: 200 * 1024 * 1024 * 1024, // 200GB
            monthly_cost: Some(99.99),
            description: "Enterprise tier with unlimited features".to_string(),
        },
    ];
    
    Ok(Json(StorageTiersResponse { tiers }))
}

/// Upgrade storage tier endpoint
#[utoipa::path(
    post,
    path = "/api/v1/economy/upgrade",
    request_body = StorageTierUpgradeRequest,
    responses(
        (status = 200, description = "Tier upgrade initiated", body = serde_json::Value),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse)
    ),
    tag = "economy"
)]
async fn upgrade_storage_tier(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(request): Json<StorageTierUpgradeRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = extract_user_id(&headers, &state).await?;
    
    let result = state.storage_economy.upgrade_storage_tier(
        &user_id,
        &request.target_tier,
        &request.payment_method
    ).await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to upgrade tier: {}", e)))?;
    
    Ok(Json(serde_json::json!({
        "status": "upgrade_initiated",
        "upgrade_id": result.upgrade_id,
        "target_tier": request.target_tier,
        "message": "Tier upgrade initiated successfully"
    })))
}

/// Get verification status endpoint
#[utoipa::path(
    get,
    path = "/api/v1/economy/verification",
    responses(
        (status = 200, description = "Verification status retrieved", body = serde_json::Value),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse)
    ),
    tag = "economy"
)]
async fn get_verification_status(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = extract_user_id(&headers, &state).await?;
    
    let status = state.storage_economy.get_verification_status(&user_id).await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to get verification status: {}", e)))?;
    
    Ok(Json(serde_json::json!({
        "verification_active": status.verification_active,
        "last_challenge": status.last_challenge,
        "success_rate": status.success_rate,
        "pending_challenges": status.pending_challenges,
        "reputation_score": status.reputation_score
    })))
}

/// Respond to challenge endpoint
#[utoipa::path(
    post,
    path = "/api/v1/economy/verification/challenge",
    request_body = ChallengeResponseRequest,
    responses(
        (status = 200, description = "Challenge response submitted", body = serde_json::Value),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 400, description = "Bad request", body = ApiErrorResponse)
    ),
    tag = "economy"
)]
async fn respond_to_challenge(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(request): Json<ChallengeResponseRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = extract_user_id(&headers, &state).await?;
    
    let result = state.storage_economy.respond_to_challenge(
        &user_id,
        &request.challenge_id,
        &request.response_data
    ).await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to respond to challenge: {}", e)))?;
    
    Ok(Json(serde_json::json!({
        "status": "response_submitted",
        "challenge_id": request.challenge_id,
        "verification_successful": result.verification_successful,
        "message": "Challenge response submitted successfully"
    })))
}

/// Get economy transactions endpoint
#[utoipa::path(
    get,
    path = "/api/v1/economy/transactions",
    responses(
        (status = 200, description = "Economy transactions retrieved", body = Vec<EconomyTransactionResponse>),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse)
    ),
    tag = "economy"
)]
async fn get_economy_transactions(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<EconomyTransactionResponse>>, ApiError> {
    let user_id = extract_user_id(&headers, &state).await?;
    
    let limit: usize = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50);
    let offset: usize = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);
    
    let transactions = state.storage_economy.get_user_transactions(&user_id, limit, offset).await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to get transactions: {}", e)))?;
    
    let response: Vec<EconomyTransactionResponse> = transactions.into_iter().map(|tx| {
        EconomyTransactionResponse {
            transaction_id: tx.transaction_id,
            transaction_type: tx.transaction_type,
            amount: tx.amount,
            description: tx.description,
            timestamp: tx.timestamp,
            status: tx.status,
        }
    }).collect();
    
    Ok(Json(response))
}

/// Get quota status endpoint
#[utoipa::path(
    get,
    path = "/api/v1/economy/quota",
    responses(
        (status = 200, description = "Quota status retrieved", body = QuotaStatusResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse)
    ),
    tag = "economy"
)]
async fn get_quota_status(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<Json<QuotaStatusResponse>, ApiError> {
    let user_id = extract_user_id(&headers, &state).await?;
    
    let stats = state.storage_economy.get_user_storage_statistics(&user_id).await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to get quota status: {}", e)))?;
    
    let response = QuotaStatusResponse {
        storage_used: stats.current_usage,
        storage_limit: stats.max_storage,
        upload_quota_used: stats.upload_quota_used,
        upload_quota_limit: stats.upload_quota_limit,
        download_quota_used: stats.download_quota_used,
        download_quota_limit: stats.download_quota_limit,
        next_reset: Utc::now() + chrono::Duration::days(30), // Monthly reset
    };
    
    Ok(Json(response))
}
