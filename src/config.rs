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
            return Err(format!("Device '{}' not found", name));
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
            return Err(format!("Device '{}' not found", name));
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
