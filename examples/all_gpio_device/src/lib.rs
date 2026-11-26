//! The GPIO expander example device ([all_gpio_api]): every pin of a board as a standard `ww_gpio` trait, the
//! server written once.
//!
//! The API is one line, `port[]: ww_gpio::Bank`. This crate answers the calls of that standard trait by asking a
//! [Board] for its pins. A device simulated on a PC serves it with [SimBoard]; a firmware serves it with a [Board]
//! over its GPIO registers. What differs between them is only the [Board] and the transport given to [serve].
#![cfg_attr(not(feature = "std"), no_std)]

mod sim;

use core::fmt::Write as _;
use core::future::Future;
use core::pin::pin;

use embassy_futures::select::{Either, select};
use wire_weaver::prelude::*;
use wire_weaver::{MessageSink, WireWeaverAsyncApiBackend};
use ww_client_server::ResourceIndex;
use ww_device::{Clock, Instant, LinkConfig, MessageRx, MessageTx, Server};
use ww_gpio::{
    BankCapabilities, BankConfig, GpioError, IoLevel, IoPinEnabledEvents, Level, Mode,
    PinCapabilities, Pull, Speed,
};
use ww_stream::tx::Counted;

pub use sim::SimBoard;

/// Longest pin name, `PA15`.
const PIN_NAME_MAX: usize = 8;

/// The IO level of every bank: 3.3 V, without level translators.
const IO_LEVEL: IoLevel = IoLevel::new(3300);

// --8<-- [start:board]
/// One pin of a board: its bank and its number in the bank, both from 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinId {
    pub bank: u8,
    pub pin: u8,
}

/// What a board gives the GPIO expander: its banks of pins, and each pin's registers.
pub trait Board {
    /// The name of each bank, in the order of `port[]`: `["PA", "PB", "PC"]`.
    fn banks(&self) -> &[&'static str];

    /// How many pins `bank` has, numbered from 0.
    fn pins(&self, bank: u8) -> u8;

    /// Sets the output register of `pin`; the pin drives it only in an output mode.
    fn set_output(&mut self, pin: PinId, level: Level);

    /// The output register of `pin`.
    fn output(&self, pin: PinId) -> Level;

    /// The level on `pin` at `now`.
    fn input(&self, pin: PinId, now: Instant) -> Level;

    /// Puts `pin` into `mode`; refuses a mode the board can't do.
    fn set_mode(&mut self, pin: PinId, mode: Mode) -> Result<(), CommonError<'static>>;

    /// The mode of `pin`.
    fn mode(&self, pin: PinId) -> Mode;

    /// Sets the pull resistor of `pin`; refuses a pull the board can't do.
    fn set_pull(&mut self, pin: PinId, pull: Pull) -> Result<(), CommonError<'static>>;

    /// The pull resistor of `pin`.
    fn pull(&self, pin: PinId) -> Pull;
}
// --8<-- [end:board]

/// The GPIO expander: the server state that the generated code dispatches to.
pub struct AllGpio {
    board: &'static mut dyn Board,
    /// Now on the server's clock, set by [serve] before each request.
    now: Instant,
    /// The `event` streams of every pin: they never open, since no pin reports edges.
    pin_events: Counted,
    stream_sessions: u32,
    /// The name of the pin last asked for in `capabilities()`.
    pin_name: name::Name,
}

impl AllGpio {
    /// A GPIO expander over the pins of `board`.
    pub fn new(board: &'static mut dyn Board) -> Self {
        AllGpio {
            board,
            now: Instant::from_micros(0),
            pin_events: Counted::new(),
            stream_sessions: 0,
            pin_name: name::Name::default(),
        }
    }

    /// The link configuration a transport needs to serve this API.
    pub fn link_config() -> LinkConfig<'static> {
        LinkConfig::new(
            all_gpio_api::ALL_GPIO_API_FULL_GID,
            all_gpio_api::ALL_GPIO_API_WIRE_VERSION,
            api_impl::api_hash(),
            api_impl::image_facts(),
            ww_client_server::COMPACT_VERSION,
        )
    }

    /// The API id a USB device names itself by.
    pub const API_ID: &'static str = api_impl::API_ID;

