/// Database Module
///
/// This module provides a local SQLite database for storing file metadata,
/// including human-readable names, tags, and file information. This enables
/// users to manage their files with memorable names instead of cryptographic keys.
///
/// The database stores:
/// - File aliases (human-readable names)
/// - File metadata (size, upload time, tags)
/// - File keys for retrieval
/// - File health information

use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use rusqlite::{params, Connection, Row, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a file entry in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub id: i64,
    pub name: String,
    pub file_key: String,
    pub original_filename: String,
    pub file_size: u64,
    pub upload_time: DateTime<Local>,
    pub tags: Vec<String>,
    pub public_key_hex: String,
    pub chunks_total: u32,
    pub chunks_healthy: u32,
}

/// Database manager for file metadata storage
#[derive(Debug)]
pub struct DatabaseManager {
    connection: Connection,
}

impl DatabaseManager {
    /// Create a new database manager with the specified database path
    pub fn new(db_path: &PathBuf) -> Result<Self> {
        // Ensure the parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create database directory: {:?}", parent))?;
        }

        let connection = Connection::open(db_path)
            .with_context(|| format!("Failed to open database: {:?}", db_path))?;

        let manager = DatabaseManager { connection };
        manager.initialize_schema()?;
        Ok(manager)
    }

    /// Initialize the database schema
    fn initialize_schema(&self) -> Result<()> {
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                file_key TEXT NOT NULL UNIQUE,
                original_filename TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                upload_time TEXT NOT NULL,
                tags TEXT NOT NULL DEFAULT '',
                public_key_hex TEXT NOT NULL,
                chunks_total INTEGER NOT NULL DEFAULT 6,
                chunks_healthy INTEGER NOT NULL DEFAULT 6,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Create indexes for better performance
        self.connection.execute(
            "CREATE INDEX IF NOT EXISTS idx_files_name ON files(name)",
            [],
        )?;
        self.connection.execute(
            "CREATE INDEX IF NOT EXISTS idx_files_key ON files(file_key)",
            [],
        )?;
        self.connection.execute(
            "CREATE INDEX IF NOT EXISTS idx_files_tags ON files(tags)",
            [],
        )?;

        Ok(())
    }

    /// Store a new file entry
    pub fn store_file(
        &self,
        name: &str,
        file_key: &str,
        original_filename: &str,
        file_size: u64,
        upload_time: DateTime<Local>,
        tags: &[String],
        public_key_hex: &str,
    ) -> Result<i64> {
        let tags_str = tags.join(",");
        let upload_time_str = upload_time.to_rfc3339();

        let _id = self.connection.execute(
            "INSERT INTO files (name, file_key, original_filename, file_size, upload_time, tags, public_key_hex)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![name, file_key, original_filename, file_size, upload_time_str, tags_str, public_key_hex],
        )?;

        Ok(self.connection.last_insert_rowid())
    }

    /// Get file entry by name
    pub fn get_file_by_name(&self, name: &str) -> Result<Option<FileEntry>> {
        let mut stmt = self.connection.prepare(
            "SELECT id, name, file_key, original_filename, file_size, upload_time, tags, public_key_hex, chunks_total, chunks_healthy
             FROM files WHERE name = ?1"
        )?;

        let file_entry = stmt.query_row(params![name], |row| {
            Ok(self.row_to_file_entry(row)?)
        }).optional()?;

        Ok(file_entry)
    }

    /// Get file entry by file key
    pub fn get_file_by_key(&self, file_key: &str) -> Result<Option<FileEntry>> {
        let mut stmt = self.connection.prepare(
            "SELECT id, name, file_key, original_filename, file_size, upload_time, tags, public_key_hex, chunks_total, chunks_healthy
             FROM files WHERE file_key = ?1"
        )?;

        let file_entry = stmt.query_row(params![file_key], |row| {
            Ok(self.row_to_file_entry(row)?)
        }).optional()?;

        Ok(file_entry)
    }

    /// List all files, optionally filtered by tags
    pub fn list_files(&self, tag_filter: Option<&str>) -> Result<Vec<FileEntry>> {
        let mut stmt = if let Some(tag) = tag_filter {
            let query = "SELECT id, name, file_key, original_filename, file_size, upload_time, tags, public_key_hex, chunks_total, chunks_healthy
                         FROM files WHERE tags LIKE ? ORDER BY upload_time DESC";
            let mut stmt = self.connection.prepare(query)?;
            let file_entries = stmt.query_map([format!("%{}%", tag)], |row| {
                Ok(self.row_to_file_entry(row)?)
            })?;
            return Ok(file_entries.collect::<Result<Vec<_>, _>>()?);
        } else {
            let query = "SELECT id, name, file_key, original_filename, file_size, upload_time, tags, public_key_hex, chunks_total, chunks_healthy
                         FROM files ORDER BY upload_time DESC";
            self.connection.prepare(query)?
        };

        let file_entries = stmt.query_map([], |row| {
            Ok(self.row_to_file_entry(row)?)
        })?;

        let mut files = Vec::new();
        for entry in file_entries {
            files.push(entry?);
        }

        Ok(files)
    }

    /// Update file health information
    pub fn update_file_health(&self, file_key: &str, chunks_healthy: u32) -> Result<()> {
        self.connection.execute(
            "UPDATE files SET chunks_healthy = ?1 WHERE file_key = ?2",
            params![chunks_healthy, file_key],
        )?;
        Ok(())
    }

    /// Delete a file entry
    pub fn delete_file(&self, name: &str) -> Result<bool> {
        let rows_affected = self.connection.execute(
            "DELETE FROM files WHERE name = ?1",
            params![name],
        )?;
        Ok(rows_affected > 0)
    }

    /// Check if a name is already taken
    pub fn is_name_taken(&self, name: &str) -> Result<bool> {
        let mut stmt = self.connection.prepare("SELECT COUNT(*) FROM files WHERE name = ?1")?;
        let count: i64 = stmt.query_row(params![name], |row| row.get(0))?;
        Ok(count > 0)
    }

    /// Rename a file in the database
    pub fn rename_file(&self, old_name: &str, new_name: &str) -> Result<()> {
        let rows_affected = self.connection.execute(
            "UPDATE files SET name = ?1 WHERE name = ?2",
            params![new_name, old_name],
        )?;
        
        if rows_affected == 0 {
            return Err(anyhow::anyhow!("File not found: {}", old_name));
        }
        
        Ok(())
    }

    /// List files by tag
    pub fn list_files_by_tag(&self, tag: &str) -> Result<Vec<FileEntry>> {
        let mut stmt = self.connection.prepare(
            "SELECT id, name, file_key, original_filename, file_size, upload_time, tags, public_key_hex, chunks_total, chunks_healthy
             FROM files WHERE tags LIKE ? ORDER BY upload_time DESC"
        )?;
        
        let file_entries = stmt.query_map([format!("%{}%", tag)], |row| {
            let tags_str: String = row.get(6)?;
            let tags: Vec<String> = if tags_str.is_empty() {
                Vec::new()
            } else {
                tags_str.split(',').map(|s| s.trim().to_string()).collect()
            };
            
            let upload_time_str: String = row.get(5)?;
            let upload_time = DateTime::parse_from_rfc3339(&upload_time_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(5, "upload_time".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Local::now().timezone());
            
            Ok(FileEntry {
                id: row.get(0)?,
                name: row.get(1)?,
                file_key: row.get(2)?,
                original_filename: row.get(3)?,
                file_size: row.get(4)?,
                upload_time,
                tags,
                public_key_hex: row.get(7)?,
                chunks_total: row.get(8)?,
                chunks_healthy: row.get(9)?,
            })
        })?.collect::<std::result::Result<Vec<FileEntry>, rusqlite::Error>>()?;
        
        Ok(file_entries)
    }

    /// Update tags for a file
    pub fn update_file_tags(&self, name: &str, tags: &[String]) -> Result<()> {
        let tags_str = tags.join(",");
        let rows_affected = self.connection.execute(
            "UPDATE files SET tags = ?1 WHERE name = ?2",
            params![tags_str, name],
        )?;
        
        if rows_affected == 0 {
            return Err(anyhow::anyhow!("File not found: {}", name));
        }
        
        Ok(())
    }

    /// Search files by pattern (name, filename, or tags)
    pub fn search_files(&self, pattern: &str) -> Result<Vec<FileEntry>> {
        let search_pattern = format!("%{}%", pattern);
        let mut stmt = self.connection.prepare(
            "SELECT id, name, file_key, original_filename, file_size, upload_time, tags, public_key_hex, chunks_total, chunks_healthy
             FROM files 
             WHERE name LIKE ?1 OR original_filename LIKE ?1 OR tags LIKE ?1
             ORDER BY upload_time DESC"
        )?;
        
        let file_entries = stmt.query_map([&search_pattern], |row| {
            let tags_str: String = row.get(6)?;
            let tags: Vec<String> = if tags_str.is_empty() {
                Vec::new()
            } else {
                tags_str.split(',').map(|s| s.trim().to_string()).collect()
            };
            
            let upload_time_str: String = row.get(5)?;
            let upload_time = DateTime::parse_from_rfc3339(&upload_time_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(5, "upload_time".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Local::now().timezone());
            
            Ok(FileEntry {
                id: row.get(0)?,
                name: row.get(1)?,
                file_key: row.get(2)?,
                original_filename: row.get(3)?,
                file_size: row.get(4)?,
                upload_time,
                tags,
                public_key_hex: row.get(7)?,
                chunks_total: row.get(8)?,
                chunks_healthy: row.get(9)?,
            })
        })?.collect::<std::result::Result<Vec<FileEntry>, rusqlite::Error>>()?;
        
        Ok(file_entries)
    }

    /// Generate a unique name based on the original filename
    pub fn generate_unique_name(&self, original_filename: &str) -> Result<String> {
        // Remove extension and clean up the name
        let base_name = if let Some(stem) = std::path::Path::new(original_filename).file_stem() {
            stem.to_string_lossy().to_string()
        } else {
            original_filename.to_string()
        };

        // Replace invalid characters with hyphens
        let clean_name = base_name
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
            .collect::<String>()
            .to_lowercase();

        // Try the clean name first
        if !self.is_name_taken(&clean_name)? {
            return Ok(clean_name);
        }

        // If taken, try with numbers
        for i in 1..1000 {
            let candidate = format!("{}-{}", clean_name, i);
            if !self.is_name_taken(&candidate)? {
                return Ok(candidate);
            }
        }

        // If all else fails, use a timestamp
        let timestamp = Local::now().format("%Y%m%d-%H%M%S");
        Ok(format!("{}-{}", clean_name, timestamp))
    }

    /// Get database statistics
    pub fn get_stats(&self) -> Result<DatabaseStats> {
        let mut stmt = self.connection.prepare(
            "SELECT COUNT(*) as total_files, 
                    SUM(file_size) as total_size,
                    AVG(chunks_healthy * 1.0 / chunks_total) as avg_health
             FROM files"
        )?;

        let stats = stmt.query_row([], |row| {
            Ok(DatabaseStats {
                total_files: row.get(0)?,
                total_size: row.get::<_, Option<i64>>(1)?.unwrap_or(0) as u64,
                average_health: row.get::<_, Option<f64>>(2)?.unwrap_or(1.0),
            })
        })?;

        Ok(stats)
    }

    /// Convert a database row to a FileEntry
    fn row_to_file_entry(&self, row: &Row) -> Result<FileEntry, rusqlite::Error> {
        let upload_time_str: String = row.get(5)?;
        let upload_time = DateTime::parse_from_rfc3339(&upload_time_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(5, "upload_time".to_string(), rusqlite::types::Type::Text))?
            .with_timezone(&Local);

        let tags_str: String = row.get(6)?;
        let tags = if tags_str.is_empty() {
            Vec::new()
        } else {
            tags_str.split(',').map(|s| s.trim().to_string()).collect()
        };

        Ok(FileEntry {
            id: row.get(0)?,
            name: row.get(1)?,
            file_key: row.get(2)?,
            original_filename: row.get(3)?,
            file_size: row.get(4)?,
            upload_time,
            tags,
            public_key_hex: row.get(7)?,
            chunks_total: row.get(8)?,
            chunks_healthy: row.get(9)?,
        })
    }
}

/// Database statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_files: i64,
    pub total_size: u64,
    pub average_health: f64,
}

