commands-mcp [![.github/workflows/build.yml](https://github.com/LISTENAI/commands-mcp/actions/workflows/build.yml/badge.svg)](https://github.com/LISTENAI/commands-mcp/actions/workflows/build.yml)
============

![][rust-edition] ![][mcp-version] [![license][license-img]][license-url] [![issues][issues-img]][issues-url] [![stars][stars-img]][stars-url] [![commits][commits-img]][commits-url]

Build MCP Server with command template.

## Usage

Add the following to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "commands": {
      "command": "/path/to/commands-mcp",
      "args": [
        "/path/to/your/project"
      ]
    }
  }
}
```

## Example

A `commands.yaml` file should be placed in the root of your project. Here's an example:

```yaml
# yaml-language-server: $schema=https://listenai.github.io/commands-mcp/schema/master/commands.json

commands:
  zephyr_build:
    description: Build the specified Zephyr application
    args:
      - name: board
        description: |
          The board identifier to build for. If it can't be determined from the
          context, it should be prompted for.
        required: true
      - name: source_dir
        description: |
          Path to the source directory of the Zephyr application to build.
          Defaults to the current working directory.
        default: .
      - name: pristine
        description: |
          If true, the build directory will be cleaned before building.
        default: false
    command: |
      source .venv/bin/activate
      west build -b {{board}} -s {{source_dir}} {{#if pristine}}--pristine{{/if}}
```

For code completion on the `commands.yaml` file, [redhat.vscode-yaml](https://marketplace.visualstudio.com/items?itemName=redhat.vscode-yaml) extension is recommended.

## The `commands.yaml` file

* `commands`: The root key for command definitions.
  * `<tool>`: A tool named `<tool>`.
    * `description`: A description of what the command does.
    * `args`: A list of arguments for the command, if any.
      * `name`: The name of the argument.
      * `description`: A description of the argument.
      * `type`: The type of the argument (optional, accepts `string`, `number`, `boolean`, defaults to `string`).
      * `required`: Whether the argument is required (defaults to `false`).
      * `default`: The default value for the argument, if it is not required.
    * `command`: The command to run. Supports [Handlebars](https://handlebarsjs.com/guide/expressions.html) templating for arguments.
    * `shell`: The shell used to execute the command. Defaults to "bash" on Unix-like systems and "powershell" on Windows. Also supports "python" for using Python script in the command.
    * `venv`: Optional path to a Python virtual environment to use. If specified, the command will be executed with the specified venv activated.

### Python support

If the `shell` is set to `python`, the command will be executed using Python. If a virtual environment is specified in the `venv` field, it will be activated before executing the command.

```yaml
commands:
  some_python_script:
    description: A python script that does something
    shell: python
    venv: .venv
    command: |
      import sys
      print(f"Running Python {sys.version} in virtual environment {sys.prefix}")
```

## Built-in tools

In addition to the commands defined in `commands.yaml`, the MCP Server provides several built-in tools, which can be enabled in the `commands.yaml` file with corresponding configuration.

### Flash tools

Built-in tools for flashing firmware to LISTENAI devices.

```yaml
flash:
  enabled: true         # Enable flash tools
  chip: mars            # Chip model, can be '6', 'mars'
  baudrate: 1500000     # Baud rate for flashing, defaults to 1500000
```

### Serial tools

Built-in tools for reading logs from connected serial devices.

```yaml
serial:
  enabled: true         # Enable serial tools
  baudrate: 115200      # Baud rate for serial communication, defaults to 115200
  reset: dtr            # Method to reset the device before reading logs, can be
                        # 'dtr' or 'rts'. If not specified, no reset will be performed.
  reset_interval: 100   # Interval in milliseconds between the reset line is
                        # asserted and deasserted, defaults to 100ms.
```

### Schematic tools

Built-in tools to help the AI better understand the hardware schematic of the board.
For details about the schematic files, see the [Schematic](docs/schematic.md).

```yaml
schematic:
  enabled: true                 # Enable schematic tools
  board: csk6_duomotai_devkit   # Board name, should match the board file defined
                                # in `boards_dir`.
  socs_dir: schematic/socs      # Directory containing SoC-level schematic
                                # definitions. Defaults to "schematic/socs".
  boards_dir: schematic/boards  # Directory containing board-level schematic
                                # definitions. Defaults to "schematic/boards".
```

## License

[Apache-2.0](LICENSE)

[rust-edition]: https://img.shields.io/badge/rust-2024-black?style=flat-square
[mcp-version]: https://img.shields.io/badge/mcp-2024--02--02-orange?style=flat-square
[license-img]: https://img.shields.io/github/license/LISTENAI/commands-mcp?style=flat-square
[license-url]: LICENSE
[issues-img]: https://img.shields.io/github/issues/LISTENAI/commands-mcp?style=flat-square
[issues-url]: https://github.com/LISTENAI/commands-mcp/issues
[stars-img]: https://img.shields.io/github/stars/LISTENAI/commands-mcp?style=flat-square
[stars-url]: https://github.com/LISTENAI/commands-mcp/stargazers
[commits-img]: https://img.shields.io/github/last-commit/LISTENAI/commands-mcp?style=flat-square
[commits-url]: https://github.com/LISTENAI/commands-mcp/commits/master