    /// The pin `index` addresses: `[bank, pin]`.
    fn pin(index: [ResourceIndex; 2]) -> PinId {
        PinId {
            bank: index[0].get().0 as u8,
            pin: index[1].get().0 as u8,
        }
    }
}

/// A pin name in a fixed buffer, no heap.
mod name {
    use super::PIN_NAME_MAX;

    #[derive(Default)]
    pub struct Name {
        bytes: [u8; PIN_NAME_MAX],
        len: usize,
    }

    impl Name {
        pub fn clear(&mut self) {
            self.len = 0;
        }

        pub fn as_str(&self) -> &str {
            core::str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
        }
    }

    impl core::fmt::Write for Name {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let end = self.len + s.len();
            if end > PIN_NAME_MAX {
                return Err(core::fmt::Error);
            }
            self.bytes[self.len..end].copy_from_slice(s.as_bytes());
            self.len = end;
            Ok(())
        }
    }
}

/// The handlers of `ww_gpio::Bank`: `port_` and the item's name.
impl AllGpio {
    fn valid_indices_root_port(&mut self) -> ValidIndices<'_> {
        ValidIndices::range_u32(0..self.board.banks().len() as u32)
    }

    async fn port_capabilities(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        _index: [ResourceIndex; 1],
    ) -> RpcResult<BankCapabilities<'_>> {
        Ready(BankCapabilities {
            io_levels: MaxLen::new_unchecked(RefVec::Slice {
                slice: &const { [IO_LEVEL] },
            }),
            io_level_min: None,
            io_level_max: None,
            push_pull: true,
            open_drain: true,
            input: true,
            individually_configurable_pins: true,
            custom_mode: Default::default(),
            custom_pull: Default::default(),
            custom_speed: Default::default(),
        })
    }

    async fn port_configure(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        _index: [ResourceIndex; 1],
        config: BankConfig,
    ) -> RpcResult<Result<BankConfig, wire_weaver::Error<'_, GpioError>>> {
        match config.io_level {
            None => Ready(Ok(BankConfig {
                io_level: Some(IO_LEVEL),
            })),
            Some(level) if level == IO_LEVEL => Ready(Ok(config)),
            Some(_) => Ready(Err(CommonError::Unsupported.into())),
        }
    }

    async fn port_name(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        index: [ResourceIndex; 1],
    ) -> RpcResult<&'_ str> {
        let bank = index[0].get().0 as usize;
        Ready(self.board.banks().get(bank).copied().unwrap_or(""))
    }
}

