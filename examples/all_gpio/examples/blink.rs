//! Lists the banks of a GPIO expander, blinks its LED and reads its button, through the standard `ww_gpio` calls.
//!
//! `cargo run -p all_gpio --example all_gpio_blink` against `all_gpio_sim` on this PC, `-- --url ws://ADDRESS/ww`
//! against another, or `-- --usb --led PB0` against the Nucleo H743ZI2's all_gpio firmware over USB.

use all_gpio::AllGpio;
use anyhow::{Context, Result};
use clap::Parser;
use std::time::Duration;
use ww_gpio::{Level, Mode, Pull};

#[derive(Parser)]
struct Args {
    /// WebSocket URL of the device
    #[arg(long, default_value = "ws://127.0.0.1:8081/ww")]
    url: String,
    /// Connect over USB instead, to a board running the all_gpio firmware
    #[arg(long)]
    usb: bool,
    /// The pin the LED sits on
    #[arg(long, default_value = "PA5")]
    led: String,
    /// The pin the button sits on
    #[arg(long, default_value = "PC13")]
    button: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let device = if args.usb {
        AllGpio::new().connect().await?
    } else {
        AllGpio::config(|c| c.websocket_url(args.url.clone()).no_ww_usb())
            .connect()
            .await?
    };

    // --8<-- [start:banks]
    let mut banks = Vec::new();
    for port in device.port_valid_indices().read().await?.iter() {
        let name = device.port(port).name().call().await?;
        let pins = device.port(port).pin_valid_indices().read().await?;
        println!("port {port}: {name}, pins {pins:?}");
        banks.push((port, name));
    }
    // --8<-- [end:banks]
    let pin = |name: &str| -> Result<(u32, u32)> {
        let (port, bank) = banks
            .iter()
            .find(|(_, bank)| name.starts_with(bank.as_str()))
            .with_context(|| format!("no bank of {name}"))?;
        let number = name[bank.len()..]
            .parse()
            .with_context(|| format!("no pin number in {name}"))?;
        Ok((*port, number))
    };

    // --8<-- [start:blink]
    let (port, number) = pin(&args.led)?;
    let bank = device.port(port);
    let led = bank.pin(number);
    led.set_mode(Mode::PushPullOutput, Some(Level::Low))
        .call()
        .await?;
    for _ in 0..3 {
        led.toggle().call().await?;
        println!("{}: {:?}", args.led, led.output_level().call().await?);
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    // --8<-- [end:blink]

    // --8<-- [start:button]
    let (port, number) = pin(&args.button)?;
    let bank = device.port(port);
    let button = bank.pin(number);
    button.set_mode(Mode::Input, None).call().await?;
    button.write_pull(Pull::None).write().await?;
    for _ in 0..6 {
        let pressed = button.input_level().call().await? == Level::Low;
        println!(
            "{}: {}",
            args.button,
            if pressed { "pressed" } else { "up" }
        );
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    // --8<-- [end:button]

    device.disconnect().asynch().await?;
    Ok(())
}
