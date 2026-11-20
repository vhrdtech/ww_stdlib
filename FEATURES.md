# ww_stdlib features and roadmap

This file is the single source of truth for what the WireWeaver standard library supports, what is being worked on
and what is planned, for humans and AI agents alike.

## How to use this file

- **Status** of each item:
  - ✅ done
  - 🚧 in progress or partially done (the note says what is missing)
  - 📋 planned
  - 💡 idea, not committed to
  - ⛔ blocked (the note says on what)
  - 🔍 probably done or obsolete, needs a check before closing
  - 🧪 done and tested on the host, the check on hardware is still open (the note says what to check)
  - 🐛 known bug, not yet fixed (the note says how to hit it)
- **Target** is the release an item is planned for (`v0.5` is WireWeaver's next release, which these items were
  planned with; `v0.5.x` the next compatible release after it). No tag means not scheduled.
- **IDs** (`STD-12`, `TOOL-28`) are stable: never renumber or reuse one. New items take the next free number of
  their area. Use the ID in commit messages, CHANGELOG entries and code `TODO`s (`// TODO(STD-20): ...`). When
  talking to people, name an ID with a short slug from its title (`STD-12 ww-dfu-api`), never bare.
- The items came from WireWeaver's `FEATURES.md` with their IDs, where these crates were developed: every item whose
  code lies wholly in this repository's crates. IDs not listed here (`SW`, `DER`, `API`, `SRV`, `CLT`, `EVO`, `TOOL`,
  `TEST`, ... and other `STD` items, such as the base types' and `ww_timebase`'s) are
  [wire_weaver](https://github.com/vhrdtech/wire_weaver)'s or [ww_base](https://github.com/vhrdtech/ww_base)'s, and
  so are paths outside this repository's crates (`wire_weaver_cli`, `wire_weaver_client`, `docs/api/..`). Paths in
  the items name the old layout where they were written: `ww_stdlib/<crate>` is `<crate>/` here,
  `docs/std_library/<page>` is `docs/<page>`, and the `ww` commands of `wire_weaver_cli/src/cmd/` are in `ww_std_cli`.
- Items are grouped by area. Each area lists what works first (✅), then open items.
- When finishing work, update the item in the same change: mark it ✅, move it up to the done items of its area and
  add a pointer (docs page, crate, test or example). Don't delete done items. Move items that turn out to be obsolete
  to [Dropped and superseded](#dropped-and-superseded) with a one-line reason.
- New ideas go into the matching area as 💡. Small code-level gaps stay as `TODO` comments in the source; only those
  that limit users or block a feature get an item here.

## Standard APIs (`STD`)

- ✅ **STD-5 `ww_gpio`**.
- ✅ **STD-19 `ww_pwm`** — `Pwm` / `PwmChannel` outputs (`set_output`, `set_duty`, `stop`, `state`), `PwmCapture` /
  `PwmCaptureChannel` measurements (frequency, duty, pulse width, edge count) over a plain stream with sideband
  `Open` / `Close`. Full rustdoc, mock server and in-process tests (`tests/io_pwm`), docs page
  (`docs/std_library/ww_pwm.md`) (STD-13).
- ✅ **STD-18 `ww_adc`** — `Adc`, `AdcStream` (sideband `Open` / `Close`), `AdcCapture`. Fill-in done:
  full rustdoc, `tests/io_adc` mock server (one test per trait), `docs/std_library/ww_adc.md`, API snapshot
  (STD-13).
- ✅ **STD-20 `ww_onewire`** — `OneWire`: addressed transactions with strong pull-up, reset, paged and
  bit-level (`triplet`) ROM search, the real Dallas/Maxim search algorithm. Full rustdoc, docs page
  (`docs/std_library/ww_onewire.md`), API snapshot, in-process test with a mock bus (`tests/io_onewire`).
- ✅ **STD-114 `ww_can_bus` typed numbers** · `v0.5` — the CAN traits' bare integers as exact_units types through `ww_si` (AGENTS.md "Numbers are typed", the stdlib units survey): bit timing segments and limits as `Fixed<TimeQuantum, One, _>`, the prescaler as `Prescaler` (`Per<Cycle, TimeQuantum>`), TDC offsets, filter, measured delay and XL PWM symbols as `Fixed<Cycle, One, u8>` (clock periods), bus load and sample point as `Share` (‰, `Range<0, 1000>`), frame, buffer, filter, FIFO, event and word counts as counted `Fixed`s, buffer, FIFO and filter positions as `BufferIndex`, `FifoIndex`, `FilterIndex` (one type for `RxFrame::filter` and `TriggerKind::RxCheck::filter`), TTCAN cycles as bounded `Fixed` / `Index<Cycle, ..>` and its time marks as `Index<Tick, u16>`, `Performance` rates as `Per<Frame, Second>` / `Per<Byte, Second>`, `TxMarker` and `VcId` newtypes, `TtErrorLevel` for the TTCAN error level, `TtRole::Master::priority` a `U3`; `BitTiming::bit_rate` takes the `Hertz` of `Capabilities::clock`. Raw protocol fields (TEC / REC, `TURCF`, the XL SDT and acceptance field) stay plain with `#[no_unit_hint]`. Wire change EVO-62. Tests `tests/io_can`, `tests/io_can_group`. [Docs](docs/ww_can_bus.md#configuration).
- ✅ **STD-113 `ww_onewire` / `ww_uart` typed numbers** · `v0.5` — the stdlib units survey's (P2401#11nov-0948f)
  ww_onewire/ww_uart piece of AGENTS.md "Numbers are typed": `ww_onewire`'s `search`'s `family: Option<u8>` is
  `Option<FamilyCode>` (`FamilyCode(pub u8)`, a transparent newtype, same bytes), `OneWireTransaction::read_len:
  u16` is `ww_si::Bytes<u16>` (same bytes), `OneWireConfig::pin: u8` is `ww_si::Index<ww_si::units::Pin, u8>` (same
  bytes), and `SearchState::last_discrepancy: u8` is `ww_si::IndexFrom1<RomBit, u8, Range<1, 64>>` (`RomBit` a
  counted marker of this crate, declared with `unit!`): the added `1..=64` bound is a wire break, EVO-61, on a new
  wire line `w1.0` -> `w2.0`. `ww_uart`'s `tx_pending()`, `Capabilities::rx_buffer` /
  `tx_buffer: u32` are `ww_si::Bytes<u32>`, `UartConfig` / `MonitorConfig::idle_chars: u8` are
  `ww_si::Count<ww_si::units::Char, u8>`, `UartConfig::frame_bits()` / `MonitorConfig::frame_bits()` return
  `Option<ww_si::Fixed<ww_si::units::Bit, ww_si::prefix::One, u32>>`: all same bytes, no wire bump. `strong_pullup`, `send_break`'s durations and `io_level` (both crates)
  stay `Quantity` for now (P2401 Q2, a separate cross-cutting change); IO item timestamps (`RxChunk::end`) stay
  `Second` (Q1, with the framework device-timestamp change). `ww_onewire_0_1_0.ron` / `ww_uart_0_1_0.ron` are saved again
  in place (never published). `tests/stdlib_evolution` checks both.
- ✅ **STD-24 `ww_firmware_info` / `ww_board_info` / `ww_uid` / `ww_user_info` device contract** · `v0.5` — the info
    crates every device implements. `FirmwareInfo` const items `FIRMWARE` (name, version, git, build time, profile),
    `BUILD_ID` (GNU build id, finds the ELF) and optional `BUILD_RECORD`; `build::emit()` + `firmware_summary!()` for
    `build.rs`, `build_id()` reader; `LastReset` (`ResetRecord` with a `Fault` per architecture behind `cortex-m` /
    `riscv`, stack words, `boot_count`, `clear`). `BoardInfo::BOARD` with `RevisionSource` (`Record`, `Strap`, `Assumed`, `Provisional`) and optional `REWORKS`
    (`Rework { id, title, applied }`, up to 16, numbered since STD-98; `ww info` prints them). `Uid::IDS`, first id is
    the primary one (USB serial). `UserInfo` with bounded `name` / optional `notes`, `UserInfoError`, `CAPABILITIES`
    and the persistence rule. Docs `docs/std_library/ww_{firmware_info,board_info,uid,user_info}.md`, tests
    `tests/{firmware_info,board_info,uid,user_info}`. The const items are served by `get_` handlers until constant
    items come from the introspection data; properties cannot carry `#[errors]` yet.
- ✅ **STD-98 Recording a board rework** · `v0.5` — `ww_board_info` reworks are numbers, `ReworkId(u16)` per board
    revision (Roman, 2 Nov 2026), so a record keeps a bit per number and the titles stay in the firmware.
    `Rework.title: Option<..>`, `None` for an applied number the firmware doesn't know (written by a newer one: since
    STD-101 the board refuses to boot unless an allowance for its image covers it). Optional `BoardInfo::set_reworks(changes, program)`: all changes or none, dry
    run, `ReworkChange` (`Unchanged`, `WouldWrite`, `Written` from the next boot), `ReworkError` (`UnknownRework`, never
    stored; `Conflict`; `ProductionRecord`). `ww_board_info::table` (`no_std`) answers both from the
    firmware's table (`ReworkTable`, `ReworkSet`, `apply`, `list`, `unknown`, `known_ids`). `ww info` prints unknown
    ones. Docs `docs/std_library/ww_board_info.md#recording-a-rework`, tests `tests/board_info`, `tests/device_report`.
    The `ww board rework` command is TOOL-28; the first device is usb_io_fw PLAT-17.
- ✅ **STD-25 `ww_counters` on `cnt` words** · `v0.5` — the device's `cnt` counters as raw buffer words, decoded on
    the host with the firmware's ELF found by its build id: `Counters::INFO` (format, used/capacity words of RAM and
    backup, build id), `read(buffer, offset)` pages of `UVlq32`, `reset`, optional `watch` stream (not served by
    anything yet); `read_words` / `reset_words` serve `cnt` buffers, feature `host` has `WwSource` (a
    `cnt_core::Source`). `ww counters read|watch|tui|reset|info` shows the `cnt_view` views (CLT-10). In the `cnt`
    repo: `cnt::ram_used_words()`, word sources and ELF lookup by build id in `cnt_core`, `cnt_view`, and the probe
    path checks the build id too. Docs `docs/std_library/ww_counters.md`, tests `tests/counters`; `cnt_core` /
    `cnt_view` 0.5 from crates.io.
