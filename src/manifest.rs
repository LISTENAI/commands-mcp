use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Manifest {
    /// A collection of commands
    pub commands: HashMap<String, CommandSpec>,

    /// Flash options for the manifest
    pub flash: Option<FlashOptions>,

    /// Serial options for the manifest
    pub serial: Option<SerialOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CommandSpec {
    /// A brief description of the command
    pub description: String,

    /// The arguments for the command
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<ArgumentSpec>>,

    /// The shell used to execute the command. Defaults to "bash" on Unix-like
    /// systems and "powershell" on Windows. Also supports "python" for using
    /// Python script in the command.
    pub shell: Option<String>,

    /// The command template
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ArgumentSpec {
    /// The name of the argument
    pub name: String,

    /// A brief description of the argument
    pub description: String,

    /// The type of the argument (e.g., string, number, boolean)
    #[serde(rename = "type")]
    pub arg_type: Option<ArgumentType>,

    /// Whether the argument is required
    #[serde(default)]
    pub required: bool,

    /// Default value for the argument
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ArgumentType {
    /// String argument
    #[serde(rename = "string")]
    String,

    /// Number argument
    #[serde(rename = "number")]
    Number,

    /// Boolean argument
    #[serde(rename = "boolean")]
    Boolean,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FlashOptions {
    /// Whether flash tools are enabled
    pub enabled: bool,

    /// The chip model to flash
    pub chip: String,

    /// The baud rate for the flash operation
    pub baudrate: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum SerialResetMethod {
    /// Reset the device by asserting and deasserting DTR
    #[serde(rename = "dtr")]
    DTR,

    /// Reset the device by asserting and deasserting RTS
    #[serde(rename = "rts")]
    RTS,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SerialOptions {
    /// Whether serial tools are enabled
    pub enabled: bool,

    /// The baud rate for serial communication, default is 115200
    pub baudrate: Option<u32>,

    /// Reset method. If specified, the tool will reset the device before each read
    pub reset: Option<SerialResetMethod>,

    /// Optional interval of milliseconds between the reset line is asserted and
    /// deasserted. If not specified, the default is 100ms.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reset_interval: Option<u64>,
}
