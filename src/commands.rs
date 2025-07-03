use std::{path::PathBuf, sync::Arc};

use clap::crate_version;
use handlebars::Handlebars;
use rmcp::{
    Error as McpError, RoleServer, ServerHandler, handler::server::tool::ToolCallContext,
    handler::server::tool::ToolRouter, model::*, service::RequestContext, tool_router,
};
use serde_json::Value as JsonValue;

use crate::manifest::{CommandSpec, Manifest};

#[derive(Clone)]
pub struct Commands {
    tool_router: ToolRouter<Self>,
    cwd: PathBuf,
    manifest: Manifest,
    handlebars: Handlebars<'static>,
}

#[tool_router]
impl Commands {
    pub fn new(cwd: PathBuf, manifest: Manifest) -> Self {
        Self {
            tool_router: Self::tool_router(),
            cwd,
            manifest,
            handlebars: Handlebars::new(),
        }
    }

    async fn execute(
        &self,
        spec: &CommandSpec,
        args: &JsonValue,
    ) -> Result<CallToolResult, McpError> {
        spec.validate(args)
            .map_err(|e| McpError::invalid_params(format!("Invalid argument: {}", e), None))?;

        let (command, output, exit_code) = spec
            .execute(&self.handlebars, args, &self.cwd)
            .map_err(|e| {
                McpError::invalid_params(format!("Command execution error: {}", e), None)
            })?;

        let mut response = String::new();

        response.push_str(&format!("## Command\n\n```\n{}\n```\n\n", command.trim()));

        if !output.is_empty() {
            response.push_str(&format!("## Output\n\n```\n{}\n```\n\n", output.trim()));
        } else {
            response.push_str("## Output\n\nNo output.\n\n");
        }

        response.push_str(&format!("Command exited with code: {}", exit_code));

        Ok(CallToolResult::success(vec![Content::text(response)]))
    }
}

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

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let mut tools = self.tool_router.list_all();

        tools.extend(self.manifest.commands.iter().map(|(name, spec)| Tool {
            name: name.clone().into(),
            description: Some(spec.description.clone().into()),
            input_schema: Arc::new(spec.to_schema()),
            annotations: None,
        }));

        Ok(ListToolsResult::with_all_items(tools))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        if self.tool_router.has_route(request.name.as_ref()) {
            let tcc = ToolCallContext::new(self, request, context);
            return self.tool_router.call(tcc).await;
        }

        if let Some(spec) = self.manifest.commands.get(request.name.as_ref()) {
            let args = match &request.arguments {
                Some(args) => JsonValue::Object(args.clone()),
                None => JsonValue::Object(serde_json::Map::new()),
            };
            return self.execute(spec, &args).await;
        }

        Err(McpError::invalid_params(
            format!("Tool '{}' not found", request.name),
            None,
        ))
    }
}