- ✅ **STD-27 `reset-record-age`** · `v0.5` — `ww_firmware_info::ResetRecord::boots_since_crash`: boots since the
    crash data was saved (0 = the last reset was the crash), so a record kept across later normal resets shows how old
    it is. `bedrock_crash` counts it (`Crash::boots_since`) and the firmware template fills it.
- ✅ **STD-28 `ww_indication` lightweight rework** — cut to one required item: `mode: Mode` (`AllOff` /
    `OnlyEssential` / `Normal`). `brightness: Brightness` (5 levels, one setting for all indicators),
    `identify(duration)` (blink everything to find the unit on a bench, errors on `CommonError`) and `NAMES`
    (indicator names, API-36) are each their own `#[optional]` item. Dropped `alert_brightness` /
    `AlertBrightness`, `animation_enable`, `test_mode` / `TestMode` and `HAS_NOT_CONTROLLABLE`. Docs
    `docs/std_library/ww_indication.md`, tests `tests/indication` (mock servers with only `mode`, and with every
    optional item).
- ✅ **STD-29 `ww_can_bus` on the stdlib idioms** — from simple bridges to bus debuggers. `Can` is one controller:
  six required items (`capabilities()`, `configure(CanConfig)` answering what was set, `config()`, the observable
  `status`, the counted `rx` stream, the `tx` credit sink) and `#[optional]` `set_mode` (incl. `Config` and
  `Sleep`), `recover`, `tx_abort`, `tx_events` (marker, measured queue time), `bus_errors`, `protocol_status`,
  `bus_load`, `detect_timing`, `set_filters`, `set_layout`, `rx_buffer`, `tx_buffer` (once or cyclic),
  transceiver (standby / silent / sleep, wiring faults), terminator, `test_pins`. `CanGroup` (`interface[]: Can`,
  enable / disable, optional clock source and frequency, performance) for controllers sharing clock, power and
  message RAM; `CanTimeTriggered` (TTCAN); `CanXl` behind the `xl` feature. Errors on `CommonError` +
  `OutOfFilters`, `OutOfMemory`, `BusOff`; `defmt` no longer pulls `std`. Docs `docs/std_library/ww_can_bus.md`,
  tests `tests/io_can` (full and minimal node), `tests/io_can_group` (group, TTCAN, XL).
