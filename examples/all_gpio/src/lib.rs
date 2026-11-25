//! Client of the GPIO expander example device ([all_gpio_api]), and the device simulated on a PC.
//!
//! [AllGpio] is the host side: the generated calls `port(i).name()`, `port(i).pin(j).set_output_level(..)` and
//! the rest of `ww_gpio`'s `Bank` and `Pin`. [simulated] runs the device itself, `all_gpio_device` on its
//! [SimBoard], over WebSocket: from a TCP listener in the `all_gpio_sim` example, or in process for tests.
//!
//! ```text
//! terminal 1: cargo run -p all_gpio --example all_gpio_sim
//! terminal 2: cargo run -p all_gpio --example all_gpio_blink
//! ```

use std::future::Future;

use all_gpio_device::AllGpio as Device;
pub use all_gpio_device::SimBoard;
use futures_util::Stream;
use futures_util::io::{AsyncRead, AsyncWrite};
pub use wire_weaver_client::ClientConfig;
use wire_weaver_client::in_process::TokioClock;
use wire_weaver_client::{Commander, DeviceApiInfo, WwClient};
use ww_device::ws::{IoSocket, WsBuffers, WsConnection, ws_server};

/// URL path the simulated device serves the API at.
pub const WS_PATH: &str = "/ww";

/// TCP port the simulated device listens on by default.
pub const SIM_PORT: u16 = 8081;

/// Longest message either side sends.
const MAX_MESSAGE_LEN: usize = 1024;

/// The host side of a GPIO expander: the calls `ww_codegen!` generates for [all_gpio_api::AllGpioApi] live on it.
pub struct AllGpio {
    cmd: Commander,
}

impl WwClient for AllGpio {
    fn default_config() -> ClientConfig {
        ClientConfig::new().usb_vid_pid(0xc0de, 0xcafe)
    }

    fn try_from_cmd(cmd: Commander) -> Result<Self, wire_weaver_client::Error> {
        Ok(Self { cmd })
    }
}

impl AllGpio {
    /// What the device said about itself when it connected.
    pub fn info(&self) -> DeviceApiInfo {
        self.cmd.info()
    }
}

mod api_client {
    wire_weaver::ww_codegen!(
        all_gpio_api :: AllGpioApi for crate::AllGpio,
        client = "std_client",
    );
}

/// Serves a GPIO expander on a [SimBoard] to the WebSocket clients `incoming` yields, one at a time, until `stop`
/// resolves. The device's futures are not `Send`: run it on a current-thread runtime or in a `LocalSet`.
pub async fn simulated<I, S>(incoming: I, stop: impl Future<Output = ()>)
where
    I: Stream<Item = S> + Unpin,
    S: AsyncRead + AsyncWrite + Unpin,
{
    let board: &'static mut SimBoard = Box::leak(Box::default());
    let mut device = Device::new(board);
    let conn = WsConnection::new(IoSocket::new(incoming));
    let mut buffers = Box::new(WsBuffers::<MAX_MESSAGE_LEN>::new());
    let mut server = ws_server(
        Device::link_config(),
        &conn,
        TokioClock::new(),
        &mut buffers,
    );
    all_gpio_device::serve(&mut server, &mut device, stop).await;
}
