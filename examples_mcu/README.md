# Board examples

Firmware that puts the standard library's APIs into a device's own API. Each board is a Cargo workspace of its own,
on the nightly toolchain its `rust-toolchain.toml` pins, with its target and runner in `.cargo/config.toml`.

| Board | Target | Binary | What it shows |
| --- | --- | --- | --- |
| `mcu_qemu` (QEMU lm3s6965evb, Cortex-M3) | `thumbv7m-none-eabi` | `qemu-bus-slave` | `ww_bus_slave`'s I2C and SPI slave data paths, no hardware needed |
| `nucleo_h743zi2` (Nucleo H743ZI2) | `thumbv7em-none-eabihf` | `nucleo-h743zi2-all-gpio` | every pin through `ww_gpio`; edge events with `--features streams` |
| `usb_stm32h725ig` (STM32H725IG, USB ULPI) | `thumbv7em-none-eabihf` | `stm32h725ig-all-gpio`, `stm32h725ig-uart` | `ww_gpio`, and a `ww_uart` serial bridge; streams with `--features streams` |
| `rp2` (RP2350) | `thumbv8m.main-none-eabihf` | `rp2_usb_log` (`--features usb_log`) | defmt logs over USB through `ww_defmt_logger` and `ww_log`, and RTT at once |

## Build and run

```sh
just check                      # from the repository root: cargo check of every board and feature set
cd examples_mcu/mcu_qemu && cargo run --release --bin qemu-bus-slave    # needs qemu-system-arm
cd examples_mcu/nucleo_h743zi2 && cargo run --release --bin nucleo-h743zi2-all-gpio
cd examples_mcu/rp2 && just run rp2 usb_log
```

The hardware boards flash through the runner in their `.cargo/config.toml`; change it to the flashing tool at hand
(`probe-rs run --chip ...`). `nucleo_h743zi2` and `usb_stm32h725ig` have their own README with the board details.

The framework's own firmware for these boards (size and cycle budgets, framework-only examples) lives in WireWeaver's
`examples_mcu/`.
