use std::{collections::HashSet, fmt::Debug};

use rmcp::{
    Error as McpError,
    handler::server::tool::Parameters,
    model::{CallToolResult, Content},
    schemars, tool, tool_router,
};
use serde::Deserialize;

use crate::{
    commands::Commands,
    manifest_reader::ManifestReader,
    schematic::{App, Board, Connection, Function, Module, Soc},
};

#[tool_router(router =schematic_router, vis = "pub")]
impl Commands {
    fn load_app(&self, app_path: &str) -> Result<App, McpError> {
        let app_path = self.cwd.join(app_path).join("schematic.yaml");
        App::read_from(app_path).map_err(|e| {
            McpError::internal_error(format!("Failed to read app manifest: {}", e), None)
        })
    }

    fn load_schematic(&self) -> Result<(Soc, Board), McpError> {
        let schematic_opts = self
            .manifest
            .schematic
            .as_ref()
            .ok_or(McpError::invalid_params(
                "Schematic options are not defined in the manifest".to_string(),
                None,
            ))?;

        let board_path = self
            .cwd
            .join(&schematic_opts.boards_dir)
            .join(format!("{}.yaml", schematic_opts.board));

        let board = Board::read_from(board_path).map_err(|e| {
            McpError::internal_error(format!("Failed to read board manifest: {}", e), None)
        })?;

        let soc_path = self
            .cwd
            .join(&schematic_opts.socs_dir)
            .join(format!("{}.yaml", board.soc));

        let soc = Soc::read_from(soc_path).map_err(|e| {
            McpError::internal_error(format!("Failed to read SoC manifest: {}", e), None)
        })?;

        Ok((soc, board))
    }

