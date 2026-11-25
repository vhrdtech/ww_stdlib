//! Reads two sentences of the GPS receiver on port 0, then sends a line through the loopback of port 1 and reads it
//! back, through the standard `ww_uart` calls.
//!
//! `cargo run -p uart --example uart_bridge` against `uart_sim` on this PC, or `-- --url ws://ADDRESS/ww` against
//! another.

use anyhow::Result;
use clap::Parser;
use uart::UartBridge;
use wire_weaver_client::StreamEvent;
use ww_uart::{Baud, Count, DataBits, IDLE_CHARS_DEFAULT, Mode, Parity, StopBits, UartConfig};

#[derive(Parser)]
struct Args {
    /// WebSocket URL of the device
    #[arg(long, default_value = "ws://127.0.0.1:8082/ww")]
    url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let device = UartBridge::config(|c| c.websocket_url(args.url.clone()).no_ww_usb())
        .connect()
        .await?;

    // --8<-- [start:configure]
    let gps = device.uart(0);
    let config = UartConfig {
        baud_rate: Baud::base(9600u32),
        data_bits: DataBits::Eight,
        parity: Parity::None,
        stop_bits: StopBits::Stop1,
        mode: Mode::Asynchronous,
        prevent_back_feed: false,
        io_level: None,
        idle_chars: Count::new(IDLE_CHARS_DEFAULT),
    };
    let set = gps.configure(config).call().await?;
    println!("port 0: {} Bd", set.baud_rate.to_f64());
    // --8<-- [end:configure]

    // --8<-- [start:rx]
    let mut rx = gps.rx().call().await?;
    let mut sentences = 0;
    while sentences < 2 {
        if let StreamEvent::Item { t, value, .. } = rx.recv_event().await? {
            let first = t.map_or(0.0, |t| t.to_f64());
            println!(
                "{first:.3} s: {}",
                String::from_utf8_lossy(&value.bytes).trim_end()
            );
            sentences += 1;
        }
    }
    // --8<-- [end:rx]

    // --8<-- [start:loopback]
    let jumper = device.uart(1);
    let mut echo = jumper.rx().call().await?;
    let mut tx = jumper.tx().await?;
    tx.send_bytes(b"hello, loopback\n").await?;
    let chunk = echo.recv().await?;
    println!(
        "port 1: {}",
        String::from_utf8_lossy(&chunk.bytes).trim_end()
    );
    // --8<-- [end:loopback]

    device.disconnect().asynch().await?;
    Ok(())
}
