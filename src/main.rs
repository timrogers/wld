mod config;

#[cfg(feature = "mcp")]
mod mcp;

use clap::{Parser, Subcommand};
use config::Config;
use wled_json_api_library::structures::state::State;
use wled_json_api_library::wled::Wled;

#[derive(Parser)]
#[command(name = "wld")]
#[command(about = "Control WLED lights from your terminal", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new WLED device
    Add {
        /// Name for the device
        name: String,
        /// IP address of the device
        ip: String,
    },
    /// Delete a saved device
    Delete {
        /// Name of the device to delete
        name: String,
    },
    /// List all saved devices
    Ls,
    /// Set the default device
    SetDefault {
        /// Name of the device to set as default
        name: String,
    },
    /// Turn device on
    On {
        /// Device name or IP (uses default if not specified)
        #[arg(short, long)]
        device: Option<String>,
    },
    /// Turn device off
    Off {
        /// Device name or IP (uses default if not specified)
        #[arg(short, long)]
        device: Option<String>,
    },
    /// Start a MCP (Model Context Protocol) server for controlling WLED devices
    #[cfg(feature = "mcp")]
    Mcp,
    /// Set device brightness (0-255)
    Brightness {
        /// Brightness level (0-255)
        value: u8,
        /// Device name or IP (uses default if not specified)
        #[arg(short, long)]
        device: Option<String>,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

pub fn set_device_power(
    device: Option<&str>,
    power_state: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let ip = config.get_device_ip(device)?;

    let url = reqwest::Url::parse(&format!("http://{ip}"))?;
    let mut wled = Wled::try_from_url(&url)?;

    // Get current state
    wled.get_state_from_wled()?;

    // Update state
    if let Some(state) = &mut wled.state {
        state.on = Some(power_state);
    } else {
        wled.state = Some(State {
            on: Some(power_state),
            ..Default::default()
        });
    }

    // Send updated state
    wled.flush_state()?;

    let action = if power_state { "on" } else { "off" };
    println!("Turned {action} device at {ip}");

    Ok(())
}

fn set_device_brightness(
    device: Option<&str>,
    brightness: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let ip = config.get_device_ip(device)?;

    let url = reqwest::Url::parse(&format!("http://{}", ip))?;
    let mut wled = Wled::try_from_url(&url)?;

    // Get current state
    wled.get_state_from_wled()?;

    // Update state
    if let Some(state) = &mut wled.state {
        state.bri = Some(brightness);
    } else {
        wled.state = Some(State {
            bri: Some(brightness),
            ..Default::default()
        });
    }

    // Send updated state
    wled.flush_state()?;

    println!("Set brightness to {} for device at {}", brightness, ip);

    Ok(())
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { name, ip } => {
            let mut config = Config::load()?;
            config.add_device(name.clone(), ip.clone());
            config.save()?;
            println!("Added device '{name}' with IP {ip}");

            if config.devices.len() == 1 {
                println!("Set '{name}' as the default device");
            }
        }
        Commands::Delete { name } => {
            let mut config = Config::load()?;
            config.remove_device(&name)?;
            config.save()?;
            println!("Deleted device '{name}'");
        }
        Commands::Ls => {
            let config = Config::load()?;

            if config.devices.is_empty() {
                println!("No devices saved");
                return Ok(());
            }

            println!("Saved devices:");
            for (name, ip) in &config.devices {
                let default_marker = if config.default_device.as_ref() == Some(name) {
                    " (default)"
                } else {
                    ""
                };
                println!("  {name} - {ip}{default_marker}");
            }
        }
        Commands::SetDefault { name } => {
            let mut config = Config::load()?;
            config.set_default(&name)?;
            config.save()?;
            println!("Set '{name}' as the default device");
        }
        Commands::On { device } => {
            set_device_power(device.as_deref(), true)?;
        }
        Commands::Off { device } => {
            set_device_power(device.as_deref(), false)?;
        }
        #[cfg(feature = "mcp")]
        Commands::Mcp => {
            mcp::handle_mcp_command()?;
        }
        Commands::Brightness { value, device } => {
            set_device_brightness(device.as_deref(), value)?;
        }
    }

    Ok(())
}
