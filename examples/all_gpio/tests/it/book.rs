//! What the book shows of the GPIO expander, checked against `docs/generated/all_gpio/`: the `ww` transcripts are
//! the text `ww` prints, from the same device the chapter runs, served in process.

use tokio::sync::oneshot;
use wire_weaver_cli::cmd::introspect::{IntrospectView, render};
use wire_weaver_client::{DynClient, in_process};

/// The URL a reader's simulated device serves at: `all_gpio_sim`'s default.
const SIM_URL: &str = "ws://127.0.0.1:8081/ww";

/// Runs `test` against a simulated GPIO expander at `path`, through the dynamic client `ww` and Python use.
async fn with_dyn_device<F: Future<Output = ()>>(path: &str, test: impl FnOnce(DynClient) -> F) {
    let local = tokio::task::LocalSet::new();
    let (stop_tx, stop_rx) = oneshot::channel::<()>();
    let incoming = in_process::ws_device(path);
    let device = local.spawn_local(all_gpio::simulated(incoming, async {
        _ = stop_rx.await;
    }));
    local
        .run_until(async {
            let config = wire_weaver_client::ClientConfig::new()
                .in_process_path(path.to_owned())
                .no_ww_usb();
            let host = DynClient::from_config(config)
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
async fn book_output_introspect() {
    console::set_colors_enabled(false);
    with_dyn_device("all_gpio/book_introspect", |dev| async move {
        let view = IntrospectView {
            skip_docs: true,
            ..IntrospectView::default()
        };
        let tree = render(&dev, view).await.expect("renders");
        book_outputs::check(
            "all_gpio/introspect.txt",
            &format!("$ ww --websocket {SIM_URL} introspect -d\n{tree}"),
        );
    })
    .await;
}
