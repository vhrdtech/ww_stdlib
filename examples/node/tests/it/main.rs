//! The sensor node as a host sees it: the simulated device served over the in-process WebSocket
//! (`wire_weaver_client::in_process::ws_device`), on tokio's paused clock, so time moves only when both ends wait.

mod book;

use node::Node;
use tokio::sync::oneshot;
use wire_weaver::shrink_wrap::UVlq64;
use wire_weaver_client::{Run, StreamCursor, StreamEvent, in_process};

/// Runs `test` against a simulated sensor node at `path`.
async fn with_device<F: Future<Output = ()>>(path: &str, test: impl FnOnce(Node) -> F) {
    let local = tokio::task::LocalSet::new();
    let (stop_tx, stop_rx) = oneshot::channel::<()>();
    let incoming = in_process::ws_device(path);
    let device = local.spawn_local(node::simulated(incoming, async {
        _ = stop_rx.await;
    }));
    local
        .run_until(async {
            let host = Node::config(|c| c.in_process_path(path.to_owned()).no_ww_usb())
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
async fn the_log_keeps_records_from_before_the_host() {
    with_device("node/log", |n| async move {
        n.measure().call().await.unwrap();
        let device = n.device();
        let status = device.log().records_status().read().await.unwrap();
        let cursor = StreamCursor {
            session: status.session.unwrap(),
            pos: UVlq64(0),
        };
        let mut log = device.log().records_resume(cursor).call().await.unwrap();
        let mut got = vec![];
        while got.len() < 2 {
            if let StreamEvent::Item {
                pos, run, value, ..
            } = log.recv_event().await.unwrap()
            {
                got.push((pos, run, value.message.to_string()));
            }
        }
        // the newest record comes first, live; the one from before the host connected follows as the backfill
        assert_eq!(
            got,
            [
                (1, Run::Live, "reading 1: 21.50 °C".to_owned()),
                (0, Run::Backfill, "booted".to_owned())
            ]
        );
    })
    .await;
}
