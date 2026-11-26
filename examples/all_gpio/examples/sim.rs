//! The GPIO expander simulated on this PC, served over WebSocket: three banks of 16 pins, an LED on PA5 and a
//! button on PC13. Ctrl-C stops it.
//!
//! `cargo run -p all_gpio --example all_gpio_sim`, then in a second terminal
//! `cargo run -p all_gpio --example all_gpio_blink`.

use anyhow::Result;
use clap::Parser;
use futures_util::Stream;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt};

#[derive(Parser)]
struct Args {
    /// TCP port of the WebSocket server; 0 takes a free one, which the first line names
    #[arg(long, default_value_t = all_gpio::SIM_PORT)]
    port: u16,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(async {
        let listener = TcpListener::bind(("127.0.0.1", args.port)).await?;
        println!(
            "all_gpio at ws://{}{}",
            listener.local_addr()?,
            all_gpio::WS_PATH
        );
        let stop = async {
            _ = tokio::signal::ctrl_c().await;
        };
        all_gpio::simulated(incoming(listener), stop).await;
        println!("stopped");
        Ok(())
    })
}

/// The clients of `listener`, one at a time, with Nagle off.
fn incoming(listener: TcpListener) -> impl Stream<Item = Compat<TcpStream>> + Unpin {
    Box::pin(futures_util::stream::unfold(
        listener,
        |listener| async move {
            loop {
                match listener.accept().await {
                    Ok((stream, peer)) => {
                        if let Err(e) = stream.set_nodelay(true) {
                            println!("{peer}: TCP_NODELAY: {e}");
                        }
                        println!("client {peer}");
                        return Some((stream.compat(), listener));
                    }
                    Err(e) => println!("accept: {e}"),
                }
            }
        },
    ))
}
