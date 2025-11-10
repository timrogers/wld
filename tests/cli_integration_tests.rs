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
    let unique_name = format!("wld_test_home_{timestamp}_{counter}");

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

// Helper function to run command with a temporary home directory
fn run_command_with_temp_home(args: &[&str], temp_home: &PathBuf) -> std::process::Output {
    let binary_path = get_binary_path();

    Command::new(binary_path)
        .args(args)
        .env("HOME", temp_home)
        .output()
        .expect("Failed to execute command")
}

#[test]
fn test_add_device() {
    let temp_home = setup_temp_home();

    let output = run_command_with_temp_home(&["add", "test_device", "192.168.1.100"], &temp_home);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Added device 'test_device' with IP 192.168.1.100"));
    assert!(stdout.contains("Set 'test_device' as the default device"));

    cleanup_temp_home(&temp_home);
}

#[test]
fn test_add_and_list_devices() {
    let temp_home = setup_temp_home();

    // Add first device
    let output1 = run_command_with_temp_home(&["add", "living_room", "192.168.1.100"], &temp_home);
    assert!(output1.status.success());

    // Add second device
    let output2 = run_command_with_temp_home(&["add", "bedroom", "192.168.1.101"], &temp_home);
    assert!(output2.status.success());

    // List devices
    let output3 = run_command_with_temp_home(&["ls"], &temp_home);
    assert!(output3.status.success());

    let stdout = String::from_utf8_lossy(&output3.stdout);
    assert!(stdout.contains("Saved devices:"));
    assert!(stdout.contains("living_room - 192.168.1.100 (default)"));
    assert!(stdout.contains("bedroom - 192.168.1.101"));

    cleanup_temp_home(&temp_home);
}

#[test]
fn test_list_no_devices() {
    let temp_home = setup_temp_home();

    let output = run_command_with_temp_home(&["ls"], &temp_home);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No devices saved"));

    cleanup_temp_home(&temp_home);
}

#[test]
fn test_set_default_device() {
    let temp_home = setup_temp_home();

    // Add two devices
    run_command_with_temp_home(&["add", "living_room", "192.168.1.100"], &temp_home);
    run_command_with_temp_home(&["add", "bedroom", "192.168.1.101"], &temp_home);

    // Set bedroom as default
    let output = run_command_with_temp_home(&["set-default", "bedroom"], &temp_home);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Set 'bedroom' as the default device"));

    // List devices to verify
    let list_output = run_command_with_temp_home(&["ls"], &temp_home);
    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(list_stdout.contains("bedroom - 192.168.1.101 (default)"));

    cleanup_temp_home(&temp_home);
}

#[test]
fn test_set_default_nonexistent_device() {
    let temp_home = setup_temp_home();

    // Add one device
    run_command_with_temp_home(&["add", "living_room", "192.168.1.100"], &temp_home);

    // Try to set nonexistent device as default
    let output = run_command_with_temp_home(&["set-default", "kitchen"], &temp_home);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Device 'kitchen' not found"));

    cleanup_temp_home(&temp_home);
}

#[test]
fn test_delete_device() {
    let temp_home = setup_temp_home();

    // Add two devices
    run_command_with_temp_home(&["add", "living_room", "192.168.1.100"], &temp_home);
    run_command_with_temp_home(&["add", "bedroom", "192.168.1.101"], &temp_home);

    // Delete living_room
    let output = run_command_with_temp_home(&["delete", "living_room"], &temp_home);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Deleted device 'living_room'"));

    // List devices to verify
    let list_output = run_command_with_temp_home(&["ls"], &temp_home);
    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(!list_stdout.contains("living_room"));
    assert!(list_stdout.contains("bedroom"));

    cleanup_temp_home(&temp_home);
}

#[test]
fn test_delete_nonexistent_device() {
    let temp_home = setup_temp_home();

    // Add one device
    run_command_with_temp_home(&["add", "living_room", "192.168.1.100"], &temp_home);

    // Try to delete nonexistent device
    let output = run_command_with_temp_home(&["delete", "kitchen"], &temp_home);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Device 'kitchen' not found"));

    cleanup_temp_home(&temp_home);
}

