Schematic files
===============

The [Schematic tools](https://github.com/LISTENAI/commands-mcp#schematic-tools)
provide a way to query pinout information for supported boards and SoCs. The
schematic information is provided by series of YAML files in the project directory.

## The SoC file

The SoC file defines the pin mapping for a specific System-on-Chip (SoC), generally
located at `schematic/socs/<soc_name>.yaml`.

### Example SoC file

```yaml
# yaml-language-server: $schema=http://listenai.github.io/commands-mcp/schema/master/schematic-soc.json
# AI assistance: Use schematic_* tools instead of directly reading this file.

name: csk6011a
description: LISTENAI CSK6011A SoC

pins:
  - name: GPIOA_02
    pinmux:
      - gpio
      - star_swd.clk
      - uart0.rxd
      - uart1.cts
      - ir.datout
      - ir.datin
      - spi0.mosi
      - spi1.miso
      - i2c0.scl
      - i2c1.sda
      - gpt.trigger_2
      - gpt.pwm_2
      - gpt.pause
      - sdio.dat1
      - i2s1.bck
      - i2s2.lrck
      - dmic23.clk
      - dvp.d2
  - name: GPIOA_03
    pinmux:
      - gpio
      - star_swd.io
      - uart0.txd
      - uart1.rts
      - ir.datin
      - ir.datout
      - spi0.clk
      - spi1.mosi
      - i2c0.sda
      - i2c1.scl
      - gpt.trigger_3
      - gpt.pwm_3
      - gpt.clk_t1
      - sdio.clk
      - i2s1.lrck
      - i2s2.adcdat
      - dmic23.dat
      - dvp.d3
```

#### Fields

- `name`: The name of the SoC.
- `description`: A brief description of the SoC.
- `pins`: A list of pins available on the SoC, each with:
  - `name`: Name of the pin, typically in the format `GPIOA_xx` or `PBx`
  - `pinmux`: A list of functions that the pin can perform, in the format of `peripheral.signal` (e.g., `gpio`, `uart0.rxd`, `spi1.mosi`).

## The board file

The board file defines the pin mapping for a specific board, generally located
at `schematic/boards/<board_name>.yaml`.

### Example board file

```yaml
# yaml-language-server: $schema=http://listenai.github.io/commands-mcp/schema/master/schematic-board.json
# AI assistance: Use schematic_* tools instead of directly reading this file.

name: csk6_duomotai_devkit
description: LISTENAI CSK6-MIX DevKit
soc: csk6011a

devices:
  - name: usb
    connects:
      - GPIOB_10@usb.dp
      - GPIOB_11@usb.dm
  - name: key_k3
    connects:
      - GPIOB_00@gpio
  - name: key_boot
    connects:
      - GPIOB_01@gpio
  - name: debug
    connects:
      - GPIOA_00@star_swd.clk
      - GPIOA_01@star_swd.io
      - GPIOA_02@uart0.rxd
      - GPIOA_03@uart0.txd
      - GPIOA_07@uart3.txd
  - name: display_lcd
    connects:
      - GPIOA_15@spi1.mosi
      - GPIOA_16@spi1.clk
      - GPIOA_17@spi1.cs
      - GPIOA_18@gpio
      - expander:CH_PD2@pwm
  - name: expander
    connects:
      - GPIOB_08@i2c1.sda
      - GPIOB_09@i2c1.scl
    pins:
      - name: CH_PD2
        pinmux: [gpio, adc_3, pwm]

exposes:
  - name: CN1
    pins:
      - GPIOA_04
      - GPIOA_05
      - GPIOA_06
      - GPIOA_10
      - GPIOA_19
```

#### Fields

- `name`: The name of the board.
- `description`: A brief description of the board.
- `soc`: The name of the SoC used on the board, which should match a SoC file in `schematic/socs`.
- `devices`: A list of devices on the board, each with:
  - `name`: The name of the device.
  - `connects`: A list of pin connections in the format `pin@function`, where `pin` is the name of pin on the SoC and `function` is the function it serves (e.g., `gpio`, `uart0.rxd`).
  - `pins`: For devices like GPIO expanders that have additional pins, this field lists the pins exposed by the device, each with:
    - `name`: The name of the pin.
    - `pinmux`: A list of functions that the pin can perform.
- `exposes`: A list of exposed connectors on the board, each with:
  - `name`: The name of the connector.
  - `pins`: A list of pin names that are part of the connector, which can be used to identify the pinout of the connector.

## The app file

The app file declares what devices the current app is using, located at root of
the app directory named `schematic.yaml`.

### Example app file

```yaml
# yaml-language-server: $schema=http://listenai.github.io/commands-mcp/schema/master/schematic-app.json
# AI assistance: Use schematic_* tools instead of directly reading this file.

name: my_app

devices:
  - usb
  - debug
  - expander
```

#### Fields

- `name`: The name of the application.
- `devices`: A list of devices used by the application, which should match the names defined in the board file's `devices` section.
