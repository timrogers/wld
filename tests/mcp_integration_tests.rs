use serde_json::Value;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
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
    let unique_name = format!("wld_mcp_test_home_{timestamp}_{counter}");

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
        .args(["add", name, ip])
        .env("HOME", temp_home)
        .output()
        .expect("Failed to add device");
}

// Helper to send MCP requests via a bash script with timeout
fn send_mcp_request_via_script(temp_home: &Path, requests: Vec<&str>) -> Result<String, String> {
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
            .map(|r| format!("  echo '{r}'"))
            .collect::<Vec<_>>()
            .join("\n"),
        binary_path.display()
    );

    let script_path = temp_home.join("test_script.sh");
    fs::write(&script_path, script).map_err(|e| format!("Failed to write script: {e}"))?;

    let output = Command::new("bash")
        .arg(&script_path)
        .output()
        .map_err(|e| format!("Failed to run script: {e}"))?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[test]
fn test_mcp_server_initialization() {
    let temp_home = setup_temp_home();

    let init_request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}"#;

    let output = send_mcp_request_via_script(&temp_home, vec![init_request])
        .expect("Failed to send request");

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
    let call_request = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"wled_devices","arguments":{}}}"#;

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
    let call_request = r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"wled_devices","arguments":{}}}"#;

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
    let call_request = r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"wled_on","arguments":{}}}"#;

    let output = send_mcp_request_via_script(
        &temp_home,
        vec![init_request, init_notification, call_request],
    )
    .expect("Failed to send request");

    cleanup_temp_home(&temp_home);

    // Should get error about no default device
    assert!(
        output.contains("No device") || output.contains("isError"),
        "Response should indicate missing device error: {output}"
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
        "Response should contain result: {output}"
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
        "Response should contain result: {output}"
    );
}

#[test]
fn test_mcp_tools_have_valid_schemas() {
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

    // Parse the JSON-RPC response
    let lines: Vec<&str> = output.lines().collect();
    let response_line = lines
        .iter()
        .find(|line| line.contains("\"id\":2"))
        .expect("Should find tools/list response");

    let response: Value =
        serde_json::from_str(response_line).expect("Response should be valid JSON");

    let tools = response["result"]["tools"]
        .as_array()
        .expect("Should have tools array");

    assert!(!tools.is_empty(), "Should have at least one tool");

    for tool in tools {
        let tool_name = tool["name"].as_str().expect("Tool should have name");
        let input_schema = &tool["inputSchema"];

        // Verify inputSchema exists and is an object
        assert!(
            input_schema.is_object(),
            "Tool '{tool_name}' inputSchema should be an object"
        );

        // Verify inputSchema has type: "object"
        let schema_type = input_schema["type"]
            .as_str()
            .unwrap_or_else(|| panic!("Tool '{tool_name}' inputSchema should have 'type' field"));

        assert_eq!(
            schema_type, "object",
            "Tool '{tool_name}' inputSchema type should be 'object', got '{schema_type}'"
        );

        // Verify it has a $schema field
        assert!(
            input_schema["$schema"].is_string(),
            "Tool '{tool_name}' inputSchema should have $schema field"
        );
    }

    // Verify we have the expected tools
    let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();

    assert!(
        tool_names.contains(&"wled_devices"),
        "Should have wled_devices tool"
    );
    assert!(tool_names.contains(&"wled_on"), "Should have wled_on tool");
    assert!(
        tool_names.contains(&"wled_off"),
        "Should have wled_off tool"
    );
}

#[test]
fn test_wled_devices_schema_structure() {
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

    let lines: Vec<&str> = output.lines().collect();
    let response_line = lines
        .iter()
        .find(|line| line.contains("\"id\":2"))
        .expect("Should find tools/list response");

    let response: Value =
        serde_json::from_str(response_line).expect("Response should be valid JSON");

    let tools = response["result"]["tools"]
        .as_array()
        .expect("Should have tools array");

    let wled_devices = tools
        .iter()
        .find(|t| t["name"] == "wled_devices")
        .expect("Should have wled_devices tool");

    let input_schema = &wled_devices["inputSchema"];

    // wled_devices takes no parameters, but should still have valid schema
    assert_eq!(
        input_schema["type"].as_str().unwrap(),
        "object",
        "wled_devices should have object type schema"
    );

    // Should have EmptyParams title
    assert_eq!(
        input_schema["title"].as_str().unwrap(),
        "EmptyParams",
        "wled_devices should use EmptyParams schema"
    );
}

#[test]
fn test_wled_device_params_schema_structure() {
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

    let lines: Vec<&str> = output.lines().collect();
    let response_line = lines
        .iter()
        .find(|line| line.contains("\"id\":2"))
        .expect("Should find tools/list response");

    let response: Value =
        serde_json::from_str(response_line).expect("Response should be valid JSON");

    let tools = response["result"]["tools"]
        .as_array()
        .expect("Should have tools array");

    // Test both wled_on and wled_off as they use the same schema
    for tool_name in &["wled_on", "wled_off"] {
        let tool = tools
            .iter()
            .find(|t| t["name"] == *tool_name)
            .unwrap_or_else(|| panic!("Should have {tool_name} tool"));

        let input_schema = &tool["inputSchema"];

        assert_eq!(
            input_schema["type"].as_str().unwrap(),
            "object",
            "{tool_name} should have object type schema"
        );

        assert_eq!(
            input_schema["title"].as_str().unwrap(),
            "WledDeviceParams",
            "{tool_name} should use WledDeviceParams schema"
        );

        // Verify properties object exists
        let properties = input_schema["properties"]
            .as_object()
            .unwrap_or_else(|| panic!("{tool_name} should have properties"));

        // Verify device property
        assert!(
            properties.contains_key("device"),
            "{tool_name} should have device property"
        );

        let device_prop = &properties["device"];
        assert_eq!(
            device_prop["type"].as_str().unwrap(),
            "string",
            "{tool_name} device property should be string type"
        );

        assert!(
            device_prop["nullable"].as_bool().unwrap(),
            "{tool_name} device property should be nullable"
        );

        assert!(
            device_prop["description"].is_string(),
            "{tool_name} device property should have description"
        );
    }
}

#[test]
fn test_mcp_wled_status_no_devices() {
    let temp_home = setup_temp_home();

    let init_request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}"#;
    let init_notification = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    let call_request = r#"{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"wled_status","arguments":{}}}"#;

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
fn test_mcp_wled_status_with_devices() {
    let temp_home = setup_temp_home();

    // Add test devices
    add_device_to_config(&temp_home, "living_room", "192.168.1.100");
    add_device_to_config(&temp_home, "bedroom", "192.168.1.101");

    let init_request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}"#;
    let init_notification = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    let call_request = r#"{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"wled_status","arguments":{}}}"#;

    let output = send_mcp_request_via_script(
        &temp_home,
        vec![init_request, init_notification, call_request],
    )
    .expect("Failed to send request");

    cleanup_temp_home(&temp_home);

    assert!(
        output.contains("Checking status of all devices"),
        "Response should contain status check message"
    );
    assert!(
        output.contains("living_room") && output.contains("192.168.1.100"),
        "Response should contain living_room device"
    );
    assert!(
        output.contains("bedroom") && output.contains("192.168.1.101"),
        "Response should contain bedroom device"
    );
    // Since devices don't actually exist, they should be unreachable
    assert!(
        output.contains("UNREACHABLE"),
        "Response should indicate devices are unreachable"
    );
}

#[test]
fn test_mcp_tools_list_includes_status() {
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
        output.contains("wled_status"),
        "Response should list wled_status tool"
    );
}