#[test]
fn test_delete_default_device_reassigns() {
    let temp_home = setup_temp_home();

    // Add two devices
    run_command_with_temp_home(&["add", "living_room", "192.168.1.100"], &temp_home);
    run_command_with_temp_home(&["add", "bedroom", "192.168.1.101"], &temp_home);

    // living_room is default, delete it
    let output = run_command_with_temp_home(&["delete", "living_room"], &temp_home);
    assert!(output.status.success());

    // List devices to verify bedroom is now default
    let list_output = run_command_with_temp_home(&["ls"], &temp_home);
    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(list_stdout.contains("bedroom - 192.168.1.101 (default)"));

    cleanup_temp_home(&temp_home);
}

#[test]
fn test_multiple_operations_sequence() {
    let temp_home = setup_temp_home();

    // Add three devices
    run_command_with_temp_home(&["add", "device1", "192.168.1.10"], &temp_home);
    run_command_with_temp_home(&["add", "device2", "192.168.1.20"], &temp_home);
    run_command_with_temp_home(&["add", "device3", "192.168.1.30"], &temp_home);

    // Set device2 as default
    run_command_with_temp_home(&["set-default", "device2"], &temp_home);

    // Delete device1
    run_command_with_temp_home(&["delete", "device1"], &temp_home);

    // List and verify
    let list_output = run_command_with_temp_home(&["ls"], &temp_home);
    let list_stdout = String::from_utf8_lossy(&list_output.stdout);

    assert!(!list_stdout.contains("device1"));
    assert!(list_stdout.contains("device2 - 192.168.1.20 (default)"));
    assert!(list_stdout.contains("device3 - 192.168.1.30"));

    cleanup_temp_home(&temp_home);
}

#[test]
fn test_brightness_command_requires_value() {
    let temp_home = setup_temp_home();

    // Add a device
    run_command_with_temp_home(&["add", "test_device", "192.168.1.100"], &temp_home);

    // Try to run brightness without a value - should fail
    let output = run_command_with_temp_home(&["brightness"], &temp_home);
    assert!(!output.status.success());

    cleanup_temp_home(&temp_home);
}

#[test]
fn test_brightness_command_accepts_valid_range() {
    let temp_home = setup_temp_home();

    // Add a device
    run_command_with_temp_home(&["add", "test_device", "192.168.1.100"], &temp_home);

    // Test minimum value
    let output_min = run_command_with_temp_home(&["brightness", "0"], &temp_home);
    // Note: This will fail to connect to a real device, but we're testing command parsing
    // The error would be a connection error, not a parsing error
    let stderr_min = String::from_utf8_lossy(&output_min.stderr);
    // Should not contain argument parsing errors
    assert!(!stderr_min.contains("invalid value"));
    assert!(!stderr_min.contains("error: invalid"));

    // Test maximum value
    let output_max = run_command_with_temp_home(&["brightness", "255"], &temp_home);
    let stderr_max = String::from_utf8_lossy(&output_max.stderr);
    assert!(!stderr_max.contains("invalid value"));
    assert!(!stderr_max.contains("error: invalid"));

    cleanup_temp_home(&temp_home);
}

#[test]
fn test_brightness_command_rejects_out_of_range() {
    let temp_home = setup_temp_home();

    // Add a device
    run_command_with_temp_home(&["add", "test_device", "192.168.1.100"], &temp_home);

    // Test value over 255
    let output = run_command_with_temp_home(&["brightness", "256"], &temp_home);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // u8 max is 255, so 256 should be rejected
    assert!(stderr.contains("256") || stderr.contains("invalid"));

    cleanup_temp_home(&temp_home);
}

#[test]
fn test_brightness_command_with_specific_device() {
    let temp_home = setup_temp_home();

    // Add two devices
    run_command_with_temp_home(&["add", "device1", "192.168.1.100"], &temp_home);
    run_command_with_temp_home(&["add", "device2", "192.168.1.101"], &temp_home);

    // Try to set brightness on specific device
    let output = run_command_with_temp_home(&["brightness", "128", "-d", "device2"], &temp_home);
    // Should parse successfully (will fail on network, but that's expected)
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("error: invalid"));

    cleanup_temp_home(&temp_home);
}
