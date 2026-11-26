# uart

Client of the UART bridge example device (`uart_api`), and the device simulated on a PC over WebSocket.

```sh
cargo run -p uart --example uart_sim      # terminal 1: the simulated device
cargo run -p uart --example uart_bridge   # terminal 2: GPS sentences on port 0, a loopback on port 1
```

`uart_simple` and `uart_blocking` connect over USB to `examples_mcu/usb_stm32h725ig`'s UART firmware. `tests/it` runs
the simulated device in process and writes the book's outputs.
