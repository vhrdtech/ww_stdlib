//! API of the sensor node example: a device that tells about itself through the standard library's device contract.
//! Device: `examples/node_device` (simulated on a PC); host: `examples/node`.
#![cfg_attr(not(feature = "std"), no_std)]

use ww_api::prelude::*;

// --8<-- [start:device]
/// What every device tells about itself, from the standard library.
#[ww_trait]
pub trait Device {
    ww_impl!(firmware: ww_firmware_info::FirmwareInfo);
    ww_impl!(board: ww_board_info::BoardInfo);
    ww_impl!(uid: ww_uid::Uid);
    ww_impl!(label: ww_user_info::UserInfo);
    ww_impl!(log: ww_log::LogText);
    ww_impl!(indication: ww_indication::Indication);
}
// --8<-- [end:device]

// --8<-- [start:root]
/// A sensor node: the device contract, and what the node does.
#[ww_api_root]
pub trait Node {
    ww_impl!(device: Device);

    /// Takes a reading and logs it.
    #[effect(actuate)]
    fn measure();
}
// --8<-- [end:root]
