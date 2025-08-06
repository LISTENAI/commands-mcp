use std::fmt::Debug;

use rmcp::{
    Error as McpError,
    handler::server::tool::Parameters,
    model::{CallToolResult, Content},
    schemars, tool, tool_router,
};
use serde::Deserialize;

use crate::{commands::Commands, inspector::Inspector};

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct InspectorTakeScreenshotRequest {
    #[schemars(description = "ID of the device to take a screenshot from")]
    pub device: String,
}

#[tool_router(router =inspector_router, vis = "pub")]
impl Commands {
    #[tool(
        name = "inspector_list_devices",
        description = "List available devices for inspection",
        annotations(read_only_hint = true)
    )]
    async fn inspector_list_devices(&self) -> Result<CallToolResult, McpError> {
        let inspector = Inspector::new();
        let devices = inspector.list_devices()?;
        Ok(CallToolResult::success(vec![Content::text(
            devices
                .iter()
                .map(|d| format!("* ID: {}, Name: \"{}\"", d.id, d.name))
                .collect::<Vec<_>>()
                .join("\n"),
        )]))
    }

    #[tool(
        name = "inspector_take_screenshot",
        description = "Take a screenshot from a specified device",
        annotations(read_only_hint = true)
    )]
    async fn inspector_take_screenshot(
        &self,
        Parameters(InspectorTakeScreenshotRequest { device }): Parameters<
            InspectorTakeScreenshotRequest,
        >,
    ) -> Result<CallToolResult, McpError> {
        let inspector = Inspector::new();
        let screenshot = inspector.take_screenshot(&device)?;

        Ok(CallToolResult::success(vec![Content::text("")]))
    }
}
