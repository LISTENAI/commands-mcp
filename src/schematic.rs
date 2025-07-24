use std::str::FromStr;

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Soc {
    /// Name of the SoC
    pub name: String,

    /// Description of the SoC
    pub description: Option<String>,

    /// List of bus peripherals, which can be used by more than one external device
    #[serde(default)]
    pub buses: Vec<String>,

    /// List of available pins on the SoC
    #[serde(default)]
    pub pins: Vec<Pin>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Pin {
    /// Name of the pin or pad (e.g., `GPIO0`, `PB2`)
    pub name: String,

    /// List of functions that the pin can perform
    ///
    /// Can be names constructed with the format `<peripheral>.<signal>` (e.g.,
    /// `uart0.txd`, `i2c0.sda`) or just the function name (e.g., `gpio`, `pwm`).
    pub pinmux: Vec<Function>,
}

#[derive(Debug, Clone, JsonSchema)]
pub enum Function {
    /// A simple function that doesn't have explicit signal pins, e.g., `gpio`, `pwm`
    Simple(String),

    /// A function that is associated with a specific peripheral and signal, e.g., `uart0.txd`
    Peripheral { name: String, signal: String },
}

impl ToString for Function {
    fn to_string(&self) -> String {
        match self {
            Function::Simple(name) => name.clone(),
            Function::Peripheral { name, signal } => format!("{}.{}", name, signal),
        }
    }
}

impl FromStr for Function {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((name, signal)) = s.split_once('.') {
            Ok(Function::Peripheral {
                name: name.to_string(),
                signal: signal.to_string(),
            })
        } else {
            Ok(Function::Simple(s.to_string()))
        }
    }
}

impl Serialize for Function {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Function {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Function::from_str(&s).map_err(|e| D::Error::custom(e))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Board {
    /// Name of the board
    pub name: String,

    /// Description of the board
    pub description: Option<String>,

    /// SoC name used by the board
    pub soc: String,

    /// List of modules on the board
    #[serde(default)]
    pub modules: Vec<Module>,

    /// List of pins that are exposed by the board
    #[serde(default)]
    pub exposes: Vec<Expose>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Module {
    /// Name of the module
    pub name: String,

    /// Pins connected to the module
    ///
    /// Each connection is represented as a string in the format `<pin_name>@<function>`,
    /// e.g., `PB2@uart0.txd`.
    pub connects: Vec<Connection>,

    /// Optional name of buses that the module exposes
    #[serde(default)]
    pub buses: Vec<String>,

    /// Optional pins that the module exposes
    #[serde(default)]
    pub pins: Vec<Pin>,
}

#[derive(Debug, Clone, JsonSchema)]
pub struct Connection {
    /// Name of the pin or pad (e.g., `GPIO0`, `PB2`)
    pub pin: String,

    /// Function that the pin performs (e.g., `uart0.txd`, `i2c0.sda`)
    pub function: Function,
}

impl ToString for Connection {
    fn to_string(&self) -> String {
        format!("{}@{}", self.pin, self.function.to_string())
    }
}

impl FromStr for Connection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split('@').collect::<Vec<&str>>().as_slice() {
            [pin, function] if !pin.is_empty() && !function.is_empty() => Ok(Connection {
                pin: pin.to_string(),
                function: Function::from_str(function)?,
            }),
            _ => Err("Invalid connection format".to_string()),
        }
    }
}

impl Serialize for Connection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Connection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Connection::from_str(&s).map_err(|e| D::Error::custom(e))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Expose {
    /// Name of the exposed connector (e.g., `CN1`, `J1`)
    pub name: String,

    /// List of pins that are exposed by the connector (e.g., `PB2`, `GPIO0`)
    pub pins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct App {
    /// Name of the app
    pub name: String,

    /// Description of the app
    pub description: Option<String>,

    /// List of on-board modules enabled for the app
    #[serde(default)]
    pub modules: Vec<String>,
}
