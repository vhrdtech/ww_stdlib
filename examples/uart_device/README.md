# uart_device

The UART bridge example device (`uart_api`): the server that maps `ww_uart`'s `Uart` onto a board's serial ports,
written once. A board gives it a `Board`: `SimBoard` exists only in software, with a GPS receiver on port 0 and a
loopback jumper on port 1, served over WebSocket by `examples/uart` (`uart_sim`).