    #[tool(
        name = "schematic_list_modules",
        description = "List modules for a given app. Modules are components or \
        devices that placed on the board the specific app is running on. Modules \
        can be enabled or disabled in the app manifest (the `schematic.yaml` file \
        placed in the app directory).",
        annotations(read_only_hint = true)
    )]
    async fn schematic_list_modules(
        &self,
        Parameters(ListModulesRequest { app }): Parameters<ListModulesRequest>,
    ) -> Result<CallToolResult, McpError> {
        let (_soc, board) = self.load_schematic()?;
        let app = self.load_app(&app)?;

        let mut response = String::new();
        response.push_str("Available modules:\n");
        response.push_str("\n");
        response.push_str("| Name | Enabled |\n");
        response.push_str("| ---- | ------- |\n");
        for (module, enabled) in board.modules_with_state(&app) {
            response.push_str(&format!(
                "| {} | {} |\n",
                module.name,
                if enabled { "Yes" } else { "No" }
            ));
        }

        Ok(CallToolResult::success(vec![Content::text(response)]))
    }

    #[tool(
        name = "schematic_list_pins_used_by_module",
        description = "List pins used by a specific module",
        annotations(read_only_hint = true)
    )]
    async fn schematic_list_pins_used_by_module(
        &self,
        Parameters(ListPinsUsedByModuleRequest { module }): Parameters<ListPinsUsedByModuleRequest>,
    ) -> Result<CallToolResult, McpError> {
        let (_soc, board) = self.load_schematic()?;

        let module = board
            .module_by_name(&module)
            .ok_or(McpError::invalid_params(
                format!("Module '{}' not found", module),
                None,
            ))?;

        if module.connects.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Module '{}' does not have any connections",
                module.name
            ))]));
        }

        let mut response = String::new();
        response.push_str(&format!("Connections for module '{}':\n", module.name));
        response.push_str("\n");
        response.push_str("| Pin | Function |\n");
        response.push_str("| --- | -------- |\n");
        for Connection { pin, function } in &module.connects {
            response.push_str(&format!("| {} | {} |\n", pin, function.to_string()));
        }

        Ok(CallToolResult::success(vec![Content::text(response)]))
    }

    #[tool(
        name = "schematic_find_module_using_pin",
        description = "Find modules on the board using a specific pin, along \
        with the module state (enabled or disabled) and the function the pin \
        is used for.",
        annotations(read_only_hint = true)
    )]
    async fn schematic_find_module_using_pin(
        &self,
        Parameters(FindModuleUsingPinRequest { app, pin }): Parameters<FindModuleUsingPinRequest>,
    ) -> Result<CallToolResult, McpError> {
        let (_soc, board) = self.load_schematic()?;
        let app = self.load_app(&app)?;

        let modules = board.modules_with_state(&app);

        let usages = modules
            .filter_map(|(m, enabled)| {
                if let Some(conn) = m.connection_to(&pin) {
                    Some((m, enabled, conn))
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
        response.push_str(&format!("Modules using pin '{}':\n", pin));
        response.push_str("\n");
        response.push_str("| Module | Enabled | Function |\n");
        for (module, enabled, conn) in usages {
            response.push_str(&format!(
                "| {} | {} | {} |\n",
                module.name,
                if enabled { "Yes" } else { "No" },
                conn.function.to_string()
            ));
        }

        Ok(CallToolResult::success(vec![Content::text(response)]))
    }

    #[tool(
        name = "schematic_list_peripherals",
        description = "List all peripherals of the SoC running the app, along \
        with modules using them",
        annotations(read_only_hint = true)
    )]
    async fn schematic_list_peripherals(
        &self,
        Parameters(ListPeripheralsRequest { app }): Parameters<ListPeripheralsRequest>,
    ) -> Result<CallToolResult, McpError> {
        let (soc, board) = self.load_schematic()?;
        let app = self.load_app(&app)?;

        let modules = board.modules_with_state(&app).collect::<Vec<_>>();

        let mut peripherals = soc
            .peripherals()
            .into_iter()
            .map(|name| {
                let modules = modules
                    .iter()
                    .filter(|(m, _)| m.connection_using(name).is_some())
                    .collect::<Vec<_>>();

                (name, modules)
            })
            .collect::<Vec<_>>();

        peripherals.sort_by_key(|(name, _)| *name);

        let mut response = String::new();
        response.push_str("Peripherals:\n");
        response.push_str("\n");
        response.push_str("| Name | Modules |\n");
        response.push_str("| ---- | ------- |\n");
        for (name, modules) in peripherals {
            response.push_str(&format!(
                "| {} | {} |\n",
                name,
                modules
                    .iter()
                    .map(|(m, e)| format!(
                        "{} ({})",
                        m.name,
                        if *e { "enabled" } else { "disabled" }
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        Ok(CallToolResult::success(vec![Content::text(response)]))
    }

    #[tool(
        name = "schematic_list_pins_for_peripheral",
        description = "List pins can be used by a specific peripheral, along \
        with the modules connected to those pins and their state (enabled or \
        disabled).",
        annotations(read_only_hint = true)
    )]
    async fn schematic_list_pins_for_peripheral(
        &self,
        Parameters(ListPinsUsedByPeripheralRequest { app, peripheral }): Parameters<
            ListPinsUsedByPeripheralRequest,
        >,
    ) -> Result<CallToolResult, McpError> {
        let (soc, board) = self.load_schematic()?;
        let app = self.load_app(&app)?;

        let modules = board.modules_with_state(&app).collect::<Vec<_>>();

        let pins = soc
            .pins
            .iter()
            .filter_map(|pin| {
                pin.pinmux.iter().find_map(|f| {
                    if f.is(&peripheral) {
                        let module = modules
                            .iter()
                            .find(|(m, _)| m.connection_to(&pin.name).is_some());

                        Some((pin, f, module))
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
        response.push_str("| Pin | Function | Connected Module | Module Enabled |\n");
        response.push_str("| --- | -------- | ---------------- | -------------- |\n");
        for (pin, function, module) in pins {
            match module {
                Some((m, enabled)) => {
                    response.push_str(&format!(
                        "| {} | {} | {} | {} |\n",
                        pin.name,
                        function.to_string(),
                        m.name,
                        if *enabled { "Yes" } else { "No" }
                    ));
                }
                None => {
                    response.push_str(&format!("| {} | - | - |\n", pin.name));
                }
            }
        }

        Ok(CallToolResult::success(vec![Content::text(response)]))
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListModulesRequest {
    #[schemars(description = "Path to the app to list modules for")]
    pub app: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListPinsUsedByModuleRequest {
    #[schemars(description = "Name of the module to list pins for")]
    pub module: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FindModuleUsingPinRequest {
    #[schemars(description = "Path to the app")]
    pub app: String,

    #[schemars(description = "Name of the pin to find module for")]
    pub pin: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListPeripheralsRequest {
    #[schemars(description = "Path to the app to list peripherals for")]
    pub app: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListPinsUsedByPeripheralRequest {
    #[schemars(description = "Path to the app to list peripherals for")]
    pub app: String,

    #[schemars(description = "Name of the peripheral to list pins for")]
    pub peripheral: String,
}

impl Soc {
    pub fn peripherals(&self) -> HashSet<&str> {
        self.pins
            .iter()
            .flat_map(|pin| {
                pin.pinmux.iter().map(|f| match f {
                    Function::Simple(name) => name.as_str(),
                    Function::Peripheral { name, .. } => name.as_str(),
                })
            })
            .collect::<HashSet<_>>()
    }
}

impl Function {
    pub fn is(&self, peripheral: &str) -> bool {
        match self {
            Function::Simple(name) => name == peripheral,
            Function::Peripheral { name, .. } => name == peripheral,
        }
    }
}

impl Board {
    pub fn modules_with_state(&self, app: &App) -> impl Iterator<Item = (&Module, bool)> {
        let enabled = app.modules_as_set();
        self.modules
            .iter()
            .map(move |m| (m, enabled.contains(m.name.as_str())))
    }

    pub fn module_by_name(&self, name: &str) -> Option<&Module> {
        self.modules.iter().find(|m| m.name == name)
    }
}

impl Module {
    pub fn connection_to(&self, pin: &str) -> Option<&Connection> {
        self.connects.iter().find(|conn| conn.pin == pin)
    }

    pub fn connection_using(&self, peripheral: &str) -> Option<&Connection> {
        self.connects
            .iter()
            .find(|conn| conn.function.is(peripheral))
    }
}

impl App {
    pub fn modules_as_set(&self) -> HashSet<&str> {
        self.modules
            .iter()
            .map(|m| m.as_str())
            .collect::<HashSet<_>>()
    }
}
