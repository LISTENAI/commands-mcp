use std::{
    fmt::Debug,
    io,
    thread::sleep,
    time::{Duration, Instant},
};

use rmcp::{
    Error as McpError,
    handler::server::tool::Parameters,
    model::{CallToolResult, Content},
    schemars, tool, tool_router,
};
use serde::Deserialize;
use serialport::{SerialPort, available_ports};

use crate::{
    commands::Commands,
    manifest::{SerialOptions, SerialResetMethod},
};

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SerialResetRequest {
    #[schemars(description = "The port to use")]
    pub port: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SerialReadRequest {
    #[schemars(description = "The port to use")]
    pub port: String,

    #[schemars(description = "Duration in milliseconds to read, defaults to 10000")]
    pub timeout: Option<u32>,
}

#[tool_router(router = serial_router, vis = "pub")]
impl Commands {
    #[tool(
        name = "serial_list_ports",
        description = "List available serial port",
        annotations(read_only_hint = true)
    )]
    async fn serial_list_ports(&self) -> Result<CallToolResult, McpError> {
        let ports = available_ports()
            .map_err(|e| McpError::internal_error(format!("Failed to list ports: {}", e), None))
            .map(|ports| {
                ports
                    .into_iter()
                    .filter_map(|port| {
                        if cfg!(target_os = "macos") && !port.port_name.starts_with("/dev/cu.") {
                            return None;
                        }

                        if let serialport::SerialPortType::UsbPort(_) = port.port_type {
                            Some(port.port_name)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })?;

        Ok(CallToolResult::success(vec![Content::text(
            ports.join("\n"),
        )]))
    }

    #[tool(
        name = "serial_reset",
        description = "Reset the connected device via specified serial port"
    )]
    async fn serial_reset(
        &self,
        Parameters(SerialResetRequest { port }): Parameters<SerialResetRequest>,
    ) -> Result<rmcp::model::CallToolResult, McpError> {
        let serial_opts = self
            .manifest
            .serial
            .as_ref()
            .ok_or(McpError::invalid_params(
                "Serial options are not defined in the manifest".to_string(),
                None,
            ))?;

        let mut device = serial_open(&port, serial_opts)
            .map_err(|e| McpError::internal_error(format!("Failed to open device: {}", e), None))?;

        serial_reset(&mut device, serial_opts).map_err(|e| {
            McpError::internal_error(format!("Failed to reset device: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(
            "Device reset command executed successfully.".to_string(),
        )]))
    }

    #[tool(
        name = "serial_read",
        description = "Read data from the connected device via specified serial port"
    )]
    async fn serial_read(
        &self,
        Parameters(SerialReadRequest { port, timeout }): Parameters<SerialReadRequest>,
    ) -> Result<rmcp::model::CallToolResult, McpError> {
        let serial_opts = self
            .manifest
            .serial
            .as_ref()
            .ok_or(McpError::invalid_params(
                "Serial options are not defined in the manifest".to_string(),
                None,
            ))?;

        let mut device = serial_open(&port, serial_opts)
            .map_err(|e| McpError::internal_error(format!("Failed to open device: {}", e), None))?;

        serial_reset(&mut device, serial_opts).map_err(|e| {
            McpError::internal_error(format!("Failed to reset device: {}", e), None)
        })?;

        device
            .set_timeout(Duration::from_millis(100))
            .map_err(|e| McpError::internal_error(format!("Failed to set timeout: {}", e), None))?;

        let start_time = Instant::now();
        let timeout = timeout.map_or(Duration::from_secs(10), |t| Duration::from_millis(t as u64));

        let mut lines = Vec::<(Instant, String)>::new();
        let mut last_read = String::new();

        while Instant::now().duration_since(start_time) < timeout {
            let mut buffer = vec![0; 1024];
            match device.read(&mut buffer) {
                Ok(bytes) if bytes > 0 => {
                    let content = String::from_utf8_lossy(&buffer[..bytes.min(buffer.len())]);
                    last_read.push_str(&content);

                    while let Some(newline_pos) = last_read.find('\n') {
                        let line = last_read[..newline_pos].trim_end_matches('\r').to_string();
                        lines.push((Instant::now(), line));
                        last_read = last_read[newline_pos + 1..].to_string();
                    }
                }
                Ok(_) => continue, // No data read, continue
                Err(e) if e.kind() == io::ErrorKind::TimedOut => continue,
                Err(e) => {
                    return Err(McpError::internal_error(
                        format!("Failed to read from device: {}", e),
                        None,
                    ));
                }
            }
        }
        if !last_read.is_empty() {
            let final_line = last_read
                .trim_end_matches('\r')
                .trim_end_matches('\n')
                .to_string();
            lines.push((Instant::now(), final_line));
        }

        let mut response = String::new();
        if lines.is_empty() {
            response.push_str("No data read from the device.\n");
        } else {
            response.push_str("Logs read from the device, formatted as `[seconds]: message`:\n\n");
            response.push_str("```\n");
            for (timestamp, line) in lines {
                let elapsed = timestamp.duration_since(start_time);
                response.push_str(&format!("[{:.3}]: {}\n", elapsed.as_secs_f64(), line));
            }
            response.push_str("```");
        }

        Ok(CallToolResult::success(vec![Content::text(response)]))
    }
}

fn serial_open(path: &str, opts: &SerialOptions) -> serialport::Result<Box<dyn SerialPort>> {
    let baud_rate = opts.baudrate.unwrap_or(115200);

    serialport::new(path, baud_rate)
        .flow_control(serialport::FlowControl::None)
        .dtr_on_open(false)
        .open()
}

fn serial_reset(port: &mut Box<dyn SerialPort>, opts: &SerialOptions) -> serialport::Result<()> {
    let interval = Duration::from_millis(opts.reset_interval.unwrap_or(100));

    match opts.reset {
        Some(SerialResetMethod::DTR) => {
            port.write_data_terminal_ready(true)?;
            sleep(interval);
            port.write_data_terminal_ready(false)?;
            sleep(interval);
        }
        Some(SerialResetMethod::RTS) => {
            port.write_request_to_send(true)?;
            sleep(interval);
            port.write_request_to_send(false)?;
            sleep(interval);
        }
        None => (), // no-op
    }

    Ok(())
}
