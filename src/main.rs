mod config;

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
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { name, ip } => {
            let mut config = Config::load()?;
            config.add_device(name.clone(), ip.clone());
            config.save()?;
            println!("Added device '{}' with IP {}", name, ip);

            if config.devices.len() == 1 {
                println!("Set '{}' as the default device", name);
            }
        }
        Commands::Delete { name } => {
            let mut config = Config::load()?;
            config.remove_device(&name)?;
            config.save()?;
            println!("Deleted device '{}'", name);
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
                println!("  {} - {}{}", name, ip, default_marker);
            }
        }
        Commands::SetDefault { name } => {
            let mut config = Config::load()?;
            config.set_default(&name)?;
            config.save()?;
            println!("Set '{}' as the default device", name);
        }
        Commands::On { device } => {
            let config = Config::load()?;
            let ip = config.get_device_ip(device.as_deref())?;

            let url = reqwest::Url::parse(&format!("http://{}", ip))?;
            let mut wled = Wled::try_from_url(&url)?;

            // Get current state
            wled.get_state_from_wled()?;

            // Update state to turn on
            if let Some(state) = &mut wled.state {
                state.on = Some(true);
            } else {
                wled.state = Some(State {
                    on: Some(true),
                    ..Default::default()
                });
            }

            // Send updated state
            wled.flush_state()?;

            println!("Turned on device at {}", ip);
        }
        Commands::Off { device } => {
            let config = Config::load()?;
            let ip = config.get_device_ip(device.as_deref())?;

            let url = reqwest::Url::parse(&format!("http://{}", ip))?;
            let mut wled = Wled::try_from_url(&url)?;

            // Get current state
            wled.get_state_from_wled()?;

            // Update state to turn off
            if let Some(state) = &mut wled.state {
                state.on = Some(false);
            } else {
                wled.state = Some(State {
                    on: Some(false),
                    ..Default::default()
                });
            }

            // Send updated state
            wled.flush_state()?;

            println!("Turned off device at {}", ip);
        }
    }

    Ok(())
}
