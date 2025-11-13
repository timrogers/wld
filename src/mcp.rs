use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    schemars, tool, tool_handler, tool_router,
    transport::stdio,
    ErrorData as McpError, ServerHandler, ServiceExt,
};

use crate::config::Config;
use crate::{get_device_status, set_device_brightness, set_device_power, DeviceStatus};

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct EmptyParams {}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct WledDeviceParams {
    /// Device name or IP address (optional - if not specified, the default device is used)
    pub device: Option<String>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct WledBrightnessParams {
    /// Brightness level (0-255)
    pub value: u8,
    /// Device name or IP address (optional - if not specified, the default device is used)
    pub device: Option<String>,
}

#[derive(Clone)]
pub struct WledMcpServer {
    tool_router: ToolRouter<WledMcpServer>,
}

#[tool_router]
impl WledMcpServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "List saved WLED devices from configuration")]
    async fn wled_devices(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        match Config::load() {
            Ok(config) => {
                if config.devices.is_empty() {
                    return Ok(CallToolResult::success(vec![Content::text(
                        "No devices saved",
                    )]));
                }

                let mut output = String::from("Saved devices:\n");
                for (name, ip) in &config.devices {
                    let default_marker = if config.default_device.as_ref() == Some(name) {
                        " (default)"
                    } else {
                        ""
                    };
                    output.push_str(&format!("  {name} - {ip}{default_marker}\n"));
                }
                Ok(CallToolResult::success(vec![Content::text(output)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to load configuration: {e}"
            ))])),
        }
    }

    #[tool(
        description = "Turn WLED device on. By default, the default device is used, but you can optionally specify a device name or IP address."
    )]
    async fn wled_on(
        &self,
        Parameters(params): Parameters<WledDeviceParams>,
    ) -> Result<CallToolResult, McpError> {
        let device = params.device.clone();
        match tokio::task::spawn_blocking(move || {
            set_device_power(device.as_deref(), true).map_err(|e| e.to_string())
        })
        .await
        {
            Ok(Ok(())) => Ok(CallToolResult::success(vec![Content::text(
                "Device turned on successfully",
            )])),
            Ok(Err(e)) => Ok(CallToolResult::error(vec![Content::text(e)])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Task error: {e}"
            ))])),
        }
    }

    #[tool(
        description = "Turn WLED device off. By default, the default device is used, but you can optionally specify a device name or IP address."
    )]
    async fn wled_off(
        &self,
        Parameters(params): Parameters<WledDeviceParams>,
    ) -> Result<CallToolResult, McpError> {
        let device = params.device.clone();
        match tokio::task::spawn_blocking(move || {
            set_device_power(device.as_deref(), false).map_err(|e| e.to_string())
        })
        .await
        {
            Ok(Ok(())) => Ok(CallToolResult::success(vec![Content::text(
                "Device turned off successfully",
            )])),
            Ok(Err(e)) => Ok(CallToolResult::error(vec![Content::text(e)])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Task error: {e}"
            ))])),
        }
    }

    #[tool(
        description = "Set WLED device brightness (0-255). By default, the default device is used, but you can optionally specify a device name or IP address."
    )]
    async fn wled_brightness(
        &self,
        Parameters(params): Parameters<WledBrightnessParams>,
    ) -> Result<CallToolResult, McpError> {
        let device = params.device.clone();
        let value = params.value;
        match tokio::task::spawn_blocking(move || {
            set_device_brightness(device.as_deref(), value).map_err(|e| e.to_string())
        })
        .await
        {
            Ok(Ok(())) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Device brightness set to {value} successfully"
            ))])),
            Ok(Err(e)) => Ok(CallToolResult::error(vec![Content::text(e)])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Task error: {e}"
            ))])),
        }
    }

    #[tool(description = "Check status of all configured WLED devices")]
    async fn wled_status(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        match tokio::task::spawn_blocking(|| -> Result<String, String> {
            let config = Config::load().map_err(|e| e.to_string())?;

            if config.devices.is_empty() {
                return Ok("No devices saved".to_string());
            }

            let mut output = String::from("Checking status of all devices:\n\n");
            let mut all_reachable = true;

            for (name, ip) in &config.devices {
                let default_marker = if config.default_device.as_ref() == Some(name) {
                    " (default)"
                } else {
                    ""
                };

                output.push_str(&format!("  {name} ({ip}){default_marker}: "));

                match get_device_status(ip) {
                    DeviceStatus::On => {
                        output.push_str("ON\n");
                    }
                    DeviceStatus::Off => {
                        output.push_str("OFF\n");
                    }
                    DeviceStatus::Unreachable => {
                        output.push_str("UNREACHABLE\n");
                        all_reachable = false;
                    }
                }
            }

            if !all_reachable {
                output.push_str("\nWarning: Some devices are unreachable");
            }

            Ok(output)
        })
        .await
        {
            Ok(Ok(output)) => Ok(CallToolResult::success(vec![Content::text(output)])),
            Ok(Err(e)) => Ok(CallToolResult::error(vec![Content::text(e)])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Task error: {e}"
            ))])),
        }
    }
}

#[tool_handler]
impl ServerHandler for WledMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_03_26,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: None,
        }
    }
}

pub fn handle_mcp_command() -> Result<(), Box<dyn std::error::Error>> {
    // Set up tracing for the MCP server
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::DEBUG.into()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    // Create the MCP server
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        tracing::info!("Starting WLED MCP server");

        let service = WledMcpServer::new().serve(stdio()).await?;
        service.waiting().await?;
        Ok(())
    })
}
