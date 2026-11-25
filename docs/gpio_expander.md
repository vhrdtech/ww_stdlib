---
level: basic
---

# A GPIO expander

A GPIO expander lets a computer read and drive the pins of a board. This page builds one from the standard trait
`ww_gpio::Bank`: it serves the trait from a board simulated on your PC, then blinks an LED and reads a button from
Rust and from Python. The last section runs the same host program against a real board over USB.

The example has three crates. [`examples/all_gpio_api/src/lib.rs`] holds the API, `all_gpio_device` the device code,
and `all_gpio` the host program and the simulated device. Start the simulated device in a terminal of its own, and
leave it running while you read:

```sh
cargo run -p all_gpio --example all_gpio_sim
```

It prints `all_gpio at ws://127.0.0.1:8081/ww`, the address the host programs below connect to.

## The board behind the trait

`ww_gpio` says what a host may ask of a pin, but not how a board answers. The device code asks a `Board` for that:
its banks, and the registers of each pin. A firmware implements `Board` over the chip's GPIO registers, and the
simulated device over an array in memory:

```rust
--8<-- "examples/all_gpio_device/src/lib.rs:board"
```

The simulated board is modelled on a small development board:

```rust
--8<-- "examples/all_gpio_device/src/sim.rs:sim"
```

Its button follows the clock, not a person, so every run of the tests sees the same presses.

## Serving Bank and Pin

The device answers each item of `ww_gpio::Bank` and `ww_gpio::Pin` with a [[handler]], as it would for a trait of
its own. A handler's name is the item's path in the API: `port_name` for `name()` of a bank, and
`port_pin_set_output_level` for `set_output_level()` of a pin. Its `index` argument holds the array indices of that
path, the bank first and then the pin:

```rust
--8<-- "examples/all_gpio_device/src/lib.rs:levels"
```

A handler returns `Ready(value)` when it has the answer at once, as these do. `set_output_level` changes the output
register, and `input_level` reads the level on the pin at the time the request arrived.

Some calls can fail. `set_mode` returns a `Result` whose error is `GpioError`, and the board refuses a mode it can't
do with a [common error](../api/errors.md#common-errors), `CommonError::Unsupported`. `.map_err(Into::into)` turns it
into the error the trait declares:

```rust
--8<-- "examples/all_gpio_device/src/lib.rs:mode"
```

The handlers live on `AllGpio`, the device's state. `ww_codegen!` generates the server that decodes each request and
calls them:

```rust
--8<-- "examples/all_gpio_device/src/lib.rs:codegen"
```

The options are the ones of [Methods](../api/methods.md) and [Properties](../basic/properties.md). Two more functions
tell hosts which indices exist: `valid_indices_root_port` gives the banks and `valid_indices_root_port_pin` the pins
of one bank. The two traits have 13 items and the device 17 handlers, because a property has one handler to read
and one to write, and a stream one to open and one to close. All of them are in
[`examples/all_gpio_device/src/lib.rs`].

## From Rust

The host program [`examples/all_gpio/examples/blink.rs`] uses the [[client]] generated from `all_gpio_api`. It
first lists the banks, through the indices the device reports:

```rust
--8<-- "examples/all_gpio/examples/blink.rs:banks"
```

`port(port)` and `pin(number)` pick an element of an array, as `port(0).pin(5)` picks pin 5 of the first bank.
The program then makes PA5 an output and toggles it three times. The LED of the simulated board lights each time
PA5 drives high:

```rust
--8<-- "examples/all_gpio/examples/blink.rs:blink"
```

Last, it makes PC13 an input without a pull resistor and reads it every half second. The board's own pull-up holds
the pin high, and the button pulls it low while pressed:

```rust
--8<-- "examples/all_gpio/examples/blink.rs:button"
```

Run the program in a second terminal, while the simulated device runs in the first:

```sh
cargo run -p all_gpio --example all_gpio_blink
```

It prints the three banks, `PA5: High`, `PA5: Low` and `PA5: High`, then six levels of PC13. A `Low` among them is a
press: the button is pressed for the first second of every three.

## From Python

The `wire_weaver` Python module builds the API at run time from the device's introspection data, as in
[Properties](../basic/properties.md#from-python). An array is a Python list, so pin 5 of the first bank is
`dev.port[0].pin[5]`:

```python
--8<-- "examples/stdlib_py/gpio.py:connect"
```

A method is a Python method, with its arguments in order. An enum variant without fields is its name as a string:

```python
--8<-- "examples/stdlib_py/gpio.py:led"
```

A refused call raises `RemoteError`. A [common error](../api/errors.md#common-errors) is in its `common` attribute:

```python
--8<-- "examples/stdlib_py/gpio.py:refused"
```

The whole script is [`examples/stdlib_py/gpio.py`]. With the simulated device running, it prints:

```text
--8<-- "docs/generated/all_gpio/gpio_py.txt"
```

The mode stays `PushPullOutput` after the refused call: the board never left it.

## On a real board

The board examples of `examples_mcu/` serve the same API over USB. The Nucleo H743ZI2's firmware implements every
handler over the chip's GPIO registers, with all eleven banks from PA to PK. Flash it with its runner, which
`.cargo/config.toml` of the board example names:

```sh
cd examples_mcu/nucleo_h743zi2
cargo run --release --features streams --bin nucleo-h743zi2-all-gpio
```

Then run the same host program over USB. The green LED of this board is on PB0:

```sh
cargo run -p all_gpio --example all_gpio_blink -- --usb --led PB0
```

The firmware has a server of its own over the chip's registers, not `all_gpio_device`. The API is the same, so
every host program and script of this page works with both.

## Where to go next

- [A UART bridge](uart_bridge.md) serves a standard trait with a stream and a sink: bytes from a serial device, and
  bytes to it.
- [`ww_gpio`](ww_gpio.md) describes every item of `Bank` and `Pin`, the edge events of a pin, and the embedded-hal
  pins of `ww_gpio_hl`.
