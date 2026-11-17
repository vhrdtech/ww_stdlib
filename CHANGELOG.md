# ww_stdlib changelog

The standard API crates were developed inside the wire_weaver repository from 2024 (under `ww_stdlib/` there) and
moved to their own repository, vhrdtech/ww_stdlib, in November 2026 with their history, helpers, host tools, tests,
examples, board examples and docs pages. Each crate keeps its own `CHANGELOG.md`; this file summarizes changes to
the repository.

## Unreleased

### Breaking

- Repository: the API crates, `ww_bus_slave`, `ww_defmt_logger`, `ww_gpio_hl`, `ww_i2c_hl`, `ww_report`, `ww_pin_probe` and `ww_std_cli` live in vhrdtech/ww_stdlib; take them by git from there, not from wire_weaver.
- Layout: the crates sit at the repository root (`ww_gpio/`, not `ww_stdlib/ww_gpio/`) and their docs pages in `docs/` (assembled into the book's Standard library section).
- The board examples (`examples_mcu/mcu_qemu`, `nucleo_h743zi2`, `usb_stm32h725ig`, `rp2`) hold only the firmware that uses these APIs: `qemu-bus-slave`, `nucleo-h743zi2-all-gpio`, `stm32h725ig-all-gpio`, `stm32h725ig-uart` and `rp2_usb_log`; the framework benches of those boards stay in wire_weaver.

### Features

- Dependencies by git: `shrink_wrap` from romixlab/shrink_wrap, `exact_units` from romixlab/exact_units, the base crates from vhrdtech/ww_base and the framework from vhrdtech/wire_weaver; `Cargo.lock` pins each commit.
- `tests/stdlib_evolution` and `tests/global_ids` check the crates of this repository; the snapshots they refer to come from the checkouts `Cargo.lock` pins (TOOL-31, API-20).
- `tests/nucleo_g0b1re_fixture`: the frozen `nucleo_g0b1re_api` evolution fixture and its tests, from wire_weaver; `just refresh-fixture-pins` refreshes its pins (EVO-70).
- `just api-check` and `just save-snapshots` run `ww api check` and `ww api save` over every crate with `api_snapshots/`, through `ww_std_cli`'s never-installed `ww` example.