/// Get the default database path
pub fn get_default_db_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
        .context("Could not determine config directory")?;
    
    let datamesh_dir = config_dir.join("datamesh");
    Ok(datamesh_dir.join("files.db"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_database_operations() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();
        
        let db = DatabaseManager::new(&db_path).unwrap();
        
        // Test storing a file
        let upload_time = Local::now();
        let id = db.store_file(
            "test-file",
            "abc123",
            "test.txt",
            1024,
            upload_time,
            &["test".to_string(), "example".to_string()],
            "public_key_hex"
        ).unwrap();
        
        assert!(id > 0);
        
        // Test retrieving by name
        let file = db.get_file_by_name("test-file").unwrap().unwrap();
        assert_eq!(file.name, "test-file");
        assert_eq!(file.file_key, "abc123");
        assert_eq!(file.tags, vec!["test", "example"]);
        
        // Test retrieving by key
        let file = db.get_file_by_key("abc123").unwrap().unwrap();
        assert_eq!(file.name, "test-file");
        
        // Test listing files
        let files = db.list_files(None).unwrap();
        assert_eq!(files.len(), 1);
        
        // Test filtering by tags
        let files = db.list_files(Some("test")).unwrap();
        assert_eq!(files.len(), 1);
        
        let files = db.list_files(Some("nonexistent")).unwrap();
        assert_eq!(files.len(), 0);
        
        // Test name generation
        let name = db.generate_unique_name("test.txt").unwrap();
        assert_eq!(name, "test"); // Should not conflict with "test-file"
        
        // Test stats
        let stats = db.get_stats().unwrap();
        assert_eq!(stats.total_files, 1);
        assert_eq!(stats.total_size, 1024);
    }
}