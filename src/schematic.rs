use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[allow(unused)]
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

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[allow(unused)]
pub struct Pin {
    /// Name of the pin or pad (e.g., `GPIO0`, `PB2`)
    pub name: String,

    /// List of functions that the pin can perform
    ///
    /// Can be names constructed with the format `peripheral.signal` (e.g.,
    /// `uart0.txd`, `i2c0.sda`) or just the function name (e.g., `gpio`, `pwm`).
    pub pinmux: Vec<Function>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(try_from = "&str")]
#[allow(unused)]
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

impl TryFrom<&str> for Function {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Some((name, signal)) = value.split_once('.') {
            Ok(Function::Peripheral {
                name: name.into(),
                signal: signal.into(),
            })
        } else {
            Ok(Function::Simple(value.into()))
        }
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[allow(unused)]
pub struct Board {
    /// Name of the board
    pub name: String,

    /// Description of the board
    pub description: Option<String>,

    /// SoC name used by the board
    pub soc: String,

    /// List of devices on the board
    #[serde(default)]
    pub devices: Vec<Device>,

    /// List of pins that are exposed by the board
    #[serde(default)]
    pub exposes: Vec<Expose>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[allow(unused)]
pub struct Device {
    /// Name of the device
    pub name: String,

    /// Pins connected to the device
    ///
    /// Each connection is represented as a string in the format `pin_name@function`,
    /// e.g., `PB2@uart0.txd`.
    pub connects: Vec<Connection>,

    /// Optional name of buses that the device exposes
    #[serde(default)]
    pub buses: Vec<String>,

    /// Optional pins that the device exposes
    #[serde(default)]
    pub pins: Vec<Pin>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(try_from = "&str")]
#[allow(unused)]
pub struct Connection {
    /// Name of the pin or pad (e.g., `GPIO0`, `PB2`), may be with a prefix to
    /// indicate the a IO expander (e.g., `expander1:GPIO0`)
    pub net: Net,

    /// Function that the pin performs (e.g., `uart0.txd`, `i2c0.sda`)
    pub function: Function,
}

impl ToString for Connection {
    fn to_string(&self) -> String {
        format!("{}@{}", self.net.to_string(), self.function.to_string())
    }
}

impl TryFrom<&str> for Connection {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.split('@').collect::<Vec<&str>>().as_slice() {
            [net, function] if !net.is_empty() && !function.is_empty() => Ok(Connection {
                net: Net::try_from(*net)?,
                function: Function::try_from(*function)?,
            }),
            _ => Err("Invalid connection format".to_string()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(try_from = "&str")]
#[allow(unused)]
pub enum Net {
    /// A net that directly connects to the SoC
    Direct { pin: String },

    /// A net that connects through a device
    Device { device: String, pin: String },
}

impl ToString for Net {
    fn to_string(&self) -> String {
        match self {
            Net::Direct { pin } => pin.clone(),
            Net::Device { device, pin } => format!("{}:{}", device, pin),
        }
    }
}

impl TryFrom<&str> for Net {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.split(':').collect::<Vec<&str>>().as_slice() {
            [device, pin] if !device.is_empty() && !pin.is_empty() => Ok(Net::Device {
                device: device.to_string(),
                pin: pin.to_string(),
            }),
            [pin] if !pin.is_empty() => Ok(Net::Direct {
                pin: pin.to_string(),
            }),
            _ => Err("Invalid net format".to_string()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[allow(unused)]
pub struct Expose {
    /// Name of the exposed connector (e.g., `CN1`, `J1`)
    pub name: String,

    /// List of pins that are exposed by the connector (e.g., `PB2`, `GPIO0`)
    pub pins: Vec<Net>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[allow(unused)]
pub struct App {
    /// Name of the app
    pub name: String,

    /// Description of the app
    pub description: Option<String>,

    /// List of on-board devices enabled for the app
    #[serde(default)]
    pub devices: Vec<String>,
}
