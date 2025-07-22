use std::{
    env::{join_paths, split_paths, var_os},
    io::{Read, pipe},
    path::PathBuf,
    process::Command,
    str::FromStr,
};

use handlebars::Handlebars;
use rmcp::Error as McpError;
use serde_json::Value as JsonValue;

use crate::manifest::{CommandSpec, VirtualEnv};

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
            .to_command(&command)
            .current_dir(cwd)
            .envs(self.venv.to_envs(cwd)?)
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

macro_rules! command {
    ($program:expr $(, $arg:expr)* $(,)?) => {{
        let mut cmd = std::process::Command::new($program);
        $(
            cmd.arg($arg);
        )*
        cmd
    }};
}

impl Shell {
    pub fn to_command(&self, command: &str) -> Command {
        match self {
            Shell::Bash => command!("bash", "-c", normalize_newlines(command, false)),
            Shell::PowerShell => command!(
                "powershell",
                "-Command",
                format!(
                    "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8\r\n{}",
                    normalize_newlines(command, true)
                )
            ),
            Shell::Python => {
                if cfg!(windows) {
                    command!("python", "-c", normalize_newlines(command, false))
                } else {
                    command!(
                        "/usr/bin/env",
                        "-S",
                        "python3",
                        "-c",
                        normalize_newlines(command, false)
                    )
                }
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

impl VirtualEnv {
    pub fn to_envs(&self, cwd: &PathBuf) -> Result<Vec<(String, String)>, McpError> {
        match self {
            VirtualEnv::UseDefault(false) => Ok(vec![]),
            VirtualEnv::UseDefault(true) => Self::envs(cwd.join(".venv")),
            VirtualEnv::Path(path) => Self::envs(cwd.join(path)),
        }
    }

    fn envs(venv: PathBuf) -> Result<Vec<(String, String)>, McpError> {
        let bin_dir = venv.join(if cfg!(windows) { "Scripts" } else { "bin" });

        let path = match var_os("PATH") {
            Some(path) => {
                let mut paths = split_paths(&path).collect::<Vec<_>>();
                paths.insert(0, bin_dir);
                join_paths(paths).map_err(|e| {
                    McpError::internal_error(format!("Failed joining paths: {}", e), None)
                })?
            }
            None => bin_dir.into_os_string(),
        };

        Ok(vec![
            ("PATH".into(), path.to_string_lossy().into()),
            ("VIRTUAL_ENV".into(), venv.to_string_lossy().into()),
        ])
    }
}
