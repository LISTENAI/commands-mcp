use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// A collection of commands
    pub commands: HashMap<String, CommandSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSpec {
    /// A brief description of the command
    pub description: String,

    /// The arguments for the command
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<ArgumentSpec>>,

    /// The command template
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
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
