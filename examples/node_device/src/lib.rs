//! The sensor node example device ([node_api]): the standard library's device contract served once, for a device
//! simulated on a PC and for a firmware.
//!
//! What never changes, the firmware, the board and the unit's id, is constants: the generated code answers them
//! without a handler. The label, the text log and the indication have handlers. [serve] runs the event loop over
//! any transport.
#![cfg_attr(not(feature = "std"), no_std)]

use core::fmt::Write as _;
use core::future::Future;
use core::pin::pin;
use core::time::Duration;

use embassy_futures::select::{Either, select};
use wire_weaver::prelude::*;
use wire_weaver::{MessageSink, WireWeaverAsyncApiBackend};
use ww_board_info::{BoardSummary, RevisionSource};
use ww_device::{Clock, Instant, LinkConfig, MessageRx, MessageTx, Server};
use ww_firmware_info::{BuildProfile, FirmwareSummary, Version};
use ww_indication::Mode as IndicationMode;
use ww_log::{LevelFilter, LogRecord, Severity};
use ww_stream::store::RamRing;
use ww_stream::tx::Reliable;
use ww_uid::{UidKind, UidSource, UniqueId};
use ww_user_info::{NameMax, NotesMax, UserInfoCaps, UserInfoError};

/// Longest name a user gives the node.
const NAME_MAX: usize = 32;
/// Bytes of log records the node keeps until a host acknowledges them.
const LOG_STORE_BYTES: usize = 4096;
/// Longest log message.
const MESSAGE_MAX: usize = 64;
/// How often the event loop sends what the log stream holds.
const POLL_PERIOD: Duration = Duration::from_millis(100);
/// The unit's unique id: in a firmware, the chip's.
const CHIP_ID: [u8; 8] = [0x4e, 0x4f, 0x44, 0x45, 0x00, 0x00, 0x00, 0x01];

/// The sensor node: the server state that the generated code dispatches to.
pub struct Node {
    /// The name a user gave the unit, kept in RAM only.
    name: [u8; NAME_MAX],
    name_len: usize,
    level: LevelFilter,
    log: Reliable<RamRing<LOG_STORE_BYTES>>,
    indication: IndicationMode,
    /// Until when the indicators blink to identify the unit.
    identify_until: Option<Instant>,
    readings: u32,
    stream_sessions: u32,
    /// Now on the server's clock, set by [serve] before each request.
    now: Instant,
    buf: [u8; 512],
}

impl Node {
    /// A node as after reset: no name, logging at `Info`, its log stream started so that what it logs before a host
    /// connects is kept.
    pub async fn boot() -> Self {
        let mut node = Node {
            name: [0; NAME_MAX],
            name_len: 0,
            level: LevelFilter::Info,
            log: Reliable::new(RamRing::new()),
            indication: IndicationMode::Normal,
            identify_until: None,
            readings: 0,
            stream_sessions: 0,
            now: Instant::from_micros(0),
            buf: [0; 512],
        };
        let session = node.new_stream_session();
        // a RAM store can't fail to start
        _ = node.log.start_session(session).await;
        node.log(Severity::Info, "booted").await;
        node
    }

    /// The link configuration a transport needs to serve this API.
    pub fn link_config() -> LinkConfig<'static> {
        LinkConfig::new(
            node_api::NODE_FULL_GID,
            node_api::NODE_WIRE_VERSION,
            api_impl::api_hash(),
            api_impl::image_facts(),
            ww_client_server::COMPACT_VERSION,
        )
    }

    /// The API id a USB device names itself by.
    pub const API_ID: &'static str = api_impl::API_ID;

    /// Whether the indicators blink to identify the unit now.
    pub fn identifying(&self) -> bool {
        self.identify_until.is_some_and(|until| self.now < until)
    }

    // --8<-- [start:log]
    /// Logs `message` if the level lets it through: one record of the `log` stream, kept until a host has it.
    async fn log(&mut self, severity: Severity, message: &str) {
        if !self.level.allows(severity) {
            return;
        }
        let record = LogRecord {
            severity,
            target: MaxLen::new_unchecked("node"),
            message: MaxLen::new_unchecked(message),
            truncated: false,
        };
        let t = ww_si::Second::micro(self.now.as_micros() as u32);
        _ = api_impl::stream_data_ser()
            .device()
            .log()
            .records_push(&mut self.log, &record, 1, Some(t), &mut self.buf)
            .await;
    }
    // --8<-- [end:log]
}

/// What never changes: answered by the generated code, with no handler.
// --8<-- [start:consts]
impl api_impl::NodeConsts for Node {
    const DEVICE_FIRMWARE_FIRMWARE: FirmwareSummary<'static> = FirmwareSummary {
        name: MaxLen::new_unchecked("node_sim"),
        version: Version::full(0, 1, 0, None, None),
        git: None,
        built: None,
        profile: BuildProfile::Debug,
    };

    const DEVICE_FIRMWARE_BUILD_ID: MaxLen<RefVec<'static, u8>, 32> =
        MaxLen::new_unchecked(RefVec::new_bytes(&[]));

    const DEVICE_BOARD_BOARD: BoardSummary<'static> = BoardSummary {
        name: MaxLen::new_unchecked("sensor node"),
        revision: MaxLen::new_unchecked("A"),
        revision_source: RevisionSource::Assumed,
        variant: MaxLen::new_unchecked("simulated"),
        bom_variant: MaxLen::new_unchecked(""),
        serial: MaxLen::new_unchecked("SN-0001"),
        build_date: None,
    };

    const DEVICE_UID_IDS: MaxLen<RefVec<'static, UniqueId<'static>>, 4> =
        MaxLen::new_unchecked(RefVec::Slice {
            slice: &[UniqueId {
                kind: UidKind::ChipUnique,
                source: UidSource::InternalIc,
                value: MaxLen::new_unchecked(RefVec::new_bytes(&CHIP_ID)),
            }],
        });

    const DEVICE_LABEL_CAPABILITIES: UserInfoCaps = UserInfoCaps {
        name_max: NameMax::bounded(NAME_MAX as u8),
        notes_max: NotesMax::bounded(0),
        persistent: false,
    };
}
// --8<-- [end:consts]

