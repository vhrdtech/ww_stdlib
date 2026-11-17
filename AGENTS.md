# AGENTS.md

Guidance for AI code assistants working in this repository.

## What this is

ww_stdlib is the standard library of [WireWeaver](https://github.com/vhrdtech/wire_weaver): ready-made API traits and
types (GPIO, UART, I2C, SPI, CAN, 1-Wire, ADC, PWM, logic analyzer, memory, block devices, SD cards, firmware info,
board info, unique ids, logs, counters, indication, firmware update, recorder control) that a device puts into its
own API, with their device helpers (`ww_bus_slave`, `ww_defmt_logger`), host helpers (`ww_gpio_hl`, `ww_i2c_hl`,
`ww_report`, `ww_pin_probe`) and the `ww` commands of these APIs (`ww_std_cli`). It stands on WireWeaver's framework
(`wire_weaver`, `ww_api`, the client), the base crates of `ww_base` (`ww_si`, `ww_numeric`, `ww_version`, `ww_global`,
`ww_error`, `ww_date_time`, `ww_crc`), `exact_units` and `shrink_wrap`, all taken by git. The `ww` binary that ships
compiles these crates in and lives above this repository.

Read `docs/overview.md`, `docs/io_family.md` and `docs/device_contract.md` first, then the page of the crate at hand.
The docs are published as the Standard library section of the WireWeaver book, which assembles them from this
repository: links to other pages of the book (`../api/..`, `../cli_reference.md`, `../evolution/..`) only resolve
there. WireWeaver's `AGENTS.md` and docs (`docs/api/`, `docs/evolution/`, `docs/serdes/`) hold the rules every API
crate follows: typed numbers (never a bare integer with its unit in the name), evolvable types, versions.

## Layout

| Path | What |
| --- | --- |
| `ww_<api>/` | one API crate each: `#[ww_trait]` traits and their types, `no_std`, on `ww_api`; its `api_snapshots/` |
| `ww_gpio_hl/`, `ww_i2c_hl/` | host helpers: typed drivers over the GPIO and I2C traits (embedded-hal for I2C) |
| `ww_bus_slave/` | device side of the I2C and SPI slave data paths (register maps, queues, interrupt hooks) |
| `ww_defmt_logger/` | a defmt global logger feeding `ww_log` streams |
| `ww_report/`, `ww_pin_probe/` | the device report (`ww report`) and pin probing (`ww pins`) libraries |
| `ww_std_cli/` | the `ww` commands of these APIs (board, counters, dfu, pins, report, ...); they register themselves |
| `tests/<name>` + `tests/<name>_api` | one integration-test crate per API with its fixture API crate |
| `tests/stdlib_evolution` | every crate against its `api_snapshots/`, one test per crate (TOOL-31) |
| `tests/nucleo_g0b1re_fixture` | a deployed firmware's API on these crates against its frozen old-format snapshots (EVO-70) |
| `tests/global_ids` | every public trait has an id in the `ww_global` ledger (API-20) |
| `examples/` | paired `<name>_api` and `<name>` crates: all_gpio, defmt_log, uart |
| `examples_mcu/` | board firmware using these APIs (QEMU Cortex-M3, Nucleo H743ZI2, STM32H725IG, RP2350): each its own workspace |
| `docs/` | the pages of these crates |

`examples_mcu/*` and `ww_report/tests/crash_fw` are separate workspaces (bare-metal targets): build them through
`just`.

## FEATURES.md

`FEATURES.md` is the feature tracker and roadmap, the only place status is kept. Check it before starting a feature
and update the matching item in the same change when finishing one (conventions at its top).

- IDs are stable per area (`STD-12`, `CLT-31`): never renumber or reuse one; new items take the next free number of
  their area. Use them in commits, CHANGELOG entries and `// TODO(STD-20): ...`.
- When talking to the user, name an ID with a 2-4 word kebab-case slug from its title: `STD-12 ww-dfu-api`.
- Items came from WireWeaver's tracker with their IDs; an ID not listed here is WireWeaver's (or ww_base's).

## Commands

```sh
just test --changed   # the crates changed against main and their dependents; before a session's work is ready
just test             # everything: nextest, doctests, ww_block without std, just check
just lint             # fmt, cargo sort -w -g, typos, clippy and rustdoc -D warnings, just api-check
just check            # the board examples
just api-check        # `ww api check` of every crate with api_snapshots/
just save-snapshots   # `ww api save` of every crate with api_snapshots/ (--force for an unpublished version)
just refresh-fixture-pins  # `ww api refresh-pins` of the nucleo_g0b1re_api fixture; save-snapshots calls it
just dfu-transcripts  # regenerate the ww_dfu scenario transcripts the docs include
```

Single test: `cargo nextest run -p io_i2c <test>` (nextest is required). Plain `cargo sort` flattens grouped tables:
always pass `-w -g`. Project words go into `typos.toml`. CI (`.github/workflows/ci.yml`) fails on any warning.

`ww` here is `target/debug/examples/ww_stdlib`, built from `ww_std_cli/examples/ww_stdlib.rs`: the generic commands
of `wire_weaver_cli` plus this repository's. It is never installed.

## API crates: snapshots, versions, ids

- Every API crate keeps its own `api_snapshots/`. Codegen leaves types known from a dependency's snapshots out of a
  device's introspection data, and clients put them back. After changing a trait or type, run `just
  save-snapshots`; `tests/stdlib_evolution` fails otherwise. A save refreshes the crate's current version, right
  only for a version never published (`cargo info --registry crates-io <crate>`): bump a published one first.
- A crate's version is its API's identity. Wire-compatible additions bump the patch, breaking changes the minor
  (pre-1.0), at release, never in a work commit. `[package.metadata.wire_weaver] wire = "wN.M"` is the crate's wire
  line: a wire break needs WireWeaver's evolution rules (`docs/evolution/rules.md` there), not only a version bump.
- Every public `#[ww_trait]` has an id in `ww_global`'s ledger (in ww_base) or is listed in its `no_id`;
  `tests/global_ids` checks it. A new trait claims its id there first.
- Pinned signatures and snapshots are written by the tool, never by hand. The `nucleo_g0b1re_api` fixture is never
  saved again: its pins follow these crates through `just refresh-fixture-pins`. Its `ww_timebase::Clock` pin has a
  copy in wire_weaver (`tests/stdlib_evolution/pins/`): when it changes, copy the refreshed file there.

## Proper fixes, never a lost capability

- A bug or limit found on the way is fixed where it lives, as its own commit, with a test that fails without it. A
  bug in the framework or a base crate is fixed in its repository, then taken here with `cargo update -p NAME`.
- A removal, replacement or refactor keeps everything that worked before. Never narrow tests, docs or
  `FEATURES.md` to fit a loss.
- Flaky and slow tests: find the cause, never raise a limit (`.config/nextest.toml` holds hang detectors only).

## Firmware: stack, cycles and flash

API types and device helpers run on small MCUs, often in interrupts. A change to them is checked for stack, cycles
and flash, not only correctness: big state in a `static` set up in place, measure at the profile firmware ships with
(`opt-level = "z"`, fat LTO) and say which, instruction counts with WireWeaver's `examples_mcu/qemu_icount` on
`qemu-bus-slave`, no wide number type or optional feature turned on for every firmware.

## Developing with the other repositories

Dependencies outside this repository come by git with no `rev` or `tag`: `Cargo.lock` pins each commit and `cargo
update -p NAME` moves it. To develop against a local checkout of `wire_weaver`, `ww_base`, `exact_units` or
`shrink_wrap`, put a `[patch."https://github.com/..."]` in a `.cargo/config.toml` above this checkout, never
committed, and commit no `Cargo.lock` that names a patched path.

## Changelogs, commits and versions

- Every user-visible change gets an entry under `## Unreleased` in the `CHANGELOG.md` of each affected crate, in the
  same change (`### Breaking`, `### Features`, `### Fixes`; name the public items; for breaking changes say what
  users must change). The root `CHANGELOG.md` gets one summary line per change.
- Commits use Conventional Commits with a scope (`feat(ww_i2c): ...`): a short imperative subject, a blank line, a
  body of at most 5 lines saying what and why, and a `Tested:` line. A move is never edited in the same commit.
- Work commits never bump versions; the release commit does.
- Dependencies live in the root `[workspace.dependencies]`; crates inherit `authors`, `edition`, `license` and
  `repository` from `[workspace.package]`.
- Public text (README, docs, CHANGELOG, commits, comments) is neutral: no private paths or internal tracker names.