- 🧪 **STD-26 `ww_log` defmt over reliable streams** · `v0.5` — `ww_log_bare_metal` renamed `ww_log`.
    `LogDefmt { const DEFMT (encoding, build id); frames() }`: a reliable stream whose unit is one `defmt` frame,
    whole frames per item (up to 512 B), so `lost` is the exact number of dropped messages and logs from before the
    host connects arrive by resuming the device's session. `LogText { records(); rw level: LevelFilter }` with
    bounded `LogRecord`, `Severity` without `User`, `LogError` on `CommonError`. Device side: new crate
    `ww_defmt_logger` (`global_logger!`, lock-free ring with whole-frame commits, RTT tee that never blocks (whole
    frames or skipped and counted, whatever mode a probe set), `pump()` into the store,
    drops reported in place), ring stress-tested on the host with concurrent producers. `tests/log_defmt` decodes
    with `defmt-decoder` end to end; `examples/defmt_log` (host) and `examples_mcu/rp2`'s `rp2_usb_log` (USB and RTT
    at once, build id in flash). `docs/std_library/ww_log.md`. Tested 16 Oct 2026 on Portwright B153A (STM32G0B1,
    typec_dongle API-10, with a `PersistentRamRing`): logs over USB and RTT at once, `lost` exact under a flood of
    1000 (227 + 773), logs from before connecting and from before a forced HardFault; the non-blocking RTT tee
    with probe-rs attached (it sets the channel to blocking): floods of 1000 go on at ~62 us a line instead of
    ~620 us, RTT shows what fits. Known gap: frames dropped at the end of a burst are reported (`lost`) only with
    the next frame the device logs, so a quiet device shows the loss late. `rp2_usb_log` not run (no Pico 2 probe
    on the mesh). `ww log` in the CLI is a later item.
- 🧪 **STD-50 `ww_indication` animations in phase** — a device with a shared clock (STD-49 or a sync line) aligns
  its periodic animations to period boundaries of shared time (UTC, WSPR-style: `t mod P`), so devices showing the
  same animation are in phase with no epoch passed around, across PCs to NTP precision. A rule in the trait's docs,
  no API item (an explicit epoch was dropped: every server would have had to implement or omit it).
  `docs/std_library/ww_indication.md#animations-in-phase-std-50`; device side embedded_bedrock `bedrock_led` with
  `bedrock_sync`. Hardware check open: Portwright and rePort breathing in phase.
- 🧪 **STD-35 Slave data paths in `ww_i2c` and `ww_spi`** · `v0.5.x` — an `#[optional(slave)]` group in `I2c` and
  in `Spi`: opening the counted `slave_events` / `slave_frames` stream with a config starts slave mode (closing stops
  it) and reports every transfer (bytes both ways, register pointer or address, fill bytes, stretch time); the
  answers come from a register map the host fills and reads back (`slave_map_write` / `slave_map_read`, applied
  between two transfers) or from the `slave_answers` credit sink, one answer per read or frame. A host that is too
  slow: I2C `I2cSlaveUnderrun` (`Fill`, `Stretch { max }` with `ReadWaiting` sent at once, `Nack` while empty), SPI
  the idle byte or the last answer again, flagged `underrun`. Limits of clock stretching (SMBus 25 ms, masters
  without it) in the crate docs and `docs/std_library/ww_i2c.md#slave-mode`. `I2cMode::Slave`, `SpiRole::Slave` and
  the `slave` capability flags are gone. API, snapshots, docs, the device-side engine (SRV-26, `ww_bus_slave`) and an
  end-to-end test of both slaves done: `tests/io_i2c` / `tests/io_spi` drive `I2cSlave` / `SpiSlave` byte by byte as a
  simulated master (test-only `slave_bus` / `slave_pump` methods, a fake clock) through the real host and device event
  loops: register maps with page wrap and host updates between transactions, the answer queues, stretching to the
  deadline and past it, `Nack`, `repeat_last`, cleared answers and dropped events. Open: a check on real silicon, with
  the first user, the rePort (usb_io_fw IF-5, v1.0 launch, Roman 16 Oct 2026).
