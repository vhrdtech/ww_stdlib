# ww_stdlib

[![CI](https://github.com/vhrdtech/ww_stdlib/actions/workflows/ci.yml/badge.svg)](https://github.com/vhrdtech/ww_stdlib/actions/workflows/ci.yml)

<p align="center">
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/vhrdtech/wire_weaver/master/docs/assets/wire_weaver_logo-dark.png">
  <img src="https://raw.githubusercontent.com/vhrdtech/wire_weaver/master/docs/assets/wire_weaver_logo.png" alt="WireWeaver logo" width="480"/>
</picture>
</p>

> The [WireWeaver](https://github.com/vhrdtech/wire_weaver) standard library: ready-made API traits and types for
> devices and the hosts that talk to them.

Instead of designing its own GPIO, logging or firmware update API, a device picks the crates it needs and puts their
traits into its API; a host (driver, CLI, GUI) gets a matching typed client from the same codegen, and generic tools
(`ww`, Python, the dynamic client) understand the device because they know the same traits.

## Crates

### Device contract

Every device implements these, so one host tool asks the same questions of any of them.

| Crate | What |
| --- | --- |
| `ww_firmware_info` | firmware name, version, build, why it last reset |
| `ww_board_info` | board name, revision, variant, BOM, serial, reworks |
| `ww_uid` | the unit's unique ids (chip id, MAC, ...) |
| `ww_user_info` | the label and notes a user gave the unit |
| `ww_counters` | the device's `cnt` counters, decoded on the host with the firmware's ELF |
| `ww_log` | the device's log: defmt frames or text records, as reliable streams |

### Device lifecycle

| Crate | What |
| --- | --- |
| `ww_dfu` | firmware update: A/B and one-slot schemes, inventory, staging, boot control |
| `ww_indication` | the device's indication LED |
| `ww_recorder` | control a recorder writing many tracks to an SD card or flash |

### IO

| Crate | What |
| --- | --- |
| `ww_gpio` | logical-pin GPIO: read, drive, reconfigure, watch edges |
| `ww_uart` | bridge a serial port, watch a link |
| `ww_can_bus` | CAN and CAN FD controllers, groups, TTCAN, CAN XL |
| `ww_i2c`, `ww_spi` | master transfers, bus sniffing, slave data paths |
| `ww_onewire` | 1-Wire transactions with strong pull-up |
| `ww_adc`, `ww_pwm` | analog inputs and streams; PWM outputs and capture |
| `ww_logic_analyzer`, `ww_signal_generator` | triggered captures; pattern and waveform generation |
| `ww_memory` | 8- and 32-bit memory and register access, with watches |
| `ww_pin_diag` | a board's digital points for bring-up and production tests |
| `ww_block`, `ww_sd` | block devices (SD card, eMMC, flash); raw SD card access |
| `ww_interned` | strings sent as an index into the firmware's string table |

### Helpers and tools

| Crate | What |
| --- | --- |
| `ww_bus_slave` | device side of the I2C and SPI slave data paths |
| `ww_defmt_logger` | a defmt global logger feeding `ww_log` |
| `ww_gpio_hl`, `ww_i2c_hl` | typed host drivers over the GPIO and I2C traits (embedded-hal for I2C) |
| `ww_report`, `ww_pin_probe` | the device report and pin probing libraries |
| `ww_std_cli` | the `ww` commands of these APIs: board, counters, dfu, pins, report, ... |

The base types these APIs use (`ww_si` with `exact_units`, `ww_numeric`, `ww_version`, `ww_date_time`, `ww_error`)
and the id ledger (`ww_global`) live in [ww_base](https://github.com/vhrdtech/ww_base); the framework (codegen, the
runtime, the client, `ww_timebase`, `ww_client_server`) in [wire_weaver](https://github.com/vhrdtech/wire_weaver).

## Using a crate

```toml
[dependencies]
ww_gpio = { git = "https://github.com/vhrdtech/ww_stdlib", default-features = false }
ww_firmware_info = { git = "https://github.com/vhrdtech/ww_stdlib", default-features = false }
```

API crates are `no_std` without their default `std` feature. A firmware takes them with `default-features = false`,
a host with the defaults.

## Documentation

The pages in [`docs/`](docs) describe each crate: what its traits are for, how a device implements them and how a
host uses them. They are published as the Standard library section of the
[WireWeaver book](https://ww.vhrd.tech/std_library/overview/), which assembles them from this repository, so their
links to other parts of the book work there. `FEATURES.md` tracks what is done and what is planned.

## Repository layout

| Path | What |
| --- | --- |
| `ww_*/` | the API crates, helpers and tools |
| `tests/` | an integration-test crate per API with its fixture API crate; the evolution and id checks |
| `examples/` | all_gpio, defmt_log and uart: an API crate and its server and client |
| `examples_mcu/` | board firmware using these APIs |
| `docs/` | the pages of these crates |

`just --list` shows the recipes: `just test`, `just lint`, `just check`, `just save-snapshots`.

## License

MIT, see [LICENSE](LICENSE).
