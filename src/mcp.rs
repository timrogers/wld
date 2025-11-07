use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    schemars, tool, tool_handler, tool_router,
    transport::stdio,
    ErrorData as McpError, ServerHandler, ServiceExt,
};

use crate::config::Config;
use crate::set_device_power;

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct WledDeviceParams {
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
    async fn wled_devices(&self) -> Result<CallToolResult, McpError> {
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
                    output.push_str(&format!("  {} - {}{}\n", name, ip, default_marker));
                }
                Ok(CallToolResult::success(vec![Content::text(output)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to load configuration: {}",
                e
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
        match set_device_power(params.device.as_deref(), true) {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(
                "Device turned on successfully",
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    #[tool(
        description = "Turn WLED device off. By default, the default device is used, but you can optionally specify a device name or IP address."
    )]
    async fn wled_off(
        &self,
        Parameters(params): Parameters<WledDeviceParams>,
    ) -> Result<CallToolResult, McpError> {
        match set_device_power(params.device.as_deref(), false) {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(
                "Device turned off successfully",
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
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
