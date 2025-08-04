use std::{collections::BTreeMap, fmt::Debug};

use rmcp::{
    Error as McpError,
    handler::server::tool::Parameters,
    model::{CallToolResult, Content},
    schemars, tool, tool_router,
};
use serde::Deserialize;

use crate::{
    commands::Commands,
    manifest::SchematicOptions,
    manifest_reader::ManifestReader,
    schematic::{App, Board, Net, Soc},
    schematic_lookup::DeviceStatus,
};

#[tool_router(router =schematic_router, vis = "pub")]
impl Commands {
    #[tool(
        name = "schematic_list_devices",
        description = "List devices for a given app. Devices are components that \
            placed on the board the specific app is running on. List of devices \
            that the app is using are declared in the app manifest (the \
            `schematic.yaml` file located in the app directory). A device can \
            be either used (declared) or free (not declared).",
        annotations(read_only_hint = true)
    )]
    async fn schematic_list_devices(
        &self,
        Parameters(ListDevicesRequest { app }): Parameters<ListDevicesRequest>,
    ) -> Result<CallToolResult, McpError> {
        let app = App::for_spec(&self, &app)?;
        let board = Board::for_spec(&self)?;

        let mut response = String::new();
        response.push_str(format!("Devices on board '{}':\n", board.name).as_str());
        response.push_str("\n");
        response.push_str("| Name | Status |\n");
        response.push_str("| ---- | ------ |\n");
        for (device, status) in board.devices_with_status(&app) {
            response.push_str(&format!("| {} | {} |\n", device.name, status.to_string()));
        }

        Ok(CallToolResult::success(vec![Content::text(response)]))
    }

    #[tool(
        name = "schematic_list_pins_used_by_device",
        description = "List pins used by a specific device, along with other \
            devices that are using those pins.",
        annotations(read_only_hint = true)
    )]
    async fn schematic_list_pins_used_by_device(
        &self,
        Parameters(ListPinsUsedByDeviceRequest { app, device }): Parameters<
            ListPinsUsedByDeviceRequest,
        >,
    ) -> Result<CallToolResult, McpError> {
        let app = App::for_spec(&self, &app)?;
        let board = Board::for_spec(&self)?;

        let device = board
            .device_by_name(&device)
            .ok_or(McpError::invalid_params(
                format!("Device '{}' not found", device),
                None,
            ))?;

        let devices = board.devices_with_status(&app).collect::<Vec<_>>();

        let connects = device
            .connects
            .iter()
            .map(|conn| {
                let usage = devices.iter().find_map(|(device, status)| {
                    if device.connection_to(&conn.net).is_some() && *status == DeviceStatus::Used {
                        Some(*device)
                    } else {
                        None
                    }
                });
                (conn, usage)
            })
            .collect::<Vec<_>>();

        if connects.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Device '{}' does not have any connections",
                device.name
            ))]));
        }

        let mut response = String::new();
        response.push_str(&format!("Connections for device '{}':\n", device.name));
        response.push_str("\n");
        response.push_str("| Pin | Function | Status |\n");
        response.push_str("| --- | -------- | ------ |\n");
        for (conn, usages) in connects {
            response.push_str(&format!(
                "| {} | {} | {} |\n",
                conn.net.to_string(),
                conn.function.to_string(),
                match usages {
                    Some(device) => format!("Used by {}", device.name),
                    None => "Free".to_string(),
                }
            ));
        }

        Ok(CallToolResult::success(vec![Content::text(response)]))
    }

    #[tool(
        name = "schematic_find_device_using_pin",
        description = "Find devices on the board using a specific pin, along \
            with the device status (used or free) and the function the pin \
            is used for.",
        annotations(read_only_hint = true)
    )]
    async fn schematic_find_device_using_pin(
        &self,
        Parameters(FindDeviceUsingPinRequest { app, pin }): Parameters<FindDeviceUsingPinRequest>,
    ) -> Result<CallToolResult, McpError> {
        let app = App::for_spec(&self, &app)?;
        let board = Board::for_spec(&self)?;
        let soc = Soc::for_spec(&self, &board)?;

        let net = Net::try_from(pin.as_str())
            .map_err(|e| McpError::invalid_params(format!("Invalid pin format: {}", e), None))?;

        if !soc
            .nets()
            .chain(board.devices.iter().flat_map(|device| device.nets()))
            .any(|(n, _)| n == net)
        {
            return Err(McpError::invalid_params(
                format!("Pin '{}' not found on the board", pin),
                None,
            ));
        }

        let devices = board.devices_with_status(&app);

        let usages = devices
            .filter_map(|(device, status)| {
                if let Some(conn) = device.connection_to(&net) {
                    Some((device, status, conn))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if usages.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "No module found using pin '{}'",
                pin
            ))]));
        }

        let mut response = String::new();
        response.push_str(&format!("Devices using pin '{}':\n", pin));
        response.push_str("\n");
        response.push_str("| Device | Status | Function |\n");
        response.push_str("| ------ | ------ | -------- |\n");
        for (device, status, conn) in usages {
            response.push_str(&format!(
                "| {} | {} | {} |\n",
                device.name,
                status.to_string(),
                conn.function.to_string()
            ));
        }

        Ok(CallToolResult::success(vec![Content::text(response)]))
    }

    #[tool(
        name = "schematic_list_peripherals",
        description = "List all peripherals, such as spi0, i2c1, etc., available \
            to the specific app, along with devices and their status (used or \
            free) that are using those peripherals. Peripherals are provided \
            by either the SoC or devices like IO expanders on the board.",
        annotations(read_only_hint = true)
    )]
    async fn schematic_list_peripherals(
        &self,
        Parameters(ListPeripheralsRequest { app }): Parameters<ListPeripheralsRequest>,
    ) -> Result<CallToolResult, McpError> {
        let app = App::for_spec(&self, &app)?;
        let board = Board::for_spec(&self)?;
        let soc = Soc::for_spec(&self, &board)?;

        let devices = board.devices_with_status(&app).collect::<Vec<_>>();

        let peripherals = soc
            .nets()
            .flat_map(|(net, pinmux)| {
                pinmux
                    .iter()
                    .map(|function| {
                        let devices = devices
                            .iter()
                            .filter(|(d, _)| d.connected(&net, function))
                            .collect::<Vec<_>>();
                        (function, devices)
                    })
                    .collect::<Vec<_>>()
            })
            .fold(BTreeMap::new(), |mut acc, (function, devices)| {
                let entry = acc
                    .entry(function.peripheral())
                    .or_insert_with(BTreeMap::new);
                for (device, status) in devices {
                    entry.insert(device.name.clone(), status);
                }
                acc
            });

        let mut response = String::new();
        response.push_str("Peripherals from SoC:\n");
        response.push_str("\n");
        response.push_str("| Name | Connected Devices (status) |\n");
        response.push_str("| ---- | -------------------------- |\n");
        for (name, usages) in peripherals {
            response.push_str(&format!(
                "| {} | {} |\n",
                name,
                usages
                    .iter()
                    .map(|(device, status)| format!("{} ({})", device, status.to_string()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        for (device, status) in &devices {
            let peripherals = device
                .nets()
                .flat_map(|(net, pinmux)| {
                    pinmux
                        .iter()
                        .map(|function| {
                            let devices = devices
                                .iter()
                                .filter(|(d, _)| d.connected(&net, function))
                                .collect::<Vec<_>>();
                            (function, devices)
                        })
                        .collect::<Vec<_>>()
                })
                .fold(BTreeMap::new(), |mut acc, (function, devices)| {
                    let entry = acc
                        .entry(function.peripheral())
                        .or_insert_with(BTreeMap::new);
                    for (device, status) in devices {
                        entry.insert(device.name.clone(), status);
                    }
                    acc
                });

            if peripherals.is_empty() {
                continue;
            }

            response.push_str("\n");
            response.push_str(&format!(
                "Peripherals exposed from device '{}' (status: {}):\n",
                device.name,
                status.to_string()
            ));
            response.push_str("\n");
            response.push_str("| Name | Connected Devices (status) |\n");
            response.push_str("| ---- | -------------------------- |\n");
            for (name, usages) in peripherals {
                response.push_str(&format!(
                    "| {} | {} |\n",
                    name,
                    usages
                        .iter()
                        .map(|(device, status)| format!("{} ({})", device, status.to_string()))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }

        Ok(CallToolResult::success(vec![Content::text(response)]))
    }

    #[tool(
        name = "schematic_list_peripheral_pins",
        description = "List pins can be used by a specific peripheral, both from \
            the SoC and from expander devices on the board, along with the \
            devices connected to those pins and their status (used or free).",
        annotations(read_only_hint = true)
    )]
    async fn schematic_list_peripheral_pins(
        &self,
        Parameters(ListPeripheralPinsRequest { app, peripheral }): Parameters<
            ListPeripheralPinsRequest,
        >,
    ) -> Result<CallToolResult, McpError> {
        let app = App::for_spec(&self, &app)?;
        let board = Board::for_spec(&self)?;
        let soc = Soc::for_spec(&self, &board)?;

        let devices = board.devices_with_status(&app).collect::<Vec<_>>();

        let pins = soc
            .nets()
            .chain(board.devices.iter().flat_map(|device| device.nets()))
            .filter_map(|(net, pinmux)| {
                pinmux.iter().find_map(|function| {
                    if function.is(&peripheral) {
                        let usage = devices.iter().find_map(|(device, status)| {
                            if device.connection_to(&net).is_some() && *status == DeviceStatus::Used
                            {
                                Some(*device)
                            } else {
                                None
                            }
                        });

                        Some((net.clone(), function, usage))
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>();

        let mut response = String::new();
        response.push_str(&format!(
            "Pins can be used by peripheral '{}':\n",
            peripheral
        ));
        response.push_str("\n");
        response.push_str("| Pin | Function | Status |\n");
        response.push_str("| --- | -------- | ------ |\n");
        for (net, function, usage) in pins {
            response.push_str(&format!(
                "| {} | {} | {} |\n",
                net.to_string(),
                function.to_string(),
                match usage {
                    Some(device) => format!("Used by {}", device.name),
                    None => "Free".to_string(),
                }
            ));
        }

        Ok(CallToolResult::success(vec![Content::text(response)]))
    }

    #[tool(
        name = "schematic_find_peripheral_using_pin",
        description = "Find peripherals using a specific pin, along with the \
            devices connected to that pin and their status (used or free).",
        annotations(read_only_hint = true)
    )]
    async fn schematic_find_peripheral_using_pin(
        &self,
        Parameters(FindPeripheralUsingPinRequest { app, pin }): Parameters<
            FindPeripheralUsingPinRequest,
        >,
    ) -> Result<CallToolResult, McpError> {
        let app = App::for_spec(&self, &app)?;
        let board = Board::for_spec(&self)?;
        let soc = Soc::for_spec(&self, &board)?;

        let net = Net::try_from(pin.as_str())
            .map_err(|e| McpError::invalid_params(format!("Invalid pin format: {}", e), None))?;

        let (_, pinmux) = soc
            .nets()
            .chain(board.devices.iter().flat_map(|device| device.nets()))
            .find(|(n, _)| n == &net)
            .ok_or(McpError::invalid_params(
                format!("Pin '{}' not found on the board", pin),
                None,
            ))?;

        let devices = board.devices_with_status(&app).collect::<Vec<_>>();

        let usages = pinmux
            .iter()
            .map(|function| {
                let devices = devices
                    .iter()
                    .filter_map(|(device, status)| match device.connection_to(&net) {
                        Some(conn) if conn.function == *function => Some((*device, status)),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                (function, devices)
            })
            .collect::<Vec<_>>();

        if usages.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "No peripheral found using pin '{}'",
                pin
            ))]));
        }

        let mut response = String::new();
        response.push_str(&format!("Peripherals using pin '{}':\n", pin));
        response.push_str("\n");
        response.push_str("| Peripheral | Signal | Connected Devices (status) |\n");
        response.push_str("| ---------- | ------ | -------------------------- |\n");
        for (function, devices) in usages {
            response.push_str(&format!(
                "| {} | {} | {} |\n",
                function.peripheral(),
                function.signal().unwrap_or("-"),
                devices
                    .iter()
                    .map(|(device, status)| format!("{} ({})", device.name, status.to_string()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        Ok(CallToolResult::success(vec![Content::text(response)]))
    }

    #[tool(
        name = "schematic_list_exposes",
        description = "List all pins exposed by the board, along with devices \
            connected to those pins and their status (used or free). Exposed \
            pins are connectors on the board that give the user convenience to \
            connect any external devices to the board.",
        annotations(read_only_hint = true)
    )]
    async fn schematic_list_exposes(
        &self,
        Parameters(ListExposesRequest { app }): Parameters<ListExposesRequest>,
    ) -> Result<CallToolResult, McpError> {
        let app = App::for_spec(&self, &app)?;
        let board = Board::for_spec(&self)?;

        if board.exposes.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                "No exposed pins found on the board.".to_string(),
            )]));
        }

        let devices = board.devices_with_status(&app).collect::<Vec<_>>();

        let mut response = String::new();

        for expose in board.exposes.iter() {
            let pins = expose
                .pins
                .iter()
                .map(|net| {
                    let usage = devices.iter().find_map(|(device, status)| {
                        if device.connection_to(&net).is_some() && *status == DeviceStatus::Used {
                            Some(*device)
                        } else {
                            None
                        }
                    });
                    (net, usage)
                })
                .collect::<Vec<_>>();

            response.push_str(&format!("Pins exposed via {}:\n", expose.name));
            response.push_str("\n");
            response.push_str("| Pin | Status |\n");
            response.push_str("| --- | ------ |\n");
            for (net, usage) in pins {
                response.push_str(&format!(
                    "| {} | {} |\n",
                    net.to_string(),
                    match usage {
                        Some(device) => format!("Used by {}", device.name),
                        None => "Free".to_string(),
                    }
                ));
            }
            response.push_str("\n");
        }

        Ok(CallToolResult::success(vec![Content::text(response)]))
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListDevicesRequest {
    #[schemars(description = "Path to the app to list devices for")]
    pub app: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListPinsUsedByDeviceRequest {
    #[schemars(description = "Path to the app to list pins for")]
    pub app: String,

    #[schemars(description = "Name of the device to list pins for")]
    pub device: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FindDeviceUsingPinRequest {
    #[schemars(description = "Path to the app")]
    pub app: String,

    #[schemars(description = "Name of the pin to find device for")]
    pub pin: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListPeripheralsRequest {
    #[schemars(description = "Path to the app to list peripherals for")]
    pub app: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListPeripheralPinsRequest {
    #[schemars(description = "Path to the app to list peripherals for")]
    pub app: String,

    #[schemars(description = "Name of the peripheral to list pins for")]
    pub peripheral: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FindPeripheralUsingPinRequest {
    #[schemars(description = "Path to the app to find peripheral for")]
    pub app: String,

    #[schemars(description = "Name of the pin to find peripheral for")]
    pub pin: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListExposesRequest {
    #[schemars(description = "Path to the app to list exposes for")]
    pub app: String,
}

impl SchematicOptions {
    pub fn from(spec: &Commands) -> Result<&Self, McpError> {
        spec.manifest
            .schematic
            .as_ref()
            .ok_or(McpError::invalid_params(
                "Schematic options are not defined in the manifest".to_string(),
                None,
            ))
    }
}

impl App {
    pub fn for_spec(spec: &Commands, app_path: &str) -> Result<Self, McpError> {
        let app_path = spec.cwd.join(app_path).join("schematic.yaml");
        Self::read_from(app_path).map_err(|e| {
            McpError::internal_error(format!("Failed to read app manifest: {}", e), None)
        })
    }
}

impl Board {
    pub fn for_spec(spec: &Commands) -> Result<Self, McpError> {
        let schematic_opts = SchematicOptions::from(spec)?;

        let board_path = spec
            .cwd
            .join(&schematic_opts.boards_dir)
            .join(format!("{}.yaml", schematic_opts.board));

        Board::read_from(board_path).map_err(|e| {
            McpError::internal_error(format!("Failed to read board manifest: {}", e), None)
        })
    }
}

impl Soc {
    pub fn for_spec(spec: &Commands, board: &Board) -> Result<Self, McpError> {
        let schematic_opts = SchematicOptions::from(spec)?;

        let soc_path = spec
            .cwd
            .join(&schematic_opts.socs_dir)
            .join(format!("{}.yaml", board.soc));

        Soc::read_from(soc_path).map_err(|e| {
            McpError::internal_error(format!("Failed to read SoC manifest: {}", e), None)
        })
    }
}
