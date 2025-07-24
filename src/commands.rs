use std::{path::PathBuf, sync::Arc};

use clap::crate_version;
use handlebars::Handlebars;
use rmcp::{
    Error as McpError, ServerHandler,
    handler::server::tool::{ToolCallContext, ToolRoute, ToolRouter},
    model::*,
    tool_handler,
};
use serde_json::{Map, Value as JsonValue};

use crate::manifest::{CommandSpec, Manifest};

#[derive(Clone)]
pub struct Commands {
    tool_router: ToolRouter<Self>,
    pub cwd: PathBuf,
    pub manifest: Manifest,
    handlebars: Handlebars<'static>,
}

impl Commands {
    pub fn new(cwd: PathBuf, manifest: Manifest) -> Self {
        let mut tool_router = ToolRouter::<Self>::new();

        for (name, spec) in manifest.commands.iter() {
            tool_router.add_route(spec.to_tool_route(name));
        }

        if let Some(opts) = &manifest.flash
            && opts.enabled
        {
            tool_router.merge(Self::flash_router());
        }

        if let Some(opts) = &manifest.serial
            && opts.enabled
        {
            tool_router.merge(Self::serial_router());
        }

        if let Some(opts) = &manifest.schematic
            && opts.enabled
        {
            tool_router.merge(Self::schematic_router());
        }

        Self {
            tool_router,
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

        let (_command, output, exit_code) = spec
            .execute(&self.handlebars, args, &self.cwd)
            .map_err(|e| {
                McpError::invalid_params(format!("Command execution error: {}", e), None)
            })?;

        let mut response = String::new();

        if !output.is_empty() {
            response.push_str(&format!("## Output\n\n```\n{}\n```\n\n", output.trim()));
        } else {
            response.push_str("## Output\n\nNo output.\n\n");
        }

        response.push_str(&format!("Command exited with code: {}", exit_code));

        Ok(CallToolResult::success(vec![Content::text(response)]))
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

impl CommandSpec {
    pub fn to_tool_route(&self, name: &String) -> ToolRoute<Commands> {
        ToolRoute::<Commands> {
            attr: Tool {
                name: name.to_string().into(),
                description: Some(self.description.as_str().to_string().into()),
                input_schema: Arc::new(self.to_schema()),
                annotations: None,
            },
            call: Arc::new(|tcc: ToolCallContext<'_, Commands>| {
                Box::pin(async move {
                    let name = tcc.name.as_ref();

                    let spec = tcc.service.manifest.commands.get(name).ok_or_else(|| {
                        McpError::invalid_params(format!("Command '{}' not found", name), None)
                    })?;

                    let args = match tcc.arguments {
                        Some(args) => JsonValue::Object(args),
                        None => JsonValue::Object(Map::new()),
                    };

                    tcc.service.execute(spec, &args).await
                })
            }),
        }
    }
}
