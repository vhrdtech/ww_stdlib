//! The UART bridge example device ([uart_api]): the serial ports of a board as standard `ww_uart` traits, the
//! server written once.
//!
//! The API is one line, `uart[]: ww_uart::Uart`. This crate answers the calls of that standard trait by asking a
//! [Board] for its ports, and sends what a port receives up its `rx` stream. A device simulated on a PC serves it
//! with [SimBoard]; a firmware serves it with a [Board] over its UART peripherals. What differs between them is only
//! the [Board] and the transport given to [serve].
#![cfg_attr(not(feature = "std"), no_std)]

mod sim;

use core::future::Future;
use core::pin::pin;

use embassy_futures::select::{Either, select};
use wire_weaver::prelude::*;
use wire_weaver::{MessageSink, WireWeaverAsyncApiBackend};
use ww_client_server::{ResourceIndex, StreamSideband};
use ww_device::{Clock, Instant, LinkConfig, MessageRx, MessageTx, Server};
use ww_stream::sink::SinkCredit;
use ww_stream::tx::Counted;
use ww_uart::{
    Baud, Bytes, Capabilities, Pin, RxChunk, RxFlags, Second, UART_CHUNK_MAX, UartConfig,
};

pub use sim::SimBoard;

/// Ports a board can have at most.
pub const PORTS_MAX: usize = 2;

/// Bytes each port's transmit buffer holds: the credit of its `tx` sink.
pub const TX_BUFFER_BYTES: u32 = 512;

// --8<-- [start:board]
/// Bytes a port received back to back, ready to send up its `rx` stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Received {
    /// The port, from 0.
    pub port: u8,
    /// How many bytes [Board::read] put into its buffer.
    pub len: usize,
    /// When the first byte arrived.
    pub first: Instant,
    /// When the last byte arrived.
    pub end: Instant,
    /// The bytes did not fit the configured line settings.
    pub framing_error: bool,
}

/// What a board gives the UART bridge: its serial ports.
pub trait Board {
    /// How many ports the board has, at most [PORTS_MAX].
    fn ports(&self) -> u8;

    /// Configures `port` at `now`; answers what was actually set, or refuses a setting the port can't do.
    fn configure(
        &mut self,
        port: u8,
        config: UartConfig,
        now: Instant,
    ) -> Result<UartConfig, CommonError<'static>>;

    /// Sends `bytes` out of `port`, starting at `now`.
    fn write(&mut self, port: u8, bytes: &[u8], now: Instant);

    /// When the next received chunk is complete, if one is on its way.
    fn next_rx(&self) -> Option<Instant>;

    /// A chunk complete by `now`, copied into `buf`.
    fn read(&mut self, now: Instant, buf: &mut [u8]) -> Option<Received>;
}
// --8<-- [end:board]

/// The UART bridge: the server state that the generated code dispatches to.
pub struct UartBridge {
    board: &'static mut dyn Board,
    /// Position and loss accounting of each port's counted `rx` stream.
    rx: [Counted; PORTS_MAX],
    /// Credit of each port's `tx` sink: the host never sends more than the transmit buffer holds.
    tx_credit: [SinkCredit; PORTS_MAX],
    stream_sessions: u32,
    /// Now on the server's clock, set by [serve] before each request.
    now: Instant,
    buf: [u8; UART_CHUNK_MAX],
    /// The IO level of every port: 3.3 V, without level translators.
    io_levels: [ww_si::Volt; 1],
}

impl UartBridge {
    /// A UART bridge over the ports of `board`.
    pub fn new(board: &'static mut dyn Board) -> Self {
        UartBridge {
            board,
            rx: [Counted::new(), Counted::new()],
            tx_credit: [
                SinkCredit::new(TX_BUFFER_BYTES),
                SinkCredit::new(TX_BUFFER_BYTES),
            ],
            stream_sessions: 0,
            now: Instant::from_micros(0),
            buf: [0; UART_CHUNK_MAX],
            io_levels: [ww_si::Volt::milli(3300u32)],
        }
    }

    /// The link configuration a transport needs to serve this API.
    pub fn link_config() -> LinkConfig<'static> {
        LinkConfig::new(
            uart_api::UART_BRIDGE_FULL_GID,
            uart_api::UART_BRIDGE_WIRE_VERSION,
            api_impl::api_hash(),
            api_impl::image_facts(),
            ww_client_server::COMPACT_VERSION,
        )
    }

    /// The API id a USB device names itself by.
    pub const API_ID: &'static str = api_impl::API_ID;

    fn port(index: [ResourceIndex; 1]) -> usize {
        index[0].get().0 as usize
    }

    /// Sends every chunk the board completed by now up its port's `rx` stream; with no host up, counts it lost.
    async fn send_received(&mut self, out: &mut EventWriter<'_, impl MessageSink>, up: bool) {
        while let Some(received) = self.board.read(self.now, &mut self.buf) {
            let port = received.port as usize;
            let rx = &mut self.rx[port];
            if !up {
                rx.lost(received.len as u64);
                continue;
            }
            // --8<-- [start:send]
            let chunk = RxChunk {
                end: Some(Second::micro(received.end.as_micros() as u32)),
                flags: received.framing_error.then_some(RxFlags {
                    parity_error: false,
                    framing_error: true,
                    overrun: false,
                    break_detected: false,
                }),
                bytes: MaxLen::new_unchecked(RefVec::new_bytes(&self.buf[..received.len])),
            };
            // the unit of `rx` is a byte; the head's time is the first byte's
            let first = Second::micro(received.first.as_micros() as u32);
            _ = api_impl::stream_data_ser()
                .uart(received.port.into())
                .rx_send(rx, &chunk, received.len as u64, Some(first), out)
                .await;
            // --8<-- [end:send]
        }
    }
}

