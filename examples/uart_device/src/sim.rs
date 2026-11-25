//! A board that exists only in software, for the simulated device and the tests.

use core::fmt::Write as _;

use wire_weaver::prelude::CommonError;
use ww_device::Instant;
use ww_uart::{Baud, Count, DataBits, IDLE_CHARS_DEFAULT, Mode, Parity, StopBits, UartConfig};

use crate::{Board, PORTS_MAX, Received};

/// The port the GPS receiver sits on.
const GPS: u8 = 0;
/// The port with a jumper from its TX to its RX.
const LOOPBACK: u8 = 1;
/// The GPS receiver's line: 9600 Bd, 8 data bits, no parity, 1 stop bit.
const GPS_BAUD: u64 = 9600;
/// The receiver sends one sentence per second.
const GPS_PERIOD_US: u64 = 1_000_000;
/// The port's line after reset.
const BAUD_AT_RESET: u32 = 115_200;
/// Slowest and fastest baud rate a port takes.
const BAUD_MIN: u64 = 1200;
const BAUD_MAX: u64 = 1_000_000;
/// Longest sentence of the GPS receiver.
const SENTENCE_MAX: usize = 96;
/// Bytes the loopback holds before they arrive back.
const LOOPBACK_MAX: usize = 512;

// --8<-- [start:sim]
/// A board with two serial ports. A GPS receiver on port 0 sends one NMEA sentence per second at 9600 Bd, 8N1: a
/// GGA fix whose time counts up from 12:00:00. At any other baud rate its bytes arrive garbled and flagged as framing
/// errors. Configuring port 0 restarts the receiver: its first sentence, 12:00:01, comes one second later. Port 1 has a jumper from its TX to its RX, so every byte sent arrives back one character time later.
// --8<-- [end:sim]
#[derive(Debug, Clone)]
pub struct SimBoard {
    config: [UartConfig; PORTS_MAX],
    /// The second whose GPS sentence comes next, from 1.
    gps_second: u64,
    /// When the GPS receiver last started: second 0 of its sentences, in µs.
    gps_origin_us: u64,
    loopback: [u8; LOOPBACK_MAX],
    loopback_len: usize,
    loopback_first: Instant,
    loopback_end: Instant,
}

impl Default for SimBoard {
    fn default() -> Self {
        let config = UartConfig {
            baud_rate: Baud::base(BAUD_AT_RESET),
            data_bits: DataBits::Eight,
            parity: Parity::None,
            stop_bits: StopBits::Stop1,
            mode: Mode::Asynchronous,
            prevent_back_feed: false,
            io_level: None,
            idle_chars: Count::new(IDLE_CHARS_DEFAULT),
        };
        SimBoard {
            config: [config.clone(), config],
            gps_second: 1,
            gps_origin_us: 0,
            loopback: [0; LOOPBACK_MAX],
            loopback_len: 0,
            loopback_first: Instant::from_micros(0),
            loopback_end: Instant::from_micros(0),
        }
    }
}

impl SimBoard {
    /// One character time of `port`'s line, in µs.
    fn char_us(&self, port: u8) -> u64 {
        let config = &self.config[port as usize];
        let bits = config.frame_bits().map_or(10, |bits| u64::from(bits.get()));
        let baud = (config.baud_rate.to_f64() as u64).max(1);
        (bits * 1_000_000).div_ceil(baud)
    }

    /// When the GPS receiver's next sentence has arrived in full.
    fn gps_end(&self) -> Instant {
        let first = self.gps_origin_us + self.gps_second * GPS_PERIOD_US;
        let len = gps_sentence(self.gps_second, &mut [0; SENTENCE_MAX]) as u64;
        let char_us = (10 * 1_000_000u64).div_ceil(GPS_BAUD);
        Instant::from_micros(first + (len - 1) * char_us)
    }

    /// Whether port 0 runs at the GPS receiver's line settings.
    fn gps_line_matches(&self) -> bool {
        let config = &self.config[GPS as usize];
        config.baud_rate.to_f64() as u64 == GPS_BAUD
            && config.data_bits == DataBits::Eight
            && config.parity == Parity::None
    }
}

