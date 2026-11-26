# uart_api

API of the UART bridge example: the serial ports of a board as standard `ww_uart::Uart`s, in one line.

- Device: `examples/uart_device` (simulated on a PC by `examples/uart`), and the UART firmware of
  `examples_mcu/usb_stm32h725ig`.
- Host: `examples/uart` (the generated client, `uart_bridge`) and `examples/stdlib_py` (`uart.py`).

The crate is `no_std`; its `std` feature turns on the owned types for host code.
