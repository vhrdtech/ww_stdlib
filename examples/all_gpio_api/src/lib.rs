//! API of the GPIO expander example: every GPIO bank of a board as a standard `ww_gpio::Bank`. Firmware:
//! `examples/all_gpio_device` (simulated on a PC) and `examples_mcu`'s all-GPIO bins; host: `examples/all_gpio`.
#![no_std]

use ww_api::prelude::*;

// --8<-- [start:api]
/// Every GPIO bank of a board, each a standard `ww_gpio::Bank` of pins.
#[ww_api_root]
pub trait AllGpioApi {
    ww_impl!(port[]: ww_gpio::Bank);
}
// --8<-- [end:api]