- 🧪 **STD-52 `ww_i2c` master errors and limits** · `v0.5` — what a good I2C master knows reaches the host (found
  by the rePort I2C driver): `I2cError::Nack { op, at }` (which operation, and the address
  or which data byte, counted as `I2cRecord::nack_at`), `ClockStretch` with `I2cMode::Master`'s `stretch_timeout`
  (set and answered by `i2c_configure`, range in the capabilities), `BusStuck { scl_low, sda_low }` (no silent recovery:
  `i2c_wiggle_clock` on request, at the bus speed up to 100 kHz), and the limits in `I2cCapabilities` (`speeds`,
  `max_ops`, `max_write_read_len`, `zero_length_write`, `stretch_timeout` range; grouped by topic, sizes all `u16`). Transactions are stated as
  a repeated start before every operation (Linux `I2C_RDWR`), not embedded-hal merging; `i2c_configure` answers the
  highest SCL not above the request. Breaking on the wire and in Rust (nothing released or deployed, user 27 Oct
  2026): callers in usb_io_fw and typec_dongle adapt. Docs
  `docs/std_library/ww_i2c.md#transactions`, tests in `ww_i2c` (round trips) and `tests/io_i2c`. Open:
  the check on real silicon with the rePort driver (usb_io_fw IF-2, RP2350 DW_apb_i2c).
- 🧪 **STD-76 `ww_i2c_hl` embedded-hal I2C over `ww_i2c`** · `v0.5` — a stock embedded-hal 1.0 I2C driver,
    blocking or async, runs unchanged on the host against a remote `ww_i2c::I2c` (for a FUSB302 PD sink
    driven from the PC): `I2cMaster` merges contiguous same-direction operations (a repeated start before every
    `ww_i2c` operation, STD-52), so one embedded-hal transaction is one call and one round trip; refuses, never
    splits, what the device's `I2cCapabilities` don't take; `Nack`/`Arbitration`/`Bus`/`Overrun` mapped to
    `ErrorKind`; request priority as a handle setting (`urgent()`, CLT-50). Docs `docs/std_library/ww_i2c_hl.md`,
    tests `tests/io_i2c` (`tmp1x2` async, `eeprom24x` blocking, unchanged). Open: the check on real silicon with
    the FUSB302 over the rePort's I2C.
- 🚧 **STD-6 `ww_gpio_hl`** — usable, some APIs are still missing. Blocking, async and promise flavors; the blocking
    pins implement `embedded_hal::digital` (`OutputPin`, `StatefulOutputPin`, `InputPin`), the async ones
    `embedded_hal_async::digital::Wait` without a race (stream opened, then level read, then edge awaited) and keep
    the edge's device time (`last_edge_time`), tested in `tests/io_gpio` (`docs/std_library/ww_gpio.md#embedded-hal-ww_gpio_hl`).
- 🔍 **STD-7 Interrupt left enabled in `ww_gpio_hl`** · `v0.5` — the falling edge helper documents that it leaves
  interrupts enabled; decide whether that is the intended fix or a bug.
- 🚧 **STD-8 Prototypes**, to be completed later: `ww_i2c`, `ww_uart`,
  `ww_memory` (was `ww_ll`: one word type per trait, optional watches, errors on `CommonError`, docs page),
  Known gaps are `TODO`s in their sources (e.g. SMBus and 7-bit addresses in `ww_i2c`; `ww_can_bus` is done in
  STD-29). `ww_log` (was `ww_log_bare_metal`) left for STD-26.
- 🚧 **STD-12 `ww_dfu` firmware update API** — traits `UpdateNode`, `BootControl`, `Inventory`, `Stage`,
  `StageBulk`: each node (app, bootloader, recovery app) implements what it needs; A/B live, in place from a stored
  plan, in place live; step by step or bulk. Carries the octet records of the chunky protocol spec. End to
  end test `tests/dfu`. Guide with the eight update scenarios (`2C-APP-STEP` Swap Shuttle .. `1L-UPD-BULK` Pit-stop
  Freight), each tested end to end over USB (simulated), WebSocket or UDP with reboots and power cuts, its exchange
  recorded as a golden transcript the docs include (`docs/std_library/ww_dfu.md`, `tests/dfu_scenarios`,
  `just dfu-transcripts`). `StageBulk::plan` is a credit sink (API-28 credit-sinks). Every variable-size field is bounded (SW-19:
  `MAX_PIECE_LEN`, `MAX_HEADER_LEN`, `MAX_CHUNKS_PAGE`, `MAX_PRODUCT_LEN`), so every method has a known largest
  request and reply. `Profile::AB(SlotSwitch::{Copy, Remap})` / `InPlace`. Its numbers are typed (EVO-65): steps,
  chunks and erase blocks as `Index`/`Count` of `Step`, `Chunk`, `Block`, byte offsets and sizes as `Index<Byte, ..>`
  and `Bytes`, ids and versions as newtypes, the anti-rollback floor a `SecurityCounter` apart from the release
  `ImageVersion`. Next: a chunky implementation (chunky NET-1).
- 🚧 **STD-13 IO trait family** · `v0.5` — `ww_gpio`, `ww_uart` (optional `Uart::monitor`), `ww_i2c` (+ `I2cWatch`,
  `I2cMonitor`, 10-bit addresses), `ww_spi` (optional `Spi::monitor`), `ww_adc` (STD-18), `ww_pwm` (STD-19),
  `ww_onewire` (STD-20), `ww_timebase` (STD-17), `ww_logic_analyzer` / `ww_signal_generator` (STD-21, STD-22);
  `ww_si` values and timestamps (STD-23): frequencies `Hertz`, baud rates `Baud`. Family overview in
  `docs/std_library/io_family.md`; every crate filled in with full rustdoc, a mock-server test pair `tests/io_<x>`
  and a docs page `docs/std_library/ww_<x>.md`; `examples_mcu` GPIO and UART boards follow. A bus or bank is
  configured with one `configure(XConfig)` answering what was actually set, the IO level inside it
  (`io_level: Option<Volt>`, capabilities `io_levels` / `io_level_min` / `io_level_max`). Every crate's error enum
  keeps only what is specific (`Nack`, `NoPresence`, `NoSuchChannel`, ...), the common errors come beside it
  (STD-16), methods list their variants with `#[errors(..)]` (API-32). Streams with a configuration are
  counted stream functions (API-31: `AdcStream::samples`, `LogicStream::data`, `Pin::event`,
  `PwmCaptureChannel::capture`, `Spi::monitor`, `I2cMonitor::records`, `Uart::monitor`), `Uart::rx` and
  `I2cWatch::changes` are counted: positions, device losses and the time of the first unit in the event head; UART
  chunks end on line idle (`UartConfig::idle_chars`, one back-to-back burst each) and carry the time of their last
  byte when the device measures it (`RxChunk::end`). Clients take one `?` per call (CLT-24
  `CallError`), `Unsized` enums read newer variants as `Unknown(u32)` (SW-47). Review round 1 (15 Oct 2026) done.
  Open: `*_hl` crates are STD-14.
