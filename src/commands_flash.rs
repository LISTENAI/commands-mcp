use std::{str::FromStr, time::Duration};

use cskburn::{CSKBurn, Family, Image, ProbeTarget, WriteTarget};
use rmcp::{
    Error as McpError,
    handler::server::tool::Parameters,
    model::{CallToolResult, Content},
    schemars, tool, tool_router,
};
use serde::Deserialize;

use crate::commands::Commands;

const PROBE_RESET_ATTEMPTS: usize = 5;
const PROBE_SYNC_ATTEMPTS: usize = 3;
const RESET_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FlashWriteRequest {
    #[schemars(description = "The port to use for flashing the device")]
    pub port: String,

    #[schemars(description = "The path to the binary file to flash")]
    pub path: String,
}

#[tool_router(router = flash_router, vis = "pub")]
impl Commands {
    #[tool(
        name = "flash_list_ports",
        description = "List available ports for flashing devices",
        annotations(read_only_hint = true)
    )]
    async fn flash_list_ports(&self) -> Result<CallToolResult, McpError> {
        let port = cskburn::list_ports()
            .map_err(|e| McpError::internal_error(format!("Failed to list ports: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            port.join("\n"),
        )]))
    }

    #[tool(
        name = "flash_write",
        description = "Write target binary to device via specified port"
    )]
    async fn flash_write(
        &self,
        Parameters(FlashWriteRequest { port, path }): Parameters<FlashWriteRequest>,
    ) -> Result<CallToolResult, McpError> {
        let flash_opts = self
            .manifest
            .flash
            .as_ref()
            .ok_or(McpError::invalid_params(
                "Flash options are not defined in the manifest".to_string(),
                None,
            ))?;

        let chip =
            Family::from_str(&flash_opts.chip).map_err(|e| McpError::invalid_params(e, None))?;

        let mut burner = chip.burner();

        let mut cskburn = CSKBurn::connect(&port, flash_opts.baudrate, chip)
            .map_err(|e| McpError::internal_error(format!("Failed to open device: {}", e), None))?;

        let mut probed = false;
        for _ in 0..PROBE_RESET_ATTEMPTS {
            cskburn.reset(true, Some(RESET_INTERVAL)).map_err(|e| {
                McpError::internal_error(format!("Failed to reset device: {}", e), None)
            })?;

            if cskburn
                .probe(ProbeTarget::ROM, Some(PROBE_SYNC_ATTEMPTS))
                .is_ok()
            {
                probed = true;
                break;
            }
        }

        if !probed {
            return Err(McpError::internal_error(
                "Failed to probe device after multiple attempts".to_string(),
                None,
            ));
        }

        cskburn
            .write(&mut burner, WriteTarget::Memory { action: None })
            .map_err(|e| {
                McpError::internal_error(format!("Failed to write burner: {}", e), None)
            })?;

        cskburn
            .probe(ProbeTarget::Burner, Some(PROBE_SYNC_ATTEMPTS))
            .map_err(|e| McpError::internal_error(format!("Failed to boot burner: {}", e), None))?;

        let path = self.cwd.join(&path);
        let path = path.to_str().ok_or(McpError::invalid_params(
            "Invalid path for binary file".to_string(),
            None,
        ))?;

        let mut source = Image::try_from_file(0, path)
            .map_err(|e| McpError::invalid_params(format!("Failed to read image: {}", e), None))?;

        cskburn
            .write(&mut source, WriteTarget::Flash)
            .map_err(|e| McpError::internal_error(format!("Failed to write image: {}", e), None))?;

        cskburn.reset(false, Some(RESET_INTERVAL)).map_err(|e| {
            McpError::internal_error(
                format!("Failed to reset device after flashing: {}", e),
                None,
            )
        })?;

        let mut response = String::new();
        response.push_str("## Device Info\n\n");
        response.push_str(format!("* Port: {}\n", port).as_str());
        response.push_str(format!("* Baud rate: {}\n", flash_opts.baudrate).as_str());
        response.push_str(format!("* Chip model: {}\n", flash_opts.chip).as_str());
        response.push_str("\n");
        response.push_str("## Flashing Summary\n\n");
        response.push_str(format!("* Image to flash: {}\n", path).as_str());
        response.push_str(format!("* Offset: 0x{:08x}\n", source.addr).as_str());
        response.push_str(format!("* Size: {}\n", source.size().unwrap_or(0)).as_str());
        response.push_str("\n");
        response.push_str("## Operation Status\n\n");
        response.push_str("Operation completed successfully.\n");

        Ok(CallToolResult::success(vec![Content::text(response)]))
    }
}
