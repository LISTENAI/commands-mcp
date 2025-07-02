use clap::crate_version;
use rmcp::{
    Error as McpError, ServerHandler, handler::server::tool::ToolRouter, model::*, tool,
    tool_handler, tool_router,
};

#[derive(Clone)]
pub struct Commands {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl Commands {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Hello, world!")]
    async fn hello_world(&self) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text("Hello world!")]))
    }
}

#[tool_handler]
impl ServerHandler for Commands {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "commands".to_string(),
                version: crate_version!().to_string(),
            },
            instructions: None,
        }
    }
}