- 🚧 **STD-21 `ww_logic_analyzer`** · `v0.5.x` — `LogicAnalyzer` (triggered capture into RAM), `LogicStream`;
  `Raw` and per-channel `Transitions` encodings (edges as a `Series<u32>`), staged pattern triggers and protocol
  stages (UART byte, I2C address, SPI first byte). Skeleton written, docs
  page `docs/std_library/ww_logic_analyzer.md`; mock-server test pair still open (STD-13).
- 🚧 **STD-22 `ww_signal_generator`** · `v0.5.x` — `PatternGenerator` (digital, replays analyzer captures through
  a credit sink), `WaveformGenerator` (DAC channels). Skeleton written, docs page
  `docs/std_library/ww_signal_generator.md`; mock-server test pair still open (STD-13).
- ⛔ **STD-31 Observable `ww_gpio` pin level** · `v0.5.x` — `property!(ro+observe level: Level)` on `Pin` at one of its
  reserved indices 4..=6, so a host sees input changes without opening the `event` stream of every pin (slow inputs,
  a dashboard). `Pin` sits in `Bank::pin[]` (and banks usually in an array too): levels there are observable per
  element since API-63, but a pin's change is seen in an interrupt, so it is blocked on SRV-59
  element-marks-from-tasks.
- 🚧 **STD-34 `ww_recorder`** · `v0.5.x` — control a recorder over WireWeaver: `Recorder` (status, card facts,
  two-step prepare/format with a token, recordings, markers, `sync`, settings with the flush thresholds, counters,
  optional `refresh_usb`) and `Track[]` (info, enable, durability, compression, counters). For the SD card log store
  of repo `logring`, usable by any recorder. API crate, snapshot and docs page
  `docs/std_library/ww_recorder.md` done, the scaffold compiles; the first server is the logring's (rePort
  integration).
- 🚧 **STD-39 `ww_block`** · `v0.5.x` — a block device over WireWeaver (SD card, eMMC, flash): `BlockDevice` with
  `geometry` (block size and count, blocks per call, erase unit, erased value), free `read` and device-side `crc`
  (CRC-32, `ww_block::crc32`), `claim` / `release` of a write lease that stops the card's own user, `write` with a
  CRC and `erase` under it, and the optional `bulk` group: the `write_runs` credit sink acknowledged on the counted
  `written` stream, and the counted `read_runs` stream. Who may claim is the firmware's choice per link
  (`CommonError::Refused`). Tier T1 of logring's design (its PC `LinkMedium`, S2 prepare on the PC). API crate, snapshot,
  docs page `docs/std_library/ww_block.md` and the round trip `tests/block_sd` done, the scaffold compiles; the
  first server is the logring's. `crc32`/`crc32_update` are `ww_crc`'s `CRC32_ISO_HDLC` (TR-31), table size
  (none, `table-256`, `table-slice16`) picked by `ww_crc`'s cargo features.
