# Changelog

## Unreleased

### Changed

- The tap test of global trait naming moved to vhrdtech/ww_tools with the tap decoder (TR-14); `ww monitor`, `stats`, `capture` are gone.

### 🚀 Features

- `ww dfu flash` runs `ww chunky flash` in process when the binary links `chunky_ww_cli`, else the `ww-chunky` plugin (TOOL-39).
- `ww board rework` and the `ww dfu` commands print a common error the device answered with by its text (STD-16).
- New crate (TOOL-36): `ww board`, `counters`, `dfu`, `pins`, `report` and the device parts of `info` and `sync`, moved from `wire_weaver_cli`, register themselves.

### Fixes

- `ww info` prints its API and hello lines once; they were printed twice.
- `ww board`, `counters`, `dfu`, `pins` and `report` follow `wire_weaver_cli` dropping `GlobalFlags::DeviceAndTap` with the link tap: they declare `GlobalFlags::Device`.
