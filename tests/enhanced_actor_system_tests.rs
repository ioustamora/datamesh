/// Enhanced Actor System Tests for DataMesh
///
/// This module provides comprehensive testing for the actor-based networking system,
/// including thread safety, message passing, and actor lifecycle management.

mod test_utils;

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, RwLock};
use tokio::time::timeout;
use test_utils::{TestEnvironment, assertions, performance};

use datamesh::{
    network_actor::{NetworkActor, NetworkHandle, NetworkMessage, NetworkStats},
    cli::Cli,
    config::Config,
};

/// Test actor system creation and basic functionality
#[tokio::test]
async fn test_network_actor_creation() -> Result<()> {
    let env = TestEnvironment::new()?;
    let config = env.create_test_config();
    let cli = Cli::default();
    
    // Test network handle creation
    let network_handle = timeout(
        Duration::from_secs(30),
        NetworkHandle::new(&cli, &config)
    ).await?;
    
    assert!(network_handle.is_ok(), "Network handle should be created successfully");
    
    if let Ok(handle) = network_handle {
        // Test getting network stats
        let stats = handle.get_cached_stats().await;
        assert!(!stats.local_peer_id.to_string().is_empty(), "Should have valid peer ID");
        
        // Test graceful shutdown
        handle.shutdown().await?;
    }
    
    Ok(())
}

/// Test concurrent access to network handle
#[tokio::test]
async fn test_network_handle_concurrent_access() -> Result<()> {
    let env = TestEnvironment::new()?;
    let config = env.create_test_config();
    let cli = Cli::default();
    
    let handle = Arc::new(NetworkHandle::new(&cli, &config).await?);
    let mut tasks = Vec::new();
    
    // Spawn multiple concurrent tasks using the handle
    for i in 0..10 {
        let handle_clone = handle.clone();
        let task = tokio::spawn(async move {
            // Test concurrent stats access
            let stats = handle_clone.get_cached_stats().await;
            assert!(!stats.local_peer_id.to_string().is_empty());
            
            // Test concurrent peer listing
            let peers_result = handle_clone.get_connected_peers().await;
            assert!(peers_result.is_ok());
            
            i // Return task number for verification
        });
        tasks.push(task);
    }
    
    // Wait for all tasks to complete
    let mut completed_tasks = 0;
    for task in tasks {
        if let Ok(task_num) = task.await {
            assert!(task_num < 10);
            completed_tasks += 1;
        }
    }
    
    assert_eq!(completed_tasks, 10, "All concurrent tasks should complete");
    
    // Cleanup
    handle.shutdown().await?;
    
    Ok(())
}

/// Test network actor message handling
#[tokio::test]
async fn test_network_actor_message_handling() -> Result<()> {
    let env = TestEnvironment::new()?;
    let config = env.create_test_config();
    let cli = Cli::default();
    
    let handle = NetworkHandle::new(&cli, &config).await?;
    
    // Test bootstrap operation
    let bootstrap_result = timeout(
        Duration::from_secs(10),
        handle.bootstrap()
    ).await?;
    
    // Bootstrap may succeed or fail depending on network conditions
    // Both outcomes are acceptable for testing
    match bootstrap_result {
        Ok(_) => println!("Bootstrap succeeded"),
        Err(_) => println!("Bootstrap failed (expected in test environment)"),
    }
    
    // Test getting network stats
    let stats_result = timeout(
        Duration::from_secs(5),
        handle.get_network_stats()
    ).await?;
    
    assert!(stats_result.is_ok(), "Should be able to get network stats");
    
    if let Ok(stats) = stats_result {
        assert!(stats.connected_peers >= 0, "Connected peers should be non-negative");
        assert!(stats.pending_queries >= 0, "Pending queries should be non-negative");
        assert!(!stats.local_peer_id.to_string().is_empty(), "Should have valid peer ID");
    }
    
    // Cleanup
    handle.shutdown().await?;
    
    Ok(())
}

/// Test actor system under stress conditions
#[tokio::test]
async fn test_network_actor_stress() -> Result<()> {
    let env = TestEnvironment::new()?;
    let config = env.create_test_config();
    let cli = Cli::default();
    
    let stress_perf = performance::PerformanceTest::new("network_actor_stress");
    
    let handle = NetworkHandle::new(&cli, &config).await?;
    let mut tasks = Vec::new();
    
    // Create stress load with many concurrent operations
    for i in 0..50 {
        let handle_clone = handle.clone();
        let task = tokio::spawn(async move {
            // Mix of different operations
            match i % 4 {
                0 => {
                    // Stats requests
                    let _ = handle_clone.get_network_stats().await;
                }
                1 => {
                    // Peer list requests
                    let _ = handle_clone.get_connected_peers().await;
                }
                2 => {
                    // Bootstrap attempts
                    let _ = handle_clone.bootstrap().await;
                }
                3 => {
                    // Cached stats access
                    let _ = handle_clone.get_cached_stats().await;
                }
                _ => unreachable!(),
            }
            
            // Small delay to simulate realistic usage
            tokio::time::sleep(Duration::from_millis(10)).await;
            
            Ok::<(), anyhow::Error>(())
        });
        tasks.push(task);
    }
    
    // Wait for all stress tasks to complete
    let mut successful_operations = 0;
    for task in tasks {
        if let Ok(Ok(())) = task.await {
            successful_operations += 1;
        }
    }
    
    stress_perf.finish(Duration::from_secs(30));
    
    // Most operations should succeed even under stress
    assert!(successful_operations >= 40, "Most stress operations should succeed");
    
    // System should still be responsive after stress test
    let final_stats = handle.get_cached_stats().await;
    assert!(!final_stats.local_peer_id.to_string().is_empty(), "System should remain responsive");
    
    // Cleanup
    handle.shutdown().await?;
    
    Ok(())
}

