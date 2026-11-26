//! The UART bridge as a host sees it: the simulated device served over the in-process WebSocket
//! (`wire_weaver_client::in_process::ws_device`), on tokio's paused clock, so time moves only when both ends wait.

mod book;

use tokio::sync::oneshot;
use uart::UartBridge;
use wire_weaver_client::{StreamEvent, in_process};
use ww_uart::{Baud, Count, DataBits, IDLE_CHARS_DEFAULT, Mode, Parity, StopBits, UartConfig};

/// Runs `test` against a simulated UART bridge at `path`.
async fn with_device<F: Future<Output = ()>>(path: &str, test: impl FnOnce(UartBridge) -> F) {
    let local = tokio::task::LocalSet::new();
    let (stop_tx, stop_rx) = oneshot::channel::<()>();
    let incoming = in_process::ws_device(path);
    let device = local.spawn_local(uart::simulated(incoming, async {
        _ = stop_rx.await;
    }));
    local
        .run_until(async {
            let host = UartBridge::config(|c| c.in_process_path(path.to_owned()).no_ww_usb())
                .connect()
                .await
                .expect("connects");
            test(host).await;
            _ = stop_tx.send(());
            device.await.expect("the device runs to its stop");
        })
        .await;
}

fn line(baud: u32) -> UartConfig {
    UartConfig {
        baud_rate: Baud::base(baud),
        data_bits: DataBits::Eight,
        parity: Parity::None,
        stop_bits: StopBits::Stop1,
        mode: Mode::Asynchronous,
        prevent_back_feed: false,
        io_level: None,
        idle_chars: Count::new(IDLE_CHARS_DEFAULT),
    }
}

#[tokio::test(start_paused = true)]
async fn the_gps_sends_one_sentence_per_second_at_9600_baud() {
    with_device("uart/gps", |d| async move {
        let gps = d.uart(0);
        gps.configure(line(9600)).call().await.unwrap();
        let mut rx = gps.rx().call().await.unwrap();
        let mut firsts = vec![];
        while firsts.len() < 2 {
            if let StreamEvent::Item { t, value, .. } = rx.recv_event().await.unwrap() {
                assert!(value.flags.is_none());
                let text = String::from_utf8(value.bytes.to_vec()).unwrap();
                // configuring the port restarts the receiver: its sentences count from 12:00:01
                let time = format!("$GPGGA,12000{}.00,", firsts.len() + 1);
                assert!(text.starts_with(&time), "{text}");
                assert!(text.ends_with("\r\n"), "{text}");
                // at 9600 Bd and 10 bits a character, the last of n characters arrives n - 1 times 1042 µs after the
                // first
                let first = t.unwrap().to_f64();
                let end = value.end.unwrap().to_f64();
                let chars_after = (text.len() - 1) as f64;
                assert!(
                    (end - first - chars_after * 0.001042).abs() < 1e-6,
                    "{first} {end}"
                );
                firsts.push(first);
            }
        }
        assert_eq!(firsts[1] - firsts[0], 1.0);
    })
    .await;
}

#[tokio::test(start_paused = true)]
async fn at_the_wrong_baud_rate_the_sentence_arrives_garbled() {
    with_device("uart/garbled", |d| async move {
        let gps = d.uart(0);
        let mut rx = gps.rx().call().await.unwrap();
        let chunk = rx.recv().await.unwrap();
        assert!(chunk.flags.expect("flags").framing_error);
        assert!(!chunk.bytes.starts_with(b"$GP"));
    })
    .await;
}

#[tokio::test(start_paused = true)]
async fn the_loopback_sends_every_byte_back() {
    with_device("uart/loopback", |d| async move {
        let port = d.uart(1);
        let mut rx = port.rx().call().await.unwrap();
        let mut tx = port.tx().await.unwrap();
        tx.send_bytes(b"hello").await.unwrap();
        let chunk = rx.recv().await.unwrap();
        assert_eq!(chunk.bytes.as_slice(), b"hello");
        assert_eq!(port.tx_pending().call().await.unwrap().get(), 0);
    })
    .await;
}

#[tokio::test(start_paused = true)]
async fn a_line_the_port_cant_do_is_refused() {
    with_device("uart/refused", |d| async move {
        let port = d.uart(1);
        assert!(port.configure(line(5_000_000)).call().await.is_err());
        let mut flow = line(9600);
        flow.mode = Mode::HardwareFlowControl;
        assert!(port.configure(flow).call().await.is_err());
    })
    .await;
}
