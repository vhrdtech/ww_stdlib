# all_gpio_device

The GPIO expander example device (`all_gpio_api`): the server that maps `ww_gpio`'s `Bank` and `Pin` onto a
board's pins, written once. A board gives it a `Board`: `SimBoard` is a board that exists only in software, with
an LED on PA5 and a button on PC13, served over WebSocket by `examples/all_gpio` (`all_gpio_sim`).
