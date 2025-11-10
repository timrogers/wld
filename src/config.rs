use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub devices: HashMap<String, String>, // name -> ip mapping
    pub default_device: Option<String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            devices: HashMap::new(),
            default_device: None,
        }
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(&self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    pub fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let base_dirs = BaseDirs::new().ok_or("Could not find home directory")?;
        Ok(base_dirs.home_dir().join(".wld.toml"))
    }

    pub fn add_device(&mut self, name: String, ip: String) {
        self.devices.insert(name.clone(), ip);

        // If this is the first device, make it default
        if self.devices.len() == 1 {
            self.default_device = Some(name);
        }
    }

    pub fn remove_device(&mut self, name: &str) -> Result<(), String> {
        if !self.devices.contains_key(name) {
            return Err(format!("Device '{name}' not found"));
        }

        self.devices.remove(name);

        // Clear default if we removed the default device
        if self.default_device.as_deref() == Some(name) {
            self.default_device = None;

            // If there's another device, make it the default
            if let Some(first_name) = self.devices.keys().next() {
                self.default_device = Some(first_name.clone());
            }
        }

        Ok(())
    }

    pub fn set_default(&mut self, name: &str) -> Result<(), String> {
        if !self.devices.contains_key(name) {
            return Err(format!("Device '{name}' not found"));
        }

        self.default_device = Some(name.to_string());
        Ok(())
    }

    pub fn get_device_ip(&self, name_or_ip: Option<&str>) -> Result<String, String> {
        // If specific name/IP provided, use it
        if let Some(identifier) = name_or_ip {
            // Check if it's a device name
            if let Some(ip) = self.devices.get(identifier) {
                return Ok(ip.clone());
            }
            // Otherwise treat it as an IP address
            return Ok(identifier.to_string());
        }

        // Use default device
        if let Some(default_name) = &self.default_device {
            if let Some(ip) = self.devices.get(default_name) {
                return Ok(ip.clone());
            }
        }

        Err("No device specified and no default device set".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    // Helper function to create a temporary config file path
    fn temp_config_path() -> PathBuf {
        let temp_dir = std::env::temp_dir();
        let unique_name = format!("wld_test_config_{}.toml", std::process::id());
        temp_dir.join(unique_name)
    }

    // Helper function to clean up temporary config file
    fn cleanup_config(path: &PathBuf) {
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn test_new_config() {
        let config = Config::new();
        assert!(config.devices.is_empty());
        assert!(config.default_device.is_none());
    }

    #[test]
    fn test_add_device() {
        let mut config = Config::new();
        config.add_device("living_room".to_string(), "192.168.1.100".to_string());

        assert_eq!(config.devices.len(), 1);
        assert_eq!(
            config.devices.get("living_room"),
            Some(&"192.168.1.100".to_string())
        );
        assert_eq!(config.default_device, Some("living_room".to_string()));
    }

    #[test]
    fn test_add_multiple_devices() {
        let mut config = Config::new();
        config.add_device("living_room".to_string(), "192.168.1.100".to_string());
        config.add_device("bedroom".to_string(), "192.168.1.101".to_string());

        assert_eq!(config.devices.len(), 2);
        // First device should remain default
        assert_eq!(config.default_device, Some("living_room".to_string()));
    }

    #[test]
    fn test_remove_device() {
        let mut config = Config::new();
        config.add_device("living_room".to_string(), "192.168.1.100".to_string());
        config.add_device("bedroom".to_string(), "192.168.1.101".to_string());

        let result = config.remove_device("living_room");
        assert!(result.is_ok());
        assert_eq!(config.devices.len(), 1);
        // bedroom should become the new default
        assert_eq!(config.default_device, Some("bedroom".to_string()));
    }

    #[test]
    fn test_remove_nonexistent_device() {
        let mut config = Config::new();
        config.add_device("living_room".to_string(), "192.168.1.100".to_string());

        let result = config.remove_device("kitchen");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Device 'kitchen' not found");
    }

    #[test]
    fn test_remove_last_device() {
        let mut config = Config::new();
        config.add_device("living_room".to_string(), "192.168.1.100".to_string());

        let result = config.remove_device("living_room");
        assert!(result.is_ok());
        assert!(config.devices.is_empty());
        assert!(config.default_device.is_none());
    }

    #[test]
    fn test_set_default() {
        let mut config = Config::new();
        config.add_device("living_room".to_string(), "192.168.1.100".to_string());
        config.add_device("bedroom".to_string(), "192.168.1.101".to_string());

        let result = config.set_default("bedroom");
        assert!(result.is_ok());
        assert_eq!(config.default_device, Some("bedroom".to_string()));
    }

    #[test]
    fn test_set_default_nonexistent_device() {
        let mut config = Config::new();
        config.add_device("living_room".to_string(), "192.168.1.100".to_string());

        let result = config.set_default("kitchen");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Device 'kitchen' not found");
    }

    #[test]
    fn test_get_device_ip_by_name() {
        let mut config = Config::new();
        config.add_device("living_room".to_string(), "192.168.1.100".to_string());

        let result = config.get_device_ip(Some("living_room"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "192.168.1.100");
    }

    #[test]
    fn test_get_device_ip_by_direct_ip() {
        let config = Config::new();

        let result = config.get_device_ip(Some("192.168.1.200"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "192.168.1.200");
    }

    #[test]
    fn test_get_device_ip_default() {
        let mut config = Config::new();
        config.add_device("living_room".to_string(), "192.168.1.100".to_string());

        let result = config.get_device_ip(None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "192.168.1.100");
    }

    #[test]
    fn test_get_device_ip_no_default() {
        let config = Config::new();

        let result = config.get_device_ip(None);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "No device specified and no default device set"
        );
    }

    #[test]
    fn test_save_and_load_config() {
        let config_path = temp_config_path();
        cleanup_config(&config_path);

        // Create and save a config
        let mut config = Config::new();
        config.add_device("living_room".to_string(), "192.168.1.100".to_string());
        config.add_device("bedroom".to_string(), "192.168.1.101".to_string());

        let content = toml::to_string_pretty(&config).unwrap();
        fs::write(&config_path, content).unwrap();

        // Load the config
        let loaded_content = fs::read_to_string(&config_path).unwrap();
        let loaded_config: Config = toml::from_str(&loaded_content).unwrap();

        assert_eq!(loaded_config.devices.len(), 2);
        assert_eq!(
            loaded_config.devices.get("living_room"),
            Some(&"192.168.1.100".to_string())
        );
        assert_eq!(
            loaded_config.devices.get("bedroom"),
            Some(&"192.168.1.101".to_string())
        );
        assert_eq!(
            loaded_config.default_device,
            Some("living_room".to_string())
        );

        cleanup_config(&config_path);
    }

    #[test]
    fn test_config_serialization() {
        let mut config = Config::new();
        config.add_device("test_device".to_string(), "192.168.1.50".to_string());

        let serialized = toml::to_string_pretty(&config).unwrap();
        assert!(serialized.contains("test_device"));
        assert!(serialized.contains("192.168.1.50"));
        assert!(serialized.contains("default_device"));

        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.devices.len(), 1);
        assert_eq!(deserialized.default_device, Some("test_device".to_string()));
    }
}