- 🚧 **STD-40 `ww_sd`** · `v0.5.x` — raw SD card access over WireWeaver: `SdCard` with `capabilities`, `state`, the
  `detect` stream, raw `registers` (CID, CSD, OCR, RCA, SCR, SD Status), `reinit` with a per-step `InitReport`, and
  `command`: one raw command with an optional data phase, answering response bytes, `CardStatus` bits, the
  `Outcome` and timing. Commands are `Free`, need the `ww_block` lease, or are `Restricted` (CMD42, CMD27) unless
  the firmware builds with `raw-sd-unrestricted` (`ww_sd::check_access`). Tier T0 of logring's design. API crate, snapshot,
  docs page `docs/std_library/ww_sd.md` and the round trip `tests/block_sd` done, the scaffold compiles; a device
  needs a raw-command hook in its SD driver (not checked yet for embassy-stm32's `Sdmmc`). The docs page says how a
  device in SPI mode fills the answers ("In SPI mode", from rePort's `sd_proto` engine); rePort's simulator serves
  `SdCard` on a simulated card in SPI mode (usb_io_fw IF-8).
- 💡 **STD-10 Device state machine trait** like CANopen.
- 💡 **STD-41 Frame splitters for byte streams** — a no_std, sans-IO splitter (bytes in, frames out with their
  time and status: CRC good or bad, truncated, overrun), the same code on the device and the host. Kernels: idle
  gap (today's `RxChunk`), delimiter (`\n`, `\r\n`, any byte), fixed length, length field, sync byte + length +
  checksum, COBS, SLIP, Modbus RTU, NMEA; picked by the host when it opens the stream (`Uart::frames(cfg)`), no new
  firmware. Frames of a bounded length are what a batched stream can take, where a raw byte stream can't (no size
  bound); with API-51, each frame XOR'd with the last one of the same key (Modbus address, NMEA sentence, CAN ID).
- 💡 **STD-44 SPI slave data-ready line** — a GPIO the slave asserts while an answer is queued (or the register map
  changed), as most SPI slave protocols with a computed answer have (ESP32 handshake, nRF IRQ); the master reads only
  when it is set, so the answer queue needs no polling. Part of `SpiSlaveConfig`, the pin named by the board.
- 💡 **STD-45 Live registers in a slave's register map** — address ranges of the I2C or SPI register map whose reads
  the host answers (I2C stretching meanwhile) or that change on read (clear-on-read status, FIFO data registers),
  so a chip with a few such registers can be emulated without moving it all to the answer queue.
- 💡 **STD-46 More I2C slave addressing** — a second address or an address mask (one emulator answering for several
  chips, a 24C16's block bits), general call, SMBus PEC and SMBALERT#/ARA. RP2350's controller has one address only,
  so this is per-device capability.
- 🚧 **STD-47 `ww_pin_diag` pin tests** · `v0.5.x` — `PinDiag { const POINTS; set(point, PointState); read(point);
  release_all() }`: raw access to a board's digital points (MCU pins, shift register and expander outputs) for
  bring-up, verification and production tests; the firmware keeps its board's rules and answers `NotCapable`,
  `CommonError::InUse` or `Refused { rule }`. Done: crate, docs page `docs/std_library/ww_pin_diag.md`, test pair
  `tests/pin_diag` (mock board: short, external pull-up, input-only pin, interlocked outputs, held pin), snapshot;
  the host tests and plan files are TOOL-15 (`ww pins test`). Missing: a device implementing it (rePort).
- 🚧 **STD-48 `ww_interned` strings as ids** · `v0.5.x` — `Interned { Id(Index<InternedStr, UNib32>), Text(&str) }` for names and reasons;
  ids resolved on the host from the firmware's ELF by build id. Done: the type, `Display` (`#17` for an unresolved
  id), `ww pins` finds the ELF by build id (`ww_pin_probe::device::Names`). Missing: an accessor for
  `defmt::Str`'s index (upstream, `Table::get_str`), reading the ids from the ELF's table with it.
- 📋 **STD-54 `ww counters` against the rePort** — `ww counters` fails against the B200A: the firmware sends no build
  id, and its `cnt` format is v0 while the tool reads v2 (report-i2c-master bench run, 27 Oct 2026). Either side may
  be behind (STD-25): bring the firmware's `cnt` and build id up to what the tool needs, or make the tool say which
  format it got and what is missing instead of failing.

## Server / device side (`SRV`)

- ✅ **SRV-26 Device-side engine for I2C and SPI slaves** — `ww_bus_slave`, no_std, sans-IO, no alloc: `I2cSlave` and
  `SpiSlave` do what STD-35's API promises, so a firmware only wires its peripheral's interrupts to them. I2C: the
  register map (pointer bytes, auto-increment, wrap at `len`, page wrap, `read_only`), host updates staged until the
  stop, the answer queue with `Fill`, `Stretch { max }` (deadline, `resume`, `ReadWaiting`) and `Nack`
  (`acks_address`), `unsent` for a TX FIFO loaded ahead. SPI: queue answers (`idle`, `repeat_last`, `underrun`) and
  `SpiRegisters` frames (address, `rw` bit, dummy bytes before read data), the answer picked at the first `miso()`
  (FIFO preload, `rewind`); 8-bit words only. Events built as `I2cSlaveEvent` / `SpiSlaveFrame`. `const fn new` for a
  `static`; 58-100 instructions per byte on a Cortex-M3 (`examples_mcu/mcu_qemu` bin `qemu-bus-slave`).
  `docs/api/bus_slave.md`, tests `ww_bus_slave/tests`. First firmware user: usb_io_fw IF-5 (rePort, RP2350B).

## Client / host side (`CLT`)

- ✅ **CLT-59 Crash call chain as `file:line`** · `v0.5` — firmware built with `-Zlocation-detail=none` has no
  `file:line` in its panic messages, so `ww report` decodes the crash record's addresses with the ELF's DWARF
  (`addr2line`, `gimli`, pure Rust): a fault's `pc` exactly, its `lr` and the stack's return addresses at the call
  before them, inlined frames innermost first, demangled names, shortened `trim-paths` paths; stack words that are
  no return address (Thumb bit, `.text`, after a `BL`/`BLX`, read from the ELF's code, the rule of
  `bedrock_crash::is_return_address`) are dropped; the panic machinery is passed over to mark the panic site.
  `ww_report::crash` (`Symbolizer`, `CrashAddresses`, `CrashFrames`, serializable) for other host tools,
  `docs/std_library/device_contract.md#crash-call-chain`, tests against a real thumbv6m ELF
  (`ww_stdlib/ww_report/tests/crash_frames.rs`, `just crash-fw-elf`).

## Tools (`TOOL`)

- ✅ **TOOL-28 `ww board rework set` / `clear`** — records a rework done on the bench through
  `BoardInfo::set_reworks` (STD-98), on any device serving it anywhere in its API (a global trait attachment,
  API-20). `ww board rework set 1,3,5` / `clear 1,3,5` mark reworks applied or undone, all or none; `--dry-run`
  passes `program = false` (nothing written, the would-be outcome printed). `UnknownRework` is shown with the
  numbers the firmware knows, read from `REWORKS`: `unknown rework 7: this firmware knows 1-5, 8; update firmware
  first`; `Conflict` and `ProductionRecord` print the device's own message. `wire_weaver_cli` `cmd::board`. The
  first device is usb_io_fw PLAT-17. Tested on B200A 027 (3 Nov 2026, firmware 0.10.29, `ww` feb97a85): `set` /
  `clear` with and without `--dry-run`, `set 9` refused with the known numbers, nothing stored.
- 🧪 **TOOL-21 `ww pins tui`** · `v0.5.x` — every point live, laid out per connector of a board preset
  (`docs/cli_reference.md#ww-pins-tui`): level and what was set on each pad, keys to drive, toggle, pull, weak
  drive, read all, release, pause; every point released on quit, signals and errors. The preset is boardmap's board
  description read as it is (`[[view.connector]]`, `[aliases]`, `[pins]` roles, pads from its geometry JSON, nets
  joined across fitted solder bridges), no dependency on embedded_bedrock; a preset for ww alone lists nets per
  connector. Devices with `ww_gpio` banks and no `PinDiag` are shown through the banks (modes and pulls put back on
  release). Library `ww_pin_probe::board`, `live`, `bank`; tests on `MockBoard` with the NUCLEO-G0B1RE preset
  (`wire_weaver_cli` `cmd::pins_tui`) and over banks end to end (`tests/io_gpio`). Hardware check open: the Nucleo
  G0B1RE once its firmware serves `PinDiag`.
- 🧪 **TOOL-29 Flashers check the board's reworks** · `v0.5` — `ww dfu rp flash` and `ww dfu stm32 flash` check
  with the chip in its ROM bootloader (user, 3 Nov 2026: a board with an unknown rework refuses to boot and sits
  there). Given the serial of a running device they reboot it there first (`BootControl::reboot(Updater)`, an A/B
  device only, the titles of its `REWORKS` kept for the question). In the ROM they read the new image's table,
  locator and build id from the file (`ww_board_info::image`, STD-101), the table of the image in flash (the board's
  name, `--board` when no image names exactly one) and the bench record and allowance from the settings store
  (PICOBOOT read, DfuSe upload), and warn with the list when the image lacks a rework; only if the user agrees (or
  `--allow-unknown-reworks` names exactly those numbers) the allowance is stored for the image's `ImageId` through
  the ROM (the store's own erases and writes replayed, read back), otherwise a stored one is cleared. An image for
  another board, without a table or with an unreadable one, or without a build id, is asked about too (`--yes`);
  without a terminal every question is refused. An STM32 whose bootloader can't read its flash (readout
  protection) is written only with `--yes`. The check as a library: `ww_report::rework_check` (`RomFlash`,
  `ImageReworks`, `scan_installed`, `read_board`, `verdict`, `set_allowance`).
  `docs/cli_reference.md#checking-the-boards-reworks`. Tests: `ww_report` `rework_check::tests` (the whole check on
  a chip in RAM), `wire_weaver_cli` `cmd::rework_check::tests`. `ww dfu flash` (the `ww-chunky` plugin, a running
  app, no ROM at hand) compares with the app's `REWORKS` and refuses a lacking image. Hardware
  check open: an RP2350 or STM32 board with a guarding firmware (usb_io_fw PLAT-18).
- 📋 **TOOL-8 `ww` inspect / read / write all GPIOs on any device** · `v0.5.x`.

## Evolution, versioning and compatibility (`EVO`)

- ✅ **EVO-60 `ww_user_info` caps bounded on the wire** · `v0.5` — the ww_stdlib units survey's Q4: `UserInfoCaps::name_max` and `notes_max` carry their device-wide ceiling as a bound (`NameMax` / `NotesMax`, `Fixed<units::Byte, One, u16, Range<0, 32>>` / `Range<0, 256>>`) instead of a plain `u16`; `ww_user_info` on a new wire line, `w2.0`. No device or host uses the format.
- ✅ **EVO-62 `ww_can_bus` on wire `w2.0`** · `v0.5` — the wire change of STD-114: new `Range` bounds (load and sample point 0-1000, element data 8-64 bytes, TTCAN matrix cycles and repeat 1-64, cycle offset and cycle 0-63, filter index 0-255), `TriggerKind::RxCheck::filter` from `u16` to the `UNib32` of `RxFrame::filter`, `TtStatus::error_level` from `u8` to the enum `TtErrorLevel` (2 bits), `TtRole::Master::priority` from `u8` to `U3`; every other change wraps the same number. The crate was never published, so its 0.1.0 snapshot was saved again in place on `w2.0`.
- ✅ **EVO-61 `ww_onewire` on a new wire line** · `v0.5` — the wire change of STD-113: `SearchState::last_discrepancy`
  narrows from a plain `u8` to `IndexFrom1<RomBit, u8, Range<1, 64>>`, a bound added, so `ww_onewire` moves from
  `w1.0` to `w2.0`. Every other change of STD-113 (`ww_onewire`'s `FamilyCode`, `read_len`,
  `pin`; all of `ww_uart`'s) is a transparent wrapper or an unbounded `Fixed`/`Index`/`Count`, same bytes, no wire
  bump. `ww_onewire_0_1_0.ron` is saved again in place (never published). No device or host uses the
  format.
- ✅ **EVO-65 `ww_dfu` typed numbers on the wire** · `v0.5` — `ww_dfu` on a new wire line, `w2.0` (P2401 stdlib units survey): step indices and counts are `Index<Step, u32>` / `Count<Step, u32>`, chunks `Index`/`Count<Chunk, u16>`, erase blocks `Index<Block, u16>` / `Count<Block, u16>`, byte offsets in a chunk or a plan `Index<Byte, u32>`, sizes `Bytes<u32>`, all the same bytes. The breaks: `NodeInfo`'s `product_id`, `app_version` and `version_floor` become `app: Option<ImageInfo>` (`ProductId`, `ImageVersion`, and the image's new `security_counter`) and `security_floor: SecurityCounter`, as chunky checks the floor against the security counter, not the version; `geometry_id` is `geometry: GeometryId`; `LastResult::version` is `Option<ImageVersion>`; `Session::resume_from` is split into `resume_step` and `resume_byte`; `StageLimits::max_piece` is `Fixed<Byte, One, u16, Range<1, 1024>>` (a `u16`, bounded), `in_flight` `Fixed<Request, One, u8, Range<1, 255>>`; `ack_every` and `plan_area` are `Niched<.., Null<0>>`. Tests `ww_dfu` (bytes of every wrapper, the bounds, the niches), `tests/dfu`, `tests/dfu_scenarios` (transcripts again). No device or host uses the format.
- ✅ **EVO-72 Capability limits bounded by their consts** · `v0.5` — each `ww_stdlib` limit documented "at most
  `MAX_..`" carries that const as its `Range` bound (`{ MAX_X as i64 }`, read by the walker's const evaluator) in the
  smallest storage that holds it, refused past it on decode: `ww_i2c` (`I2cCapabilities`' four limits,
  `I2cWatchCapabilities` with the new unit `Watch`, `I2cWatchConfig::len`; wire `w1.0` -> `w2.0`), `ww_user_info`
  (`NameMax` in a `u8`; `w2.0` -> `w3.0`), `ww_spi` (`SpiCapabilities::max_len` in a `u16`; `w2.0` -> `w3.0`),
  `ww_dfu` (`max_piece`, same bytes). Never published, so the 0.1.0 snapshots were saved again in place. No device
  or host uses the format (`~/git/tpm/area/ops/in-use.md`).
- ✅ **EVO-57 `ww_spi` typed numbers on the wire** · `v0.5` — the wire change of STD-56 for `ww_spi`: word size,
  byte lengths, register offsets and counts move from a bare integer to `exact_units::Fixed` / `Bytes` / `Index` /
  `Count` (same bytes, Rust-only break besides the renames); `SpiRwBit::SetIsRead`/`ClearIsRead`'s `bit` gets a
  `Range<0, 7>` bound and `io_level` moves from `ww_si::Quantity<Volt>` to `Fixed<Volt, Milli, u16>` (both narrow
  or change what is sent, a real wire break); `ww_spi` was never published, so its 0.1.0 snapshot was saved again
  in place on a new wire line, `w2.0`. No device or host uses the format (`~/git/tpm/area/ops/in-use.md`).
- ✅ **EVO-59 `ww_logic_analyzer` typed numbers on the wire** · `v0.5` — the stdlib units survey's per-crate change (`prj/P2401-wireweaver/design/stdlib-units-survey.md`): every channel number is an `Index<Channel, u8>` (`read_transitions`, `TriggerStage::UartByte::channel`, `I2cAddress::scl`/`sda`, `SpiFirstByte::cs`/`sck`/`data`, `ChannelCondition::channel`, `TransitionsChunk::channel`, `CaptureConfig::channels`, `StreamConfig::channels`), `RateLimit::channels` a `Count<Channel, u8>`, `ChannelInfo::gpio_pin` an `Option<Index<Pin, u8>>`; every sample position an `Index<Sample, u32>` and every sample amount a `Count<Sample, u32>` (`read_transitions`'s `from`, `CaptureStatus::Done::trigger_at`/`samples`, `SampleRange`, `RawChunk`, `TransitionsChunk::from`/`scanned_to`/`edges`, `CaptureConfig::samples`/`pre_trigger`, `CaptureActual::samples`); `LaCapabilities::memory` a `Bytes<u32>`, `RawLayout::width` a `Fixed<Bit, One, u8>`, `TriggerCapabilities::max_stages`/`max_conditions` a `Count<Item, u8>`, `TriggerStage::SpiFirstByte::mode` a `ww_spi::SpiMode` instead of a bare `0..=3` `u8` (reusing `ww_spi`'s enum, as the crate already reuses `ww_gpio::Level` and `ww_i2c::I2cAddress`; `ww_spi::SpiMode` now derives `defmt::Format`, as `TriggerStage`'s conditional derive needs). Every change a transparent wrapper or an enum of the same `ww_repr`, so the bytes are unchanged; wire line `w2.0` (shared with EVO-64) records the break, as the survey's decisions call for a new line, a CHANGELOG entry and an EVO item per break, bytes or not. `ww_logic_analyzer` 0.1.0 (never published), its `api_snapshots` refreshed. No device or host uses the format.

- ✅ **EVO-70 Old-format snapshots compare embedded definitions; pins refreshed by a tool** · `v0.5` — wire_weaver's item: the checker and `ww api refresh-pins` live there. Its `nucleo_g0b1re_api` fixture (a deployed firmware's API 0.4.0 with its old-format snapshots, on this repository's crates by path) and the fixture's tests live here, in `tests/nucleo_g0b1re_fixture`: a compatible stdlib change passes it, a break fails it, and `just refresh-fixture-pins` (called by `just save-snapshots`) refreshes its pins. `clock_pin_matches_wire_weavers_copy` fails when its `ww_timebase::Clock` pin changes, which wire_weaver's copy of the 0.4.0 snapshot must then follow.

## Testing (`TEST`)

- 🐛 **TEST-20 DFU transcript 1L-UPD-STEP races**: after the power cut, the host annotation "the call fails (link
  lost)" and the bootloader's "an update is under way" are recorded in either order, so
  `scenarios::pit_stop_shuttle_1l_upd_step` fails now and then (~2 in 10 alone, more often under the full parallel
  `just test`). Seen before the TR-12 changes too. The recorder needs one order for events of the host and the
  simulated device that happen at the same time. Found 15 Oct 2026.

## Dropped and superseded

- **STD-9 Stubs, for later: `ww_spi`** — superseded by STD-13: `ww_spi` is now a full IO-family trait (optional
  `Spi::monitor`, typed numbers, STD-56), not a stub; `ww_board_info`, `ww_firmware_info`, `ww_user_info` and
  `ww_uid` moved to STD-24 already.
- **STD-30 `ww_gpio` port access** — dropped (Roman, 20 Oct 2026), `Bank::read_inputs` / `write_outputs`,
  `PinLevel` and the `simultaneous_*` capabilities removed before `ww_gpio` 0.1.0 was published. `ww_gpio` stays
  simple and low speed, one pin per call (IO family D9): many pins in one round trip are per-pin calls in one multi
  request (CLT-15, `docs/std_library/ww_gpio.md#several-pins`); several pins at one instant or with precise timing
  (bit-banged parallel buses, a strobe with its data lines) belong to the signal generator (STD-22) and the logic
  analyzer (STD-21). The extra methods only cost flash on every GPIO server.
