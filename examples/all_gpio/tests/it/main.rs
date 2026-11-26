//! The GPIO expander as a host sees it: the simulated device served over the in-process WebSocket
//! (`wire_weaver_client::in_process::ws_device`), on tokio's paused clock, so time moves only when both ends wait.

mod book;

use std::time::Duration;

use all_gpio::AllGpio;
use tokio::sync::oneshot;
use wire_weaver_client::in_process;
use ww_gpio::{Level, Mode, Pull};

/// Runs `test` against a simulated GPIO expander at `path`.
async fn with_device<F: Future<Output = ()>>(path: &str, test: impl FnOnce(AllGpio) -> F) {
    let local = tokio::task::LocalSet::new();
    let (stop_tx, stop_rx) = oneshot::channel::<()>();
    let incoming = in_process::ws_device(path);
    let device = local.spawn_local(all_gpio::simulated(incoming, async {
        _ = stop_rx.await;
    }));
    local
        .run_until(async {
            let host = AllGpio::config(|c| c.in_process_path(path.to_owned()).no_ww_usb())
                .connect()
                .await
                .expect("connects");
            test(host).await;
            _ = stop_tx.send(());
            device.await.expect("the device runs to its stop");
        })
        .await;
}

#[tokio::test(start_paused = true)]
async fn three_banks_of_sixteen_pins_named_like_an_mcu() {
    with_device("all_gpio/banks", |d| async move {
        let ports: Vec<u32> = d
            .port_valid_indices()
            .read()
            .await
            .unwrap()
            .iter()
            .collect();
        assert_eq!(ports, [0, 1, 2]);
        let mut names = vec![];
        for port in ports {
            names.push(d.port(port).name().call().await.unwrap());
            let pins: Vec<u32> = d
                .port(port)
                .pin_valid_indices()
                .read()
                .await
                .unwrap()
                .iter()
                .collect();
            assert_eq!(pins, (0..16).collect::<Vec<_>>());
        }
        assert_eq!(names, ["PA", "PB", "PC"]);
        let caps = d.port(0).pin(5).capabilities().call().await.unwrap();
        assert_eq!(caps.name.as_str(), "PA5");
        assert!(!caps.events);
    })
    .await;
}

#[tokio::test(start_paused = true)]
async fn an_output_reads_back_and_an_open_input_reads_its_pull() {
    with_device("all_gpio/levels", |d| async move {
        let port = d.port(0);
        let led = port.pin(5);
        led.set_mode(Mode::PushPullOutput, Some(Level::High))
            .call()
            .await
            .unwrap();
        assert_eq!(led.input_level().call().await.unwrap(), Level::High);
        led.toggle().call().await.unwrap();
        assert_eq!(led.output_level().call().await.unwrap(), Level::Low);
        assert_eq!(led.mode().call().await.unwrap(), Mode::PushPullOutput);

        let open = port.pin(0);
        assert_eq!(open.input_level().call().await.unwrap(), Level::Low);
        open.write_pull(Pull::Up).write().await.unwrap();
        assert_eq!(open.input_level().call().await.unwrap(), Level::High);
        assert!(open.set_mode(Mode::Analog, None).call().await.is_err());
    })
    .await;
}

#[tokio::test(start_paused = true)]
async fn the_button_is_pressed_for_the_first_second_of_every_three() {
    with_device("all_gpio/button", |d| async move {
        let port = d.port(2);
        let button = port.pin(13);
        let start = tokio::time::Instant::now();
        let mut seen = vec![];
        for _ in 0..6 {
            seen.push(button.input_level().call().await.unwrap());
            tokio::time::sleep_until(start + Duration::from_millis(500) * (seen.len() as u32))
                .await;
        }
        assert_eq!(
            seen,
            [
                Level::Low,
                Level::Low,
                Level::High,
                Level::High,
                Level::High,
                Level::High
            ]
        );
    })
    .await;
}
