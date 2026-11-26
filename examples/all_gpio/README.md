# all_gpio

Client of the GPIO expander example device (`all_gpio_api`), and the device simulated on a PC over WebSocket.

```sh
cargo run -p all_gpio --example all_gpio_sim     # terminal 1: the simulated device
cargo run -p all_gpio --example all_gpio_blink   # terminal 2: blink PA5, read PC13
```

The other examples (`info.rs`, `mode*.rs`, `multi_read.rs`, `high_level.rs`) connect over USB to a board running
the all-GPIO firmware of `examples_mcu/`. `tests/it` runs the simulated device in process and writes the book's
outputs.