impl Board for SimBoard {
    fn ports(&self) -> u8 {
        PORTS_MAX as u8
    }

    fn configure(
        &mut self,
        port: u8,
        config: UartConfig,
        now: Instant,
    ) -> Result<UartConfig, CommonError<'static>> {
        let baud = config.baud_rate.to_f64() as u64;
        if !(BAUD_MIN..=BAUD_MAX).contains(&baud)
            || config.mode != Mode::Asynchronous
            || config.prevent_back_feed
            || config.io_level.is_some()
            || config.frame_bits().is_none()
        {
            return Err(CommonError::Unsupported);
        }
        self.config[port as usize] = config.clone();
        if port == GPS {
            self.gps_origin_us = now.as_micros();
            self.gps_second = 1;
        }
        Ok(config)
    }

    fn write(&mut self, port: u8, bytes: &[u8], now: Instant) {
        if port != LOOPBACK {
            // the GPS receiver doesn't listen
            return;
        }
        let char_us = self.char_us(port);
        let room = LOOPBACK_MAX - self.loopback_len;
        let bytes = &bytes[..bytes.len().min(room)];
        if self.loopback_len == 0 {
            self.loopback_first = Instant::from_micros(now.as_micros() + char_us);
            self.loopback_end = now;
        }
        self.loopback[self.loopback_len..self.loopback_len + bytes.len()].copy_from_slice(bytes);
        self.loopback_len += bytes.len();
        let from = self.loopback_end.as_micros().max(now.as_micros());
        self.loopback_end = Instant::from_micros(from + bytes.len() as u64 * char_us);
    }

    fn next_rx(&self) -> Option<Instant> {
        let gps = self.gps_end();
        if self.loopback_len > 0 && self.loopback_end < gps {
            return Some(self.loopback_end);
        }
        Some(gps)
    }

    fn read(&mut self, now: Instant, buf: &mut [u8]) -> Option<Received> {
        if self.loopback_len > 0 && self.loopback_end <= now {
            let len = self.loopback_len.min(buf.len());
            buf[..len].copy_from_slice(&self.loopback[..len]);
            self.loopback_len = 0;
            return Some(Received {
                port: LOOPBACK,
                len,
                first: self.loopback_first,
                end: self.loopback_end,
                framing_error: false,
            });
        }
        // a sentence whose time has passed while nobody asked is skipped, but the latest one
        while self.gps_origin_us + (self.gps_second + 1) * GPS_PERIOD_US <= now.as_micros() {
            self.gps_second += 1;
        }
        let end = self.gps_end();
        if end > now || buf.len() < SENTENCE_MAX {
            return None;
        }
        let len = gps_sentence(self.gps_second, buf);
        let framing_error = !self.gps_line_matches();
        if framing_error {
            for b in &mut buf[..len] {
                *b ^= 0xa5;
            }
        }
        let first = Instant::from_micros(self.gps_origin_us + self.gps_second * GPS_PERIOD_US);
        self.gps_second += 1;
        Some(Received {
            port: GPS,
            len,
            first,
            end,
            framing_error,
        })
    }
}

/// The GGA sentence of `second` into `buf`; returns its length.
fn gps_sentence(second: u64, buf: &mut [u8]) -> usize {
    let mut out = Buf { buf, len: 0 };
    let t = 12 * 3600 + second;
    let body_start = 1;
    _ = write!(
        out,
        "$GPGGA,{:02}{:02}{:02}.00,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,",
        (t / 3600) % 24,
        (t / 60) % 60,
        t % 60
    );
    let checksum = out.buf[body_start..out.len].iter().fold(0u8, |c, b| c ^ b);
    _ = write!(out, "*{checksum:02X}\r\n");
    out.len
}

/// A fixed buffer written with `write!`.
struct Buf<'a> {
    buf: &'a mut [u8],
    len: usize,
}

impl core::fmt::Write for Buf<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let end = self.len + s.len();
        let into = self.buf.get_mut(self.len..end).ok_or(core::fmt::Error)?;
        into.copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}