/// The handlers of `ww_gpio::Pin`: `port_pin_` and the item's name.
impl AllGpio {
    fn valid_indices_root_port_pin(&mut self, index: [ResourceIndex; 1]) -> ValidIndices<'_> {
        let bank = index[0].get().0 as u8;
        ValidIndices::range_u32(0..u32::from(self.board.pins(bank)))
    }

    // --8<-- [start:levels]
    async fn port_pin_set_output_level(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        index: [ResourceIndex; 2],
        level: Level,
    ) -> RpcResult<()> {
        self.board.set_output(Self::pin(index), level);
        Ready(())
    }

    async fn port_pin_input_level(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        index: [ResourceIndex; 2],
    ) -> RpcResult<Level> {
        Ready(self.board.input(Self::pin(index), self.now))
    }
    // --8<-- [end:levels]

    async fn port_pin_toggle(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        index: [ResourceIndex; 2],
    ) -> RpcResult<()> {
        let pin = Self::pin(index);
        let level = match self.board.output(pin) {
            Level::Low => Level::High,
            Level::High => Level::Low,
        };
        self.board.set_output(pin, level);
        Ready(())
    }

    async fn port_pin_output_level(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        index: [ResourceIndex; 2],
    ) -> RpcResult<Level> {
        Ready(self.board.output(Self::pin(index)))
    }

    /// No pin reports edges (`PinCapabilities::events` is false), so the stream never opens.
    async fn port_pin_event(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        _index: [ResourceIndex; 2],
        _enabled: IoPinEnabledEvents<'_>,
    ) -> RpcResult<Result<(), wire_weaver::Error<'_, GpioError>>> {
        Ready(Err(CommonError::Unsupported.into()))
    }

    async fn close_port_pin_event(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        _index: [ResourceIndex; 2],
    ) {
    }

    async fn stream_port_pin_event(&mut self, _index: [ResourceIndex; 2]) -> &mut Counted {
        &mut self.pin_events
    }

    fn new_stream_session(&mut self) -> u32 {
        self.stream_sessions = self.stream_sessions.wrapping_add(1);
        self.stream_sessions
    }

    // --8<-- [start:mode]
    async fn port_pin_set_mode(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        index: [ResourceIndex; 2],
        mode: Mode,
        initial: Option<Level>,
    ) -> RpcResult<Result<(), wire_weaver::Error<'_, GpioError>>> {
        let pin = Self::pin(index);
        if let Some(level) = initial {
            self.board.set_output(pin, level);
        }
        Ready(self.board.set_mode(pin, mode).map_err(Into::into))
    }
    // --8<-- [end:mode]

    async fn port_pin_mode(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        index: [ResourceIndex; 2],
    ) -> RpcResult<Mode> {
        Ready(self.board.mode(Self::pin(index)))
    }

    async fn set_port_pin_pull(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        index: [ResourceIndex; 2],
        pull: Pull,
    ) -> SetResult<wire_weaver::Error<'_, GpioError>> {
        match self.board.set_pull(Self::pin(index), pull) {
            Ok(()) => Set,
            Err(e) => SetError(e.into()),
        }
    }

    async fn get_port_pin_pull(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        index: [ResourceIndex; 2],
    ) -> GetResult<Pull, wire_weaver::Error<'_, GpioError>> {
        Value(self.board.pull(Self::pin(index)))
    }

    /// Every pin has one drive strength.
    async fn set_port_pin_speed(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        _index: [ResourceIndex; 2],
        speed: Speed,
    ) -> SetResult<wire_weaver::Error<'_, GpioError>> {
        if speed == Speed::Slow {
            Set
        } else {
            SetError(CommonError::Unsupported.into())
        }
    }

    async fn get_port_pin_speed(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        _index: [ResourceIndex; 2],
    ) -> GetResult<Speed, wire_weaver::Error<'_, GpioError>> {
        Value(Speed::Slow)
    }

    async fn port_pin_capabilities(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        index: [ResourceIndex; 2],
    ) -> RpcResult<PinCapabilities<'_>> {
        let pin = Self::pin(index);
        let bank = self
            .board
            .banks()
            .get(pin.bank as usize)
            .copied()
            .unwrap_or("");
        self.pin_name.clear();
        // a bank name of up to 4 characters and a pin number below 256 fit PIN_NAME_MAX
        _ = write!(self.pin_name, "{bank}{}", pin.pin);
        Ready(PinCapabilities {
            modes: MaxLen::new_unchecked(RefVec::Slice {
                slice: &[Mode::PushPullOutput, Mode::OpenDrainOutput, Mode::Input],
            }),
            pulls: MaxLen::new_unchecked(RefVec::Slice {
                slice: &[Pull::None, Pull::Up, Pull::Down],
            }),
            speeds: MaxLen::new_unchecked(RefVec::Slice {
                slice: &[Speed::Slow],
            }),
            events: false,
            name: MaxLen::new_unchecked(self.pin_name.as_str()),
        })
    }
}

// --8<-- [start:codegen]
mod api_impl {
    wire_weaver::ww_codegen!(
        all_gpio_api :: AllGpioApi for super::AllGpio,
        server = true, no_alloc = true, use_async = true,
        method_model = "_=immediate",
        property_model = "_=get_set",
        introspect = "with_docs",
    );
}
// --8<-- [end:codegen]

impl WireWeaverAsyncApiBackend for AllGpio {
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
        all_gpio_api::ALL_GPIO_API_FULL_GID
    }
}

/// Serves `gpio` on `server` until `stop` resolves: the one event loop of the simulated device and the firmware. A
/// firmware that never stops passes [core::future::pending].
pub async fn serve<Tx: MessageTx, Rx: MessageRx, C: Clock>(
    server: &mut Server<'_, Tx, Rx, C>,
    gpio: &mut AllGpio,
    stop: impl Future<Output = ()>,
) {
    let mut stop = pin!(stop);
    loop {
        let ready = match select(server.wait(), &mut stop).await {
            Either::First(ready) => ready,
            Either::Second(()) => return,
        };
        gpio.now = server.now();
        _ = server.handle(ready, gpio).await;
    }
}
