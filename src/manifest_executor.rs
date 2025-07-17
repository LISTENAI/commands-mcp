use std::{
    io::{Read, pipe},
    path::PathBuf,
    process::Command,
    str::FromStr,
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

        let shell = match self.shell {
            None => Shell::default(),
            Some(ref s) => Shell::from_str(s)?,
        };

        let mut proc = shell
            .to_command(&self, cwd)
            .current_dir(cwd)
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

enum Shell {
    Bash,
    PowerShell,
    Python,
}

impl Default for Shell {
    fn default() -> Self {
        if cfg!(windows) {
            Shell::PowerShell
        } else {
            Shell::Bash
        }
    }
}

impl FromStr for Shell {
    type Err = McpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bash" => Ok(Shell::Bash),
            "powershell" => Ok(Shell::PowerShell),
            "python" => Ok(Shell::Python),
            _ => Err(McpError::invalid_params(
                format!("Unsupported shell: {}", s),
                None,
            )),
        }
    }
}

impl Shell {
    pub fn to_command(&self, spec: &CommandSpec, cwd: &PathBuf) -> Command {
        match self {
            Shell::Bash => {
                let mut cmd = Command::new("bash");
                cmd.arg("-c").arg(normalize_newlines(&spec.command, false));
                cmd
            }
            Shell::PowerShell => {
                let mut cmd = Command::new("powershell");
                cmd.arg("-Command").arg(format!(
                    "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8\r\n{}",
                    normalize_newlines(&spec.command, true)
                ));
                cmd
            }
            Shell::Python if spec.venv.is_some() => {
                let venv = cwd.join(spec.venv.as_ref().unwrap());
                let python = if cfg!(windows) {
                    venv.join("Scripts/python.exe")
                } else {
                    venv.join("bin/python")
                };
                let mut cmd = Command::new(python);
                cmd.arg("-c").arg(normalize_newlines(&spec.command, false));
                cmd.env("VIRTUAL_ENV", venv);
                cmd
            }
            Shell::Python => {
                let mut cmd = Command::new("python");
                cmd.arg("-c").arg(normalize_newlines(&spec.command, false));
                cmd
            }
        }
    }
}

fn normalize_newlines(command: &str, wants_cr_lf: bool) -> String {
    let command = command.replace("\r\n", "\n");
    if wants_cr_lf {
        command.replace("\n", "\r\n")
    } else {
        command
    }
}