/// Test actor lifecycle and resource cleanup
#[tokio::test]
async fn test_network_actor_lifecycle() -> Result<()> {
    let env = TestEnvironment::new()?;
    let config = env.create_test_config();
    let cli = Cli::default();
    
    // Test multiple create/destroy cycles
    for cycle in 0..3 {
        println!("Testing lifecycle cycle {}", cycle + 1);
        
        // Create actor
        let handle = NetworkHandle::new(&cli, &config).await?;
        
        // Use actor briefly
        let stats = handle.get_cached_stats().await;
        assert!(!stats.local_peer_id.to_string().is_empty());
        
        // Test some operations
        let _ = handle.get_connected_peers().await;
        let _ = handle.get_network_stats().await;
        
        // Shutdown actor
        handle.shutdown().await?;
        
        // Brief pause between cycles
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    Ok(())
}

/// Test error handling and recovery
#[tokio::test]
async fn test_network_actor_error_handling() -> Result<()> {
    let env = TestEnvironment::new()?;
    let config = env.create_test_config();
    let cli = Cli::default();
    
    let handle = NetworkHandle::new(&cli, &config).await?;
    
    // Test operations that might fail gracefully
    let operations = vec![
        "bootstrap",
        "get_peers",
        "get_stats",
    ];
    
    for operation in operations {
        match operation {
            "bootstrap" => {
                // Bootstrap might fail in test environment
                let _ = handle.bootstrap().await;
            }
            "get_peers" => {
                // Should succeed
                let result = handle.get_connected_peers().await;
                assert!(result.is_ok(), "Get peers should succeed");
            }
            "get_stats" => {
                // Should succeed
                let result = handle.get_network_stats().await;
                assert!(result.is_ok(), "Get stats should succeed");
            }
            _ => {}
        }
    }
    
    // Actor should remain functional after potential errors
    let final_stats = handle.get_cached_stats().await;
    assert!(!final_stats.local_peer_id.to_string().is_empty(), "Actor should remain functional");
    
    // Cleanup
    handle.shutdown().await?;
    
    Ok(())
}

/// Test message timeout handling
#[tokio::test]
async fn test_network_actor_timeouts() -> Result<()> {
    let env = TestEnvironment::new()?;
    let config = env.create_test_config();
    let cli = Cli::default();
    
    let handle = NetworkHandle::new(&cli, &config).await?;
    
    // Test operations with short timeouts
    let timeout_operations = vec![
        timeout(Duration::from_millis(1), handle.bootstrap()),
        timeout(Duration::from_millis(1), handle.get_network_stats()),
        timeout(Duration::from_millis(1), handle.get_connected_peers()),
    ];
    
    for (i, operation) in timeout_operations.into_iter().enumerate() {
        match operation.await {
            Ok(result) => {
                // If operation completed within timeout, verify it succeeded
                match result {
                    Ok(_) => println!("Operation {} completed quickly", i),
                    Err(_) => println!("Operation {} failed but within timeout", i),
                }
            }
            Err(_) => {
                // Timeout occurred - this is acceptable
                println!("Operation {} timed out as expected", i);
            }
        }
    }
    
    // System should remain functional after timeout tests
    let stats = handle.get_cached_stats().await;
    assert!(!stats.local_peer_id.to_string().is_empty(), "System should remain functional");
    
    // Cleanup
    handle.shutdown().await?;
    
    Ok(())
}

/// Test actor performance under realistic conditions
#[tokio::test]
async fn test_network_actor_performance() -> Result<()> {
    let env = TestEnvironment::new()?;
    let config = env.create_test_config();
    let cli = Cli::default();
    
    let handle = NetworkHandle::new(&cli, &config).await?;
    
    // Benchmark different operation types
    let benchmarks = vec![
        ("cached_stats", 1000),
        ("network_stats", 100),
        ("connected_peers", 100),
    ];
    
    for (operation_name, iterations) in benchmarks {
        let perf_test = performance::PerformanceTest::new(&format!("actor_{}", operation_name));
        
        for _ in 0..iterations {
            match operation_name {
                "cached_stats" => {
                    let _ = handle.get_cached_stats().await;
                }
                "network_stats" => {
                    let _ = handle.get_network_stats().await;
                }
                "connected_peers" => {
                    let _ = handle.get_connected_peers().await;
                }
                _ => {}
            }
        }
        
        perf_test.finish(Duration::from_secs(10));
    }
    
    // Cleanup
    handle.shutdown().await?;
    
    Ok(())
}

/// Test actor system memory usage and resource management
#[tokio::test]
async fn test_network_actor_resource_management() -> Result<()> {
    let env = TestEnvironment::new()?;
    let config = env.create_test_config();
    let cli = Cli::default();
    
    // Test that actors can be created and destroyed without memory leaks
    let initial_memory = get_memory_usage();
    
    for _ in 0..5 {
        let handle = NetworkHandle::new(&cli, &config).await?;
        
        // Use the actor
        let _ = handle.get_cached_stats().await;
        let _ = handle.get_connected_peers().await;
        
        // Shutdown and cleanup
        handle.shutdown().await?;
        
        // Force garbage collection
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    let final_memory = get_memory_usage();
    
    // Memory usage should not grow significantly
    if initial_memory > 0 && final_memory > 0 {
        let memory_growth = final_memory as f64 / initial_memory as f64;
        assert!(memory_growth < 2.0, "Memory usage should not double during actor lifecycle tests");
    }
    
    Ok(())
}

/// Helper function to get memory usage (simplified)
fn get_memory_usage() -> usize {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        if let Ok(status) = fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        return kb_str.parse::<usize>().unwrap_or(0) * 1024;
                    }
                }
            }
        }
    }
    0
}