use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

// Global counter for unique test IDs
static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

// Helper function to get the path to the compiled binary
fn get_binary_path() -> PathBuf {
    let mut path = env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    path.pop(); // Remove 'deps' directory
    path.push("wld");
    path
}

// Helper function to create a unique temporary home directory for testing
fn setup_temp_home() -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let unique_name = format!("wld_mcp_test_home_{}_{}", timestamp, counter);

    let temp_dir = env::temp_dir();
    let temp_home = temp_dir.join(unique_name);

    // Create the temporary home directory
    fs::create_dir_all(&temp_home).expect("Failed to create temp home directory");

    temp_home
}

// Helper function to clean up temporary home directory
fn cleanup_temp_home(path: &PathBuf) {
    if path.exists() {
        let _ = fs::remove_dir_all(path);
    }
}

// Helper to add a device to config
fn add_device_to_config(temp_home: &PathBuf, name: &str, ip: &str) {
    let binary_path = get_binary_path();
    Command::new(binary_path)
        .args(&["add", name, ip])
        .env("HOME", temp_home)
        .output()
        .expect("Failed to add device");
}

// Helper to send MCP requests via a bash script with timeout
fn send_mcp_request_via_script(
    temp_home: &PathBuf,
    requests: Vec<&str>,
) -> Result<String, String> {
    let binary_path = get_binary_path();
    
    // Create a temporary script to run the MCP server with input
    // For tests involving network calls, we need to wait longer for timeouts
    let script = format!(
        r#"#!/bin/bash
export HOME={}
{{
{}
  sleep 12
}} | timeout 20 {} mcp 2>/dev/null
"#,
        temp_home.display(),
        requests
            .iter()
            .map(|r| format!("  echo '{}'", r))
            .collect::<Vec<_>>()
            .join("\n"),
        binary_path.display()
    );
    
    let script_path = temp_home.join("test_script.sh");
    fs::write(&script_path, script).map_err(|e| format!("Failed to write script: {}", e))?;
    
    let output = Command::new("bash")
        .arg(&script_path)
        .output()
        .map_err(|e| format!("Failed to run script: {}", e))?;
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[test]
fn test_mcp_server_initialization() {
    let temp_home = setup_temp_home();
    
    let init_request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}"#;
    
    let output =
        send_mcp_request_via_script(&temp_home, vec![init_request]).expect("Failed to send request");
    
    cleanup_temp_home(&temp_home);
    
    assert!(!output.is_empty(), "Should receive response");
    assert!(
        output.contains("\"result\""),
        "Response should contain result field"
    );
    assert!(
        output.contains("protocolVersion"),
        "Response should contain protocolVersion"
    );
}

#[test]
fn test_mcp_tools_list() {
    let temp_home = setup_temp_home();
    
    let init_request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}"#;
    let init_notification = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    let tools_request = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;
    
    let output = send_mcp_request_via_script(
        &temp_home,
        vec![init_request, init_notification, tools_request],
    )
    .expect("Failed to send request");
    
    cleanup_temp_home(&temp_home);
    
    assert!(
        output.contains("wled_devices"),
        "Response should list wled_devices tool"
    );
    assert!(
        output.contains("wled_on"),
        "Response should list wled_on tool"
    );
    assert!(
        output.contains("wled_off"),
        "Response should list wled_off tool"
    );
}

#[test]
fn test_mcp_wled_devices_no_devices() {
    let temp_home = setup_temp_home();
    
    let init_request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}"#;
    let init_notification = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    let call_request =
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"wled_devices","arguments":{}}}"#;
    
    let output = send_mcp_request_via_script(
        &temp_home,
        vec![init_request, init_notification, call_request],
    )
    .expect("Failed to send request");
    
    cleanup_temp_home(&temp_home);
    
    assert!(
        output.contains("No devices saved"),
        "Response should indicate no devices saved"
    );
}

#[test]
fn test_mcp_wled_devices_with_devices() {
    let temp_home = setup_temp_home();
    
    // Add test devices
    add_device_to_config(&temp_home, "living_room", "192.168.1.100");
    add_device_to_config(&temp_home, "bedroom", "192.168.1.101");
    
    let init_request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}"#;
    let init_notification = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    let call_request =
        r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"wled_devices","arguments":{}}}"#;
    
    let output = send_mcp_request_via_script(
        &temp_home,
        vec![init_request, init_notification, call_request],
    )
    .expect("Failed to send request");
    
    cleanup_temp_home(&temp_home);
    
    assert!(
        output.contains("living_room") && output.contains("192.168.1.100"),
        "Response should contain living_room device"
    );
    assert!(
        output.contains("bedroom") && output.contains("192.168.1.101"),
        "Response should contain bedroom device"
    );
}

#[test]
fn test_mcp_wled_on_no_default() {
    let temp_home = setup_temp_home();
    
    let init_request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}"#;
    let init_notification = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    let call_request =
        r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"wled_on","arguments":{}}}"#;
    
    let output = send_mcp_request_via_script(
        &temp_home,
        vec![init_request, init_notification, call_request],
    )
    .expect("Failed to send request");
    
    cleanup_temp_home(&temp_home);
    
    // Should get error about no default device
    assert!(
        output.contains("No device") || output.contains("isError"),
        "Response should indicate missing device error: {}",
        output
    );
}

#[test]
fn test_mcp_wled_on_with_device_parameter() {
    let temp_home = setup_temp_home();
    
    // Add a test device
    add_device_to_config(&temp_home, "test_light", "192.168.1.50");
    
    let init_request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}"#;
    let init_notification = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    let call_request = r#"{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"wled_on","arguments":{"device":"test_light"}}}"#;
    
    let output = send_mcp_request_via_script(
        &temp_home,
        vec![init_request, init_notification, call_request],
    )
    .expect("Failed to send request");
    
    cleanup_temp_home(&temp_home);
    
    // The device doesn't actually exist, so we'll get a connection error,
    // but the MCP call should still succeed in accepting the parameter
    assert!(
        output.contains("content") || output.contains("isError"),
        "Response should contain result: {}",
        output
    );
}

#[test]
fn test_mcp_wled_off_with_ip_address() {
    let temp_home = setup_temp_home();
    
    let init_request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}"#;
    let init_notification = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    let call_request = r#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"wled_off","arguments":{"device":"192.168.1.99"}}}"#;
    
    let output = send_mcp_request_via_script(
        &temp_home,
        vec![init_request, init_notification, call_request],
    )
    .expect("Failed to send request");
    
    cleanup_temp_home(&temp_home);
    
    // The device doesn't actually exist, so we'll get a connection error,
    // but the MCP call should still succeed in accepting the IP parameter
    assert!(
        output.contains("content") || output.contains("isError"),
        "Response should contain result: {}",
        output
    );
}


