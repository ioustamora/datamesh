/// Debug script to diagnose startup issues
use std::process::Command;

fn main() {
    println!("=== DataMesh Startup Diagnosis ===\n");
    
    // Test 1: Check if binary exists and is executable
    println!("1. Checking binary status...");
    match Command::new("./target/debug/datamesh")
        .arg("--version")
        .output() {
        Ok(output) => {
            println!("✅ Binary is executable");
            if !output.status.success() {
                println!("⚠️  Version command failed: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => {
            println!("❌ Binary not found or not executable: {}", e);
            return;
        }
    }
    
    // Test 2: Check wizard trigger (no args)
    println!("\n2. Testing wizard trigger...");
    match Command::new("./target/debug/datamesh")
        .output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            println!("Exit code: {}", output.status.code().unwrap_or(-1));
            if !stdout.is_empty() {
                println!("STDOUT:\n{}", stdout);
            }
            if !stderr.is_empty() {
                println!("STDERR:\n{}", stderr);
            }
        }
        Err(e) => {
            println!("❌ Failed to run wizard: {}", e);
        }
    }
    
    // Test 3: Check bootstrap command
    println!("\n3. Testing bootstrap command...");
    match Command::new("./target/debug/datamesh")
        .arg("bootstrap")
        .arg("--port")
        .arg("40871")
        .output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            println!("Exit code: {}", output.status.code().unwrap_or(-1));
            if !stdout.is_empty() {
                println!("STDOUT:\n{}", stdout);
            }
            if !stderr.is_empty() {
                println!("STDERR:\n{}", stderr);
            }
        }
        Err(e) => {
            println!("❌ Failed to run bootstrap: {}", e);
        }
    }
    
    // Test 4: Check interactive command with manual bootstrap
    println!("\n4. Testing interactive with bootstrap...");
    match Command::new("./target/debug/datamesh")
        .arg("interactive")
        .arg("--bootstrap-addr")
        .arg("/ip4/127.0.0.1/tcp/40871")
        .output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            println!("Exit code: {}", output.status.code().unwrap_or(-1));
            if !stdout.is_empty() {
                println!("STDOUT:\n{}", stdout);
            }
            if !stderr.is_empty() {
                println!("STDERR:\n{}", stderr);
            }
        }
        Err(e) => {
            println!("❌ Failed to run interactive: {}", e);
        }
    }
    
    // Test 5: Show help
    println!("\n5. Testing help output...");
    match Command::new("./target/debug/datamesh")
        .arg("--help")
        .output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("Help output length: {} chars", stdout.len());
            if stdout.contains("bootstrap") {
                println!("✅ Bootstrap command found in help");
            } else {
                println!("❌ Bootstrap command not found in help");
            }
        }
        Err(e) => {
            println!("❌ Failed to get help: {}", e);
        }
    }
}