/// The handlers of `ww_uart::Uart`: `uart_` and the item's name.
impl UartBridge {
    fn valid_indices_root_uart(&mut self) -> ValidIndices<'_> {
        ValidIndices::range_u32(0..u32::from(self.board.ports()))
    }

    /// The stream is always on: what arrives while it is closed counts as lost.
    async fn uart_rx(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        _index: [ResourceIndex; 1],
    ) -> RpcResult<()> {
        Ready(())
    }

    async fn close_uart_rx(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        _index: [ResourceIndex; 1],
    ) {
    }

    async fn stream_uart_rx(&mut self, index: [ResourceIndex; 1]) -> &mut Counted {
        &mut self.rx[Self::port(index)]
    }

    fn new_stream_session(&mut self) -> u32 {
        self.stream_sessions = self.stream_sessions.wrapping_add(1);
        self.stream_sessions
    }

    async fn sideband_uart_tx(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        _index: [ResourceIndex; 1],
        _sideband: StreamSideband,
    ) -> Option<StreamSideband> {
        None
    }

    fn credit_uart_tx(&mut self, index: [ResourceIndex; 1]) -> &mut SinkCredit {
        &mut self.tx_credit[Self::port(index)]
    }

    // --8<-- [start:tx]
    async fn write_uart_tx(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        index: [ResourceIndex; 1],
        data: &shrink_wrap::tail_bytes::TailBytes<'_>,
    ) {
        let port = Self::port(index);
        self.board.write(port as u8, data.0, self.now);
        // the board took the bytes: their room in the transmit buffer is free again
        self.tx_credit[port].free(data.0.len() as u32);
    }
    // --8<-- [end:tx]

    async fn uart_capabilities(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        _index: [ResourceIndex; 1],
    ) -> RpcResult<Capabilities<'_>> {
        Ready(Capabilities {
            min_baud_rate: Baud::base(1200u32),
            max_baud_rate: Baud::base(1_000_000u32),
            io_levels: MaxLen::new_unchecked(RefVec::Slice {
                slice: &self.io_levels,
            }),
            io_level_min: None,
            io_level_max: None,
            hw_flow_control: false,
            sw_flow_control: false,
            high_z_mode: false,
            test_mode: false,
            back_feed_detector: false,
            rx_buffer: Bytes::new(UART_CHUNK_MAX as u32),
            tx_buffer: Bytes::new(TX_BUFFER_BYTES),
        })
    }

    // --8<-- [start:configure]
    async fn uart_configure(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        index: [ResourceIndex; 1],
        config: UartConfig,
    ) -> RpcResult<Result<UartConfig, CommonError<'_>>> {
        Ready(
            self.board
                .configure(Self::port(index) as u8, config, self.now),
        )
    }
    // --8<-- [end:configure]

    /// No port has a test mode.
    async fn uart_set_pin_level(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        _index: [ResourceIndex; 1],
        _pin: Pin,
        _is_high: bool,
    ) -> RpcResult<Result<(), CommonError<'_>>> {
        Ready(Err(CommonError::WrongState {
            reason: MaxLen::new_unchecked("no test mode on this board"),
        }))
    }

    async fn uart_send_break(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        _index: [ResourceIndex; 1],
        _duration: Second,
    ) -> RpcResult<Result<(), CommonError<'_>>> {
        Ready(Err(CommonError::Refused {
            reason: MaxLen::new_unchecked("no break on this board"),
        }))
    }

    /// The board sends what it is given at once.
    async fn uart_tx_pending(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        _index: [ResourceIndex; 1],
    ) -> RpcResult<Bytes<u32>> {
        Ready(Bytes::new(0))
    }
}

// --8<-- [start:codegen]
mod api_impl {
    wire_weaver::ww_codegen!(
        uart_api :: UartBridge for super::UartBridge,
        server = true, no_alloc = true, use_async = true,
        // the ports can't watch a link of two other devices
        omit = ["uart.monitor"],
        method_model = "_=immediate",
        property_model = "_=get_set",
        introspect = "with_docs",
    );
}
// --8<-- [end:codegen]

impl WireWeaverAsyncApiBackend for UartBridge {
    type Medium = ();

    async fn process_bytes<'a>(
        &mut self,
        out: &mut EventWriter<'_, impl MessageSink>,
        medium: (),
        data: &[u8],
        scratch: &'a mut [u8],
    ) -> Result<&'a [u8], shrink_wrap::Error> {
        self.process_request_bytes(data, scratch, out, medium).await
    }

    fn version(&self) -> FullVersion<'_> {
        uart_api::UART_BRIDGE_FULL_GID
    }
}

// --8<-- [start:serve]
/// Serves `bridge` on `server` until `stop` resolves: the one event loop of the simulated device and the firmware.
/// Between requests it sleeps until the board's next received chunk is complete. A firmware that never stops
/// passes [core::future::pending].
pub async fn serve<Tx: MessageTx, Rx: MessageRx, C: Clock>(
    server: &mut Server<'_, Tx, Rx, C>,
    bridge: &mut UartBridge,
    stop: impl Future<Output = ()>,
) {
    let mut stop = pin!(stop);
    loop {
        let at = bridge.board.next_rx();
        let ready = match select(server.wait_until(at), &mut stop).await {
            Either::First(ready) => ready,
            Either::Second(()) => return,
        };
        bridge.now = server.now();
        if ready.is_due() {
            let up = server.is_up();
            bridge.send_received(&mut server.sink(), up).await;
        } else {
            _ = server.handle(ready, bridge).await;
        }
    }
}
// --8<-- [end:serve]
