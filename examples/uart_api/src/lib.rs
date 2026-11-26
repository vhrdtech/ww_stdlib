//! API of the UART bridge example: the serial ports of a board as standard `ww_uart::Uart`s. Firmware:
//! `examples/uart_device` (simulated on a PC) and `examples_mcu/usb_stm32h725ig`'s UART bin; host: `examples/uart`.
#![no_std]

use ww_api::prelude::*;

// --8<-- [start:api]
/// The serial ports of a board, each a standard `ww_uart::Uart`.
#[ww_api_root]
pub trait UartBridge {
    ww_impl!(uart[]: ww_uart::Uart);
}
// --8<-- [end:api]
