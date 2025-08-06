use std::{
    io::{Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult, Write},
    thread::sleep,
    time::Duration,
};

use rusb::{
    Device as UsbDevice, DeviceDescriptor, DeviceHandle, Direction, EndpointDescriptor,
    Error as UsbError, GlobalContext, TransferType,
};

pub struct Device {
    pub id: String,
    pub name: String,
    pub handle: DeviceHandle<GlobalContext>,
    interface: Interface,
    read_buf: Vec<u8>,
    read_eos: bool,
    write_buf: Vec<u8>,
}

impl TryFrom<UsbDevice<GlobalContext>> for Device {
    type Error = UsbError;

    fn try_from(device: UsbDevice<GlobalContext>) -> Result<Self, Self::Error> {
        let desc = device.device_descriptor()?;

        let interface = Interface::find_first(&device, &desc).ok_or(UsbError::NoDevice)?;

        let handle = device.open()?;

        let id = handle.read_serial_number_string_ascii(&desc)?;

        let name = format!(
            "{} {}",
            handle
                .read_manufacturer_string_ascii(&desc)
                .unwrap_or_else(|_| "Unknown Manufacturer".to_string()),
            handle
                .read_product_string_ascii(&desc)
                .unwrap_or_else(|_| "Unknown Product".to_string())
        );

        Ok(Self {
            id,
            name,
            handle,
            interface,
            read_buf: vec![],
            read_eos: false,
            write_buf: vec![],
        })
    }
}

impl Device {
    pub fn claim(&mut self) -> Result<(), UsbError> {
        self.interface.claim(&self.handle)?;
        sleep(Duration::from_millis(100));
        Ok(())
    }
}

impl Read for Device {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        while self.read_buf.len() < buf.len() && !self.read_eos {
            let read_len = self.interface.ep_in.mps as usize;
            let mut read_buf = vec![0u8; read_len];

            let actual_read = self
                .handle
                .read_bulk(
                    self.interface.ep_in.address,
                    &mut read_buf,
                    Duration::from_secs(1),
                )
                .map_err(|e| IoError::new(IoErrorKind::Other, e))?;

            self.read_buf.extend_from_slice(&read_buf[..actual_read]);

            if actual_read < read_len {
                self.read_eos = true;
                break;
            }
        }

        let copy_len = buf.len().min(self.read_buf.len());
        buf[..copy_len].copy_from_slice(&self.read_buf[..copy_len]);

        self.read_buf.drain(..copy_len);

        if self.read_buf.is_empty() {
            self.read_eos = false;
        }

        Ok(copy_len)
    }
}

impl Write for Device {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.write_buf.extend_from_slice(buf);
        let write_len = self.interface.ep_out.mps as usize;
        if self.write_buf.len() >= write_len {
            self.handle
                .write_bulk(
                    self.interface.ep_out.address,
                    &self.write_buf[..write_len],
                    Duration::from_secs(1),
                )
                .map_err(|e| IoError::new(IoErrorKind::Other, e))?;

            self.write_buf.drain(..write_len);
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> IoResult<()> {
        self.handle
            .write_bulk(
                self.interface.ep_out.address,
                &self.write_buf,
                Duration::from_secs(1),
            )
            .map_err(|e| IoError::new(IoErrorKind::Other, e))?;

        self.write_buf.clear();

        Ok(())
    }
}

struct Interface {
    config: u8,
    iface: u8,
    setting: u8,
    pub ep_in: Endpoint,
    pub ep_out: Endpoint,
}

impl Interface {
    pub fn find_first(
        device: &UsbDevice<GlobalContext>,
        device_desc: &DeviceDescriptor,
    ) -> Option<Self> {
        for n in 0..device_desc.num_configurations() {
            let config_desc = match device.config_descriptor(n) {
                Ok(c) => c,
                Err(_) => continue,
            };

            for interface in config_desc.interfaces() {
                for interface_desc in interface.descriptors() {
                    if (
                        interface_desc.class_code(),
                        interface_desc.sub_class_code(),
                        interface_desc.protocol_code(),
                    ) == (0xFF, 67, 77)
                    {
                        let ep_in = interface_desc.endpoint_descriptors().find(|ep| {
                            ep.transfer_type() == TransferType::Bulk
                                && ep.direction() == Direction::In
                        })?;

                        let ep_out = interface_desc.endpoint_descriptors().find(|ep| {
                            ep.transfer_type() == TransferType::Bulk
                                && ep.direction() == Direction::Out
                        })?;

                        return Some(Self {
                            config: config_desc.number(),
                            iface: interface_desc.interface_number(),
                            setting: interface_desc.setting_number(),
                            ep_in: Endpoint::from(ep_in),
                            ep_out: Endpoint::from(ep_out),
                        });
                    }
                }
            }
        }

        None
    }

    pub fn claim(&self, handle: &DeviceHandle<GlobalContext>) -> Result<(), UsbError> {
        handle.set_active_configuration(self.config)?;
        handle.claim_interface(self.iface)?;
        handle.set_alternate_setting(self.iface, self.setting)?;
        Ok(())
    }
}

struct Endpoint {
    pub address: u8,
    pub mps: u16,
}

impl From<EndpointDescriptor<'_>> for Endpoint {
    fn from(ep: rusb::EndpointDescriptor) -> Self {
        Self {
            address: ep.address(),
            mps: ep.max_packet_size(),
        }
    }
}
