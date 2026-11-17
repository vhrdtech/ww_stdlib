# `just jobs=N <recipe>` caps build and test parallelism for that run; unset leaves CARGO_BUILD_JOBS and
# NEXTEST_TEST_THREADS as inherited, or at cargo's default (the CPU count)
jobs := ""
export CARGO_BUILD_JOBS := if jobs != "" { jobs } else { env_var_or_default('CARGO_BUILD_JOBS', num_cpus()) }
export NEXTEST_TEST_THREADS := if jobs != "" { jobs } else { env_var_or_default('NEXTEST_TEST_THREADS', 'num-cpus') }

# `ww` with the generic and the standard library commands, built from ww_std_cli's example and never installed
ww := "target/debug/examples/ww_stdlib"

# List all available targets
default:
    @just --list

# The tests. `just test`: all of them (test-all). `just test --changed`: only the crates changed against main and
# the crates that depend on them (through tpm when installed, otherwise the full suite)
test *flags:
    if [ "{{ flags }}" = "--changed" ]; then \
        if command -v tpm >/dev/null 2>&1; then tpm test changed -- cargo nextest run --workspace; \
        else echo "tpm not found; running the full test suite instead" >&2; {{ just_executable() }} test-all; fi; \
    else {{ just_executable() }} test-all {{ flags }}; fi

# Every test: nextest, the doctests, ww_block's CRC without std at two table sizes, the builds of `just check`
test-all: check
    cargo nextest run --workspace --no-fail-fast
    # nextest does not run doctests
    cargo test --workspace --doc -q
    # ww_block's crc32/crc32_update without std, at the default and the 256-entry table (TR-31)
    cargo nextest run -p ww_block --no-default-features
    cargo nextest run -p ww_block --no-default-features --features ww_crc/table-256

# cargo check what tests don't build: the board examples (each a workspace of its own on nightly)
check:
    cd examples_mcu/mcu_qemu && cargo check
    cd examples_mcu/nucleo_h743zi2 && cargo check && cargo check --features streams
    cd examples_mcu/usb_stm32h725ig && cargo check && cargo check --features streams
    cd examples_mcu/rp2 && cargo check --features usb_log

# Format, manifests, spelling, clippy and rustdoc as CI runs them, warnings are errors; then `just api-check`
lint:
    cargo fmt --all --check
    cd examples_mcu/mcu_qemu && cargo fmt --check
    cd examples_mcu/nucleo_h743zi2 && cargo fmt --check
    cd examples_mcu/usb_stm32h725ig && cargo fmt --check
    cd examples_mcu/rp2 && cargo fmt --check
    cd ww_report/tests/crash_fw && cargo fmt --check
    cargo sort -w -g --check
    typos
    cargo clippy --workspace --all-targets -- -D warnings
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features
    just api-check

# `ww api check` of every crate with api_snapshots/: its API against its latest snapshot (docs/evolution/checker_tool.md)
api-check:
    cargo build -q -p ww_std_cli --example ww_stdlib
    for crate in */api_snapshots; do {{ ww }} api check "$(dirname $crate)" || exit 1; done

# Save the API snapshot of every crate with api_snapshots/ into it. Pass --force to overwrite the snapshot of a
# version that was never published.
save-snapshots *args:
    cargo build -q -p ww_std_cli --example ww_stdlib
    for crate in */api_snapshots; do {{ ww }} api save "$(dirname $crate)" {{ args }} || exit 1; done
    # the frozen fixture that follows these crates by path: its pins follow too (EVO-70)
    just refresh-fixture-pins

# Refresh the pinned signatures of the nucleo_g0b1re_api evolution fixture to these crates' current source, keeping
# its old snapshot format (EVO-70). Refused on a wire break, unless --accept-breaking.
refresh-fixture-pins *args:
    cargo build -q -p ww_std_cli --example ww_stdlib
    {{ ww }} api refresh-pins tests/nucleo_g0b1re_fixture/fixtures/nucleo_g0b1re_api {{ args }}

# Regenerate the ww_dfu scenario transcripts (tests/dfu_scenarios/transcripts, included by docs/ww_dfu_ab.md and ww_dfu_one_slot.md)
dfu-transcripts:
    DFU_TRANSCRIPTS=update cargo nextest run -p dfu_scenarios

# Rebuild the crash symbolizer's test ELFs from ww_report/tests/crash_fw (CLT-59); recheck the tests' addresses with `llvm-objdump -d` after
[working-directory('ww_report/tests/crash_fw')]
crash-fw-elf:
    cargo build --release
    cp target/thumbv6m-none-eabi/release/crash_fw ../crash_fw.elf
    llvm-objcopy --strip-debug ../crash_fw.elf ../crash_fw_nodebug.elf

# Upload the Nucleo H743ZI2 all-GPIO firmware to the firmware store (mx3)
[working-directory('examples_mcu/nucleo_h743zi2')]
upload-examples-mcu-nucleo-h743zi2:
    mx3 fw upload --bin all_gpio --release --rename usb_nucleo_h743zi2_all_gpio

# Upload the STM32H725IG UART bridge firmware to the firmware store (mx3)
[working-directory('examples_mcu/usb_stm32h725ig')]
upload-examples-mcu-usb-stm32h725ig:
    mx3 fw upload --bin uart --release --rename usb_b135_uart

# The libraries have nothing to put on PATH: builds them with the deploy profile, as a dependent's release build would
install:
    cargo build --profile deploy --locked --workspace

# Bring this checkout up to main: refuses on local changes, pulls, builds
deploy:
    #!/usr/bin/env bash
    set -euo pipefail
    dirty=$(git status --porcelain)
    if [ -n "$dirty" ]; then echo "local changes, not deploying:"; echo "$dirty"; exit 1; fi
    git pull --ff-only
    just install
