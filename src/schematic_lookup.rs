use std::collections::HashSet;

use crate::schematic::{App, Board, Connection, Device, Function, Net, Soc};

impl Soc {
    pub fn nets(&self) -> impl Iterator<Item = (Net, &Vec<Function>)> {
        self.pins.iter().map(|pin| {
            (
                Net::Direct {
                    pin: pin.name.clone(),
                },
                &pin.pinmux,
            )
        })
    }
}

impl Function {
    pub fn peripheral(&self) -> &str {
        match self {
            Function::Simple(name) => name.as_str(),
            Function::Peripheral { name, .. } => name.as_str(),
        }
    }

    pub fn signal(&self) -> Option<&str> {
        match self {
            Function::Simple(_) => None,
            Function::Peripheral { signal, .. } => Some(signal.as_str()),
        }
    }

    pub fn is(&self, peripheral: &str) -> bool {
        self.peripheral() == peripheral
    }
}

impl Board {
    pub fn devices_with_status(&self, app: &App) -> impl Iterator<Item = (&Device, DeviceStatus)> {
        let enabled = app.devices_as_set();
        self.devices.iter().map(move |device| {
            let status = if enabled.contains(device.name.as_str()) {
                DeviceStatus::Used
            } else {
                DeviceStatus::Free
            };
            (device, status)
        })
    }

    pub fn device_by_name(&self, name: &str) -> Option<&Device> {
        self.devices.iter().find(|device| device.name == name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceStatus {
    Used,
    Free,
}

impl ToString for DeviceStatus {
    fn to_string(&self) -> String {
        match self {
            DeviceStatus::Used => "Used".to_string(),
            DeviceStatus::Free => "Free".to_string(),
        }
    }
}

impl Device {
    pub fn connection_to(&self, net: &Net) -> Option<&Connection> {
        self.connects.iter().find(|conn| conn.net == *net)
    }

    pub fn connected(&self, net: &Net, function: &Function) -> bool {
        self.connects
            .iter()
            .find(|conn| conn.net == *net && conn.function.is(function.peripheral()))
            .is_some()
    }

    pub fn nets(&self) -> impl Iterator<Item = (Net, &Vec<Function>)> {
        self.pins.iter().map(|pin| {
            (
                Net::Device {
                    device: self.name.clone(),
                    pin: pin.name.clone(),
                },
                &pin.pinmux,
            )
        })
    }
}

impl App {
    pub fn devices_as_set(&self) -> HashSet<&str> {
        self.devices
            .iter()
            .map(|device| device.as_str())
            .collect::<HashSet<_>>()
    }
}
