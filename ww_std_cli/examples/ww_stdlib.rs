//! `ww` with the generic commands and the standard library's, for the recipes of this repository (`just
//! save-snapshots`, `just api-check`): it builds with the root workspace alone and is never installed.
//!
//! ```sh
//! cargo run -p ww_std_cli --example ww_stdlib -- api save ww_dfu
//! ```

// the standard library commands register themselves when their crate is linked
use ww_std_cli as _;

fn main() -> anyhow::Result<()> {
    wire_weaver_cli::main(wire_weaver_cli::Program {
        name: "ww",
        version: env!("CARGO_PKG_VERSION"),
        hub: Default::default(),
    })
}
