# node

Client of the sensor node example device (`node_api`), and the device simulated on a PC over WebSocket.

```sh
cargo run -p node --example node_sim     # terminal 1: the simulated device
cargo run -p node --example node_about   # terminal 2: what it tells about itself, and its log
```

`tests/it` runs the same simulated device in process, on tokio's paused clock, and writes the book's outputs.
