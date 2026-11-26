# all_gpio_api

API of the GPIO expander example: every GPIO bank of a board as a standard `ww_gpio::Bank`, in one line.

- Device: `examples/all_gpio_device` (simulated on a PC by `examples/all_gpio`), and the all-GPIO firmware of
  `examples_mcu/nucleo_h743zi2` and `examples_mcu/usb_stm32h725ig`.
- Host: `examples/all_gpio` (the generated client, `all_gpio_blink`) and `examples/stdlib_py` (`gpio.py`).

The crate is `no_std`; its `std` feature turns on the owned types for host code.
