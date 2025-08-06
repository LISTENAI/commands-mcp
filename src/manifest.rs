use std::{collections::BTreeMap, path::PathBuf};

use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[allow(unused)]
pub struct Manifest {
    /// A collection of commands
    pub commands: BTreeMap<String, CommandSpec>,

    /// Flash options for the manifest
    pub flash: Option<FlashOptions>,

    /// Serial options for the manifest
    pub serial: Option<SerialOptions>,

    /// Schematic options for the manifest
    pub schematic: Option<SchematicOptions>,

    /// Inspector options for the manifest
    pub inspector: Option<InspectorOptions>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[allow(unused)]
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

    /// Whether to activate a Python Virtual Environment (venv) when executing
    /// the command. If set to `true`, the venv will be activated from the path
    /// `.venv`. Can also be a path to a specific venv.
    #[serde(default)]
    pub venv: VirtualEnv,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[allow(unused)]
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

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[allow(unused)]
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

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[allow(unused)]
#[serde(untagged)]
pub enum VirtualEnv {
    UseDefault(bool),
    Path(String),
}

impl Default for VirtualEnv {
    fn default() -> Self {
        VirtualEnv::UseDefault(false)
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[allow(unused)]
pub struct FlashOptions {
    /// Whether flash tools are enabled
    pub enabled: bool,

    /// The chip model to flash
    pub chip: String,

    /// The baud rate for the flash operation
    #[serde(default = "default_flash_baudrate")]
    pub baudrate: u32,
}

fn default_flash_baudrate() -> u32 {
    1500000
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub enum SerialResetMethod {
    /// Reset the device by asserting and deasserting DTR
    #[serde(rename = "dtr")]
    DTR,

    /// Reset the device by asserting and deasserting RTS
    #[serde(rename = "rts")]
    RTS,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[allow(unused)]
pub struct SerialOptions {
    /// Whether serial tools are enabled
    pub enabled: bool,

    /// The baud rate for serial communication, default is 115200
    #[serde(default = "default_serial_baudrate")]
    pub baudrate: u32,

    /// Reset method. If specified, the tool will reset the device before each read
    pub reset: Option<SerialResetMethod>,

    /// Optional interval of milliseconds between the reset line is asserted and
    /// deasserted. If not specified, the default is 100ms.
    #[serde(default = "default_reset_interval")]
    pub reset_interval: u64,
}

fn default_serial_baudrate() -> u32 {
    115200
}

fn default_reset_interval() -> u64 {
    100
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[allow(unused)]
pub struct SchematicOptions {
    /// Whether schematic tools are enabled
    pub enabled: bool,

    /// Board name
    pub board: String,

    /// Directory where the SoC metadata is stored, defaults to "schematic/socs"
    #[serde(default = "default_socs_dir")]
    pub socs_dir: PathBuf,

    /// Directory where the board metadata is stored, defaults to "schematic/boards"
    #[serde(default = "default_boards_dir")]
    pub boards_dir: PathBuf,
}

fn default_socs_dir() -> PathBuf {
    "schematic/socs".into()
}

fn default_boards_dir() -> PathBuf {
    "schematic/boards".into()
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[allow(unused)]
pub struct InspectorOptions {
    /// Whether inspector tools are enabled
    pub enabled: bool,
}
