//! What the book shows of the sensor node, checked against `docs/generated/node/`: the `ww` transcripts are
//! the text `ww` prints, from the same device the chapter runs, served in process.

use tokio::sync::oneshot;
use wire_weaver_cli::cmd::introspect::{IntrospectView, render};
use wire_weaver_client::{DynClient, in_process};

/// The URL a reader's simulated device serves at: `node_sim`'s default.
const SIM_URL: &str = "ws://127.0.0.1:8083/ww";

/// Runs `test` against a simulated sensor node at `path`, through the dynamic client `ww` and Python use.
async fn with_dyn_device<F: Future<Output = ()>>(path: &str, test: impl FnOnce(DynClient) -> F) {
    let local = tokio::task::LocalSet::new();
    let (stop_tx, stop_rx) = oneshot::channel::<()>();
    let incoming = in_process::ws_device(path);
    let device = local.spawn_local(node::simulated(incoming, async {
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
    with_dyn_device("node/book_introspect", |dev| async move {
        let view = IntrospectView {
            skip_docs: true,
            ..IntrospectView::default()
        };
        let tree = render(&dev, view).await.expect("renders");
        book_outputs::check(
            "node/introspect.txt",
            &format!("$ ww --websocket {SIM_URL} introspect -d\n{tree}"),
        );
    })
    .await;
}

/// `ww info`: the link facts `wire_weaver_cli` prints first (API, Hello), then what the device says about itself
/// through the device contract traits, as `ww_std_cli` renders it (`render_contract`).
#[tokio::test(start_paused = true)]
async fn book_output_info() {
    with_dyn_device("node/book_info", |dev| async move {
        let api = dev.device_api_info();
        let mut text = format!("$ ww --websocket {SIM_URL} info\n");
        match api.user_api_wire {
            Some(wire) => text += &format!("API        {:?} (wire {wire})\n", api.user_api_version),
            None => text += &format!("API        {:?}\n", api.user_api_version),
        }
        text += &format!("Hello      {}\n", api.generations);
        let info = ww_report::info::collect_info(&ww_report::clients::Contract::new(&dev)).await;
        text += &ww_report::report::render_contract(&info);
        book_outputs::check("node/info.txt", &text);
    })
    .await;
}
