//! Asks the sensor node what it is, gives it a name, makes it log, and reads its log, through the standard device
//! contract traits.
//!
//! `cargo run -p node --example node_about` against `node_sim` on this PC, or `-- --url ws://ADDRESS/ww` against
//! another.

use anyhow::{Context, Result, anyhow};
use clap::Parser;
use node::Node;
use wire_weaver::shrink_wrap::{MaxLen, UVlq64};
use wire_weaver_client::{StreamCursor, StreamEvent};

#[derive(Parser)]
struct Args {
    /// WebSocket URL of the device
    #[arg(long, default_value = "ws://127.0.0.1:8083/ww")]
    url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let node = Node::config(|c| c.websocket_url(args.url.clone()).no_ww_usb())
        .connect()
        .await?;

    // --8<-- [start:about]
    let device = node.device();
    let firmware = device.firmware().read_FIRMWARE().read().await?;
    let board = device.board().read_BOARD().read().await?;
    let ids = device.uid().read_IDS().read().await?;
    let v = &firmware.version;
    println!(
        "firmware {} {}.{}.{}",
        firmware.name.as_str(),
        v.major.0,
        v.minor.0,
        v.patch.0
    );
    println!(
        "board {} rev {}",
        board.name.as_str(),
        board.revision.as_str()
    );
    for id in ids.iter() {
        println!("id {:?}: {:02x?}", id.kind, id.value.as_slice());
    }
    // --8<-- [end:about]

    // --8<-- [start:label]
    let name = MaxLen::new("bench 2".to_owned()).map_err(|e| anyhow!("{e:?}"))?;
    device.label().write_name(name).write().await?;
    println!("name {}", device.label().read_name().read().await?.as_str());
    // --8<-- [end:label]

    // --8<-- [start:log]
    node.measure().call().await?;
    // the log keeps its records until a host has them: read them from the start of the node's session
    let status = device.log().records_status().read().await?;
    let session = status.session.context("the log has no session")?;
    let from_start = StreamCursor {
        session,
        pos: UVlq64(0),
    };
    let mut log = device.log().records_resume(from_start).call().await?;
    let mut records = 0;
    while records < 2 {
        if let StreamEvent::Item { pos, t, value, .. } = log.recv_event().await? {
            let t = t.map_or(0.0, |t| t.to_f64());
            let (severity, message) = (value.severity, value.message.as_str());
            println!("#{pos} at {t:.3} s, {severity:?}: {message}");
            records += 1;
        }
    }
    // --8<-- [end:log]

    node.disconnect().asynch().await?;
    Ok(())
}
