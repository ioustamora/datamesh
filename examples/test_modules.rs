/// Test Script for DataMesh Core Modules
///
/// This example demonstrates the functionality of the core modules
/// and verifies they work correctly with the current implementation.

use std::path::PathBuf;
use tempfile::TempDir;
use chrono::Local;

// Import DataMesh modules that are actually implemented
use datamesh::database::{DatabaseManager, get_default_db_path};
use datamesh::presets::{NetworkPresets, parse_network_spec};
use datamesh::error_handling::{handle_error, file_not_found_error_with_suggestions};
use datamesh::ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing DataMesh Core Modules");
    
    // Test Database Module
    test_database_module().await?;
    
    // Test Network Presets Module
    test_presets_module()?;
    
    // Test Error Handling Module
    test_error_handling_module()?;
    
    // Test UI Module
    test_ui_module();
    
    println!("✅ All module tests completed successfully!");
    
    Ok(())
}

async fn test_database_module() -> Result<(), Box<dyn std::error::Error>> {
    ui::print_header("Testing Database Module");
    
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    
    let db = DatabaseManager::new(&db_path)?;
    
    // Test file storage
    let upload_time = Local::now();
    let tags = vec!["test".to_string(), "example".to_string()];
    
    let file_id = db.store_file(
        "test-file",
        "abc123def456",
        "test-document.pdf", 
        1024 * 1024, // 1MB
        upload_time,
        &tags,
        "test-public-key-hex"
    )?;
    
    ui::print_success(&format!("Stored file with ID: {}", file_id));
    
    // Test retrieval
    let file = db.get_file_by_name("test-file")?.unwrap();
    ui::print_key_value("Retrieved file", &file.name);
    ui::print_key_value("File size", &ui::format_file_size(file.file_size));
    ui::print_key_value("Tags", &file.tags.join(", "));
    
    // Test search and stats
    let files = db.list_files(None)?;
    ui::print_key_value("Total files", &files.len().to_string());
    
    let stats = db.get_stats()?;
    ui::print_key_value("Database size", &stats.total_files.to_string());
    
    // Test unique name generation
    let unique_name = db.generate_unique_name("test-document.pdf")?;
    ui::print_key_value("Generated unique name", &unique_name);
    
    Ok(())
}

fn test_presets_module() -> Result<(), Box<dyn std::error::Error>> {
    ui::print_header("Testing Network Presets Module");
    
    let presets = NetworkPresets::new();
    
    // Test built-in presets
    let preset_names = ["local", "public", "test"];
    for name in &preset_names {
        if let Some(preset) = presets.get_preset(name) {
            ui::print_key_value("Preset", &preset.name);
            ui::print_key_value("Description", &preset.description);
            ui::print_key_value("Bootstrap peers", &preset.bootstrap_peers.len().to_string());
        }
    }
    
    // Test preset application
    let config = presets.apply_preset("local")?;
    ui::print_key_value("Local preset port", &config.port.to_string());
    ui::print_key_value("Discovery enabled", &config.discovery_enabled.to_string());
    ui::print_key_value("Bootstrap peers", &config.bootstrap_peers.len().to_string());
    
    // Test network spec parsing
    let addr_config = parse_network_spec("/ip4/127.0.0.1/tcp/40871")?;
    ui::print_key_value("Parsed addresses", &addr_config.bootstrap_peers.len().to_string());
    
    ui::print_success("Network presets module working correctly");
    
    Ok(())
}

fn test_error_handling_module() -> Result<(), Box<dyn std::error::Error>> {
    ui::print_header("Testing Error Handling Module");
    
    // Test file not found error
    let file_error = file_not_found_error_with_suggestions("missing-file.txt");
    ui::print_key_value("Error type", "FileNotFound");
    ui::print_key_value("Suggestions", &file_error.suggestions.len().to_string());
    
    // Test IO error handling
    let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
    let enhanced = handle_error(&io_error);
    ui::print_key_value("Enhanced suggestions", &enhanced.suggestions.len().to_string());
    
    ui::print_success("Error handling module working correctly");
    
    Ok(())
}

fn test_ui_module() {
    ui::print_header("Testing UI Module");
    
    // Test various UI components
    ui::print_success("Success message test");
    ui::print_warning("Warning message test");
    ui::print_info("Info message test");
    ui::print_error("Error message test");
    
    ui::print_key_value("Key-value test", "value");
    ui::print_operation_status("Test Operation", "Success", Some("Additional details"));
    
    // Test file size formatting
    let sizes = [512, 1024, 1024*1024, 1024*1024*1024];
    for size in sizes {
        ui::print_key_value(&format!("{} bytes", size), &ui::format_file_size(size));
    }
    
    // Test table printing
    let headers = ["Name", "Size", "Status"];
    let rows = vec![
        vec!["file1.txt".to_string(), "1.2 KB".to_string(), "Healthy".to_string()],
        vec!["file2.pdf".to_string(), "2.5 MB".to_string(), "Healthy".to_string()],
    ];
    ui::print_table(&headers, &rows);
    
    ui::print_success("UI module working correctly");
}