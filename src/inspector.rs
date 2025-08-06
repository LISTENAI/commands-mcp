use std::io::Write;

use binrw::{BinRead, BinWrite, binrw, io::NoSeek};
use rmcp::Error as McpError;

use crate::inspector_device::Device;

pub struct Inspector {}

impl Inspector {
    pub fn new() -> Self {
        Self {}
    }

    pub fn list_devices(&self) -> Result<Vec<Device>, McpError> {
        let devices = rusb::devices().map_err(|e| {
            McpError::internal_error(format!("Failed to list devices: {}", e), None)
        })?;

        let devices = devices
            .iter()
            .filter_map(|device| Device::try_from(device).ok())
            .collect::<Vec<_>>()
            .into();

        Ok(devices)
    }

    fn open_device(&self, id: &str) -> Result<Device, McpError> {
        let devices = self.list_devices()?;
        devices.into_iter().find(|d| d.id == id).ok_or_else(|| {
            McpError::invalid_params(format!("Device with ID {} not found", id), None)
        })
    }

    pub fn take_screenshot(&self, id: &str) -> Result<Vec<u8>, McpError> {
        let mut device = self.open_device(id)?;
        device.claim().map_err(|e| {
            McpError::internal_error(format!("Failed to claim device: {}", e), None)
        })?;

        Envelope {
            version: 1,
            command: Command::SNAPSHOT,
            is_res: 0,
            rc: 0,
            payload: vec![],
        }
        .write_to(&mut device)?;

        let envelope = Envelope::read_from(&mut device)?;
        eprintln!("Received envelope: {:?}", envelope);

        Ok(vec![])
    }
}

#[binrw]
#[brw(little)]
#[derive(Debug, PartialEq, Eq)]
enum Command {
    #[brw(magic = 0x01u8)]
    SNAPSHOT,
}

#[binrw]
#[brw(little, magic = b"COMMANDS")]
#[derive(Debug)]
struct Envelope {
    #[bw(calc = 4 + payload.len() as u32)]
    pub length: u32,

    pub version: u8,

    pub command: Command,
    pub is_res: u8,
    pub rc: u8,

    #[br(count = length - 4)]
    pub payload: Vec<u8>,
}

impl Envelope {
    pub fn read_from(device: &mut Device) -> Result<Self, McpError> {
        let mut reader = NoSeek::new(device);
        Envelope::read(&mut reader).map_err(|e| {
            McpError::internal_error(format!("Failed to read from device: {}", e), None)
        })
    }

    pub fn write_to(&self, device: &mut Device) -> Result<(), McpError> {
        let mut writer = NoSeek::new(device);
        self.write(&mut writer).map_err(|e| {
            McpError::internal_error(format!("Failed to write to device: {}", e), None)
        })?;
        writer.flush().map_err(|e| {
            McpError::internal_error(format!("Failed to flush device: {}", e), None)
        })?;
        Ok(())
    }
}
