use std::{
    io::{Read, pipe},
    path::PathBuf,
    process::Command,
};

use handlebars::Handlebars;
use rmcp::Error as McpError;
use serde_json::Value as JsonValue;

use crate::manifest::CommandSpec;

impl CommandSpec {
    pub fn execute(
        &self,
        handlebars: &Handlebars,
        args: &JsonValue,
        cwd: &PathBuf,
    ) -> Result<(String, String, i32), McpError> {
        let command = handlebars
            .render_template(&self.command, args)
            .map_err(|e| {
                McpError::invalid_params(format!("Template rendering error: {}", e), None)
            })?;

        let (mut reader, writer) = pipe().map_err(|e| {
            McpError::internal_error(format!("Failed creating stdio pipes: {}", e), None)
        })?;

        let mut proc = Command::new("bash")
            .current_dir(cwd)
            .arg("-c")
            .arg(&command)
            .stdout(writer.try_clone().map_err(|e| {
                McpError::internal_error(
                    format!("Failed creating stdio pipes for child process: {}", e),
                    None,
                )
            })?)
            .stderr(writer)
            .spawn()
            .map_err(|e| {
                McpError::internal_error(format!("Failed executing command: {}", e), None)
            })?;

        let mut output = String::new();
        reader.read_to_string(&mut output).map_err(|e| {
            McpError::internal_error(format!("Failed reading command output: {}", e), None)
        })?;

        let status = proc.wait().map_err(|e| {
            McpError::internal_error(format!("Failed reading result for command: {}", e), None)
        })?;

        let exit_code = status.code().unwrap_or(1);

        Ok((command, output, exit_code))
    }
}