/// The handlers.
impl Node {
    // --8<-- [start:measure]
    async fn measure(&mut self, _cx: &mut Context<'_, impl EventOut>) -> RpcResult<()> {
        self.readings += 1;
        let mut message = Message::default();
        // a reading number and a fixed temperature fit MESSAGE_MAX
        _ = write!(message, "reading {}: 21.50 °C", self.readings);
        self.log(Severity::Info, message.as_str()).await;
        Ready(())
    }
    // --8<-- [end:measure]

    // --8<-- [start:label]
    async fn set_device_label_name(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        value: MaxLen<&'_ str, 32>,
    ) -> SetResult<wire_weaver::Error<'_, UserInfoError>> {
        let name = value.as_bytes();
        self.name[..name.len()].copy_from_slice(name);
        self.name_len = name.len();
        Set
    }

    async fn get_device_label_name(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
    ) -> GetResult<MaxLen<&'_ str, 32>, wire_weaver::Error<'_, UserInfoError>> {
        let name = core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("");
        Value(MaxLen::new_unchecked(name))
    }
    // --8<-- [end:label]

    async fn device_log_records(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
    ) -> RpcResult<Result<(), CommonError<'_>>> {
        Ready(Ok(()))
    }

    async fn close_device_log_records(&mut self, _cx: &mut Context<'_, impl EventOut>) {}

    async fn stream_device_log_records(&mut self) -> &mut Reliable<RamRing<LOG_STORE_BYTES>> {
        &mut self.log
    }

    async fn set_device_log_level(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        value: LevelFilter,
    ) -> SetResult<CommonError<'_>> {
        self.level = value;
        Set
    }

    async fn get_device_log_level(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
    ) -> GetResult<LevelFilter, CommonError<'_>> {
        Value(self.level)
    }

    async fn set_device_indication_mode(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        value: IndicationMode,
    ) -> SetResult<CommonError<'_>> {
        self.indication = value;
        Set
    }

    async fn get_device_indication_mode(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
    ) -> GetResult<IndicationMode, CommonError<'_>> {
        Value(self.indication)
    }

    async fn device_indication_identify(
        &mut self,
        _cx: &mut Context<'_, impl EventOut>,
        duration: ww_si::Second,
    ) -> RpcResult<Result<(), CommonError<'_>>> {
        if self.identifying() {
            return Ready(Err(CommonError::Busy));
        }
        let us = (duration.to_f64() * 1e6) as u64;
        self.identify_until = Some(Instant::from_micros(self.now.as_micros() + us));
        self.log(Severity::Info, "identify").await;
        Ready(Ok(()))
    }

    fn new_stream_session(&mut self) -> u32 {
        self.stream_sessions = self.stream_sessions.wrapping_add(1);
        self.stream_sessions
    }
}

/// A log message in a fixed buffer, no heap.
struct Message {
    bytes: [u8; MESSAGE_MAX],
    len: usize,
}

impl Default for Message {
    fn default() -> Self {
        Message {
            bytes: [0; MESSAGE_MAX],
            len: 0,
        }
    }
}

impl Message {
    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
    }
}

impl core::fmt::Write for Message {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let end = self.len + s.len();
        let into = self.bytes.get_mut(self.len..end).ok_or(core::fmt::Error)?;
        into.copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

// --8<-- [start:codegen]
mod api_impl {
    wire_weaver::ww_codegen!(
        node_api :: Node for super::Node,
        server = true, no_alloc = true, use_async = true,
        method_model = "_=immediate",
        property_model = "_=get_set",
        // optional items this node doesn't have
        omit = [
            "device.firmware.BUILD_RECORD",
            "device.board.REWORKS",
            "device.board.set_reworks",
            "device.label.notes",
            "device.indication.brightness",
            "device.indication.NAMES",
        ],
        introspect = "with_docs",
    );
}
// --8<-- [end:codegen]

impl WireWeaverAsyncApiBackend for Node {
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
        node_api::NODE_FULL_GID
    }
}

/// Serves `node` on `server` until `stop` resolves: the one event loop of the simulated device and the firmware.
/// Every 100 ms it sends what the log stream holds. A firmware that never stops passes
/// [core::future::pending].
pub async fn serve<Tx: MessageTx, Rx: MessageRx, C: Clock>(
    server: &mut Server<'_, Tx, Rx, C>,
    node: &mut Node,
    stop: impl Future<Output = ()>,
) {
    let mut stop = pin!(stop);
    loop {
        let at = Instant::from_micros(server.now().as_micros() + POLL_PERIOD.as_micros() as u64);
        let ready = match select(server.wait_until(Some(at)), &mut stop).await {
            Either::First(ready) => ready,
            Either::Second(()) => return,
        };
        node.now = server.now();
        if !ready.is_due() {
            _ = server.handle(ready, node).await;
        }
        let now_ms = node.now.as_micros() / 1000;
        let mut buf = [0u8; 512];
        _ = node
            .poll_streams(&mut server.sink(), now_ms, &mut buf, LOG_STORE_BYTES)
            .await;
    }
}
