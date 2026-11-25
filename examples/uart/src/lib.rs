//! Client of the UART bridge example device ([uart_api]), and the device simulated on a PC.
//!
//! [UartBridge] is the host side: the generated calls `uart(i).configure(..)`, `uart(i).rx()`, `uart(i).tx()`
//! and the rest of `ww_uart`'s `Uart`. [simulated] runs the device itself, `uart_device` on its [SimBoard], over
//! WebSocket: from a TCP listener in the `uart_sim` example, or in process for tests.
//!
//! ```text
//! terminal 1: cargo run -p uart --example uart_sim
//! terminal 2: cargo run -p uart --example uart_bridge
//! ```

use std::future::Future;

use futures_util::Stream;
use futures_util::io::{AsyncRead, AsyncWrite};
pub use uart_device::SimBoard;
use uart_device::UartBridge as Device;
pub use wire_weaver_client::ClientConfig;
use wire_weaver_client::in_process::TokioClock;
use wire_weaver_client::{Commander, DeviceApiInfo, WwClient};
use ww_device::ws::{IoSocket, WsBuffers, WsConnection, ws_server};

/// URL path the simulated device serves the API at.
pub const WS_PATH: &str = "/ww";

/// TCP port the simulated device listens on by default.
pub const SIM_PORT: u16 = 8082;

/// Longest message either side sends.
const MAX_MESSAGE_LEN: usize = 1024;

/// The host side of a UART bridge: the calls `ww_codegen!` generates for [uart_api::UartBridge] live on it.
pub struct UartBridge {
    cmd: Commander,
}

impl WwClient for UartBridge {
    fn default_config() -> ClientConfig {
        ClientConfig::new().usb_vid_pid(0xc0de, 0xcafe)
    }

    fn try_from_cmd(cmd: Commander) -> Result<Self, wire_weaver_client::Error> {
        Ok(Self { cmd })
    }
}

impl UartBridge {
    /// What the device said about itself when it connected.
    pub fn info(&self) -> DeviceApiInfo {
        self.cmd.info()
    }
}

mod api_client {
    wire_weaver::ww_codegen!(
        uart_api :: UartBridge for crate::UartBridge,
        client = "std_client",
    );
}

/// Serves a UART bridge on a [SimBoard] to the WebSocket clients `incoming` yields, one at a time, until `stop`
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
    uart_device::serve(&mut server, &mut device, stop).await;
}
