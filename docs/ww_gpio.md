---
level: basic
---

# ww_gpio

Part of the [IO trait family](io_family.md). `ww_gpio` is a global WireWeaver trait for controlling GPIO pins remotely: reading and setting output levels,
reading input levels, configuring pin mode / pull / drive strength, and
observing edge events as a stream. It is
`no_std` by default and depends only on `wire_weaver` and `ww_si` (for `IoLevel`, the IO level in mV, and the
counted indices of the `Custom` variants below).

Add it to a device API crate with:

```toml
[dependencies]
ww_gpio = "0.1"
```

## Traits

GPIO pins are grouped into banks (ports), mirroring how most MCUs expose GPIO in hardware. Both levels are traits,
so they compose with the rest of the API using [`ww_impl!`](../api/traits.md) and can be nested as
[arrays](../api/arrays.md).

### `Bank`

One bank (port) of pins sharing the same IO level:

```rust
#[ww_trait]
trait Bank {
    ww_impl!(pin[]: Pin);

    #[effect(read)]
    fn capabilities() -> BankCapabilities<'i>;

    #[effect(actuate)]
    fn configure(config: BankConfig) -> Result<BankConfig, Error>;

    #[effect(read)]
    fn name() -> &'i str;
}
```

* `pin[]` - array of individual pins in the bank, indexed 0-based; see [`Pin`](#pin) below. Indexing can either
  follow the natural range of a bank (e.g. an IO expander) or a list of specific indices (e.g. exposing only `PB1`
  and `PB5` of an MCU port).
* `capabilities()` - static description of what the bank supports: which IO levels, modes and custom
  mode/pull/speed strings, see `BankCapabilities` under [Types](#types) below.
* `configure` - what the pins of the bank share, in a `BankConfig`: the IO level, adjustable if the hardware has
  level translators (`io_level: None` leaves it as it is). Answers with what was actually set, so `configure` with
  `None` reads the level in effect. Same shape as the `configure` of `ww_uart`, `ww_i2c`, `ww_spi`.
* `name()` - user-friendly bank name (e.g. `"PA"`).

### `Pin`

One pin of a bank. Commonly used operations (`set_output_level`, `toggle`, `output_level`, `input_level`) are
defined first, to get more compact resource paths since low path indices take fewer bits on the wire (see
[traits](../api/traits.md)):

```rust
#[ww_trait]
pub trait Pin {
    #[effect(actuate)]
    fn set_output_level(level: Level);
    #[effect(actuate)]
    fn toggle();
    #[effect(read)]
    fn output_level() -> Level;
    #[effect(read)]
    fn input_level() -> Level;

    #[stream(IoPinEvent, counted)]
    fn event(enabled: IoPinEnabledEvents<'i>) -> Result<(), Error>;

    #[effect(actuate)]
    fn set_mode(mode: Mode, initial: Option<Level>) -> Result<(), Error>;
    #[effect(read)]
    fn mode() -> Mode;

    #[effect(actuate)]
    property!(rw pull: Pull, Error);
    #[effect(edit)]
    property!(rw speed: Speed, Error);

    #[effect(read)]
    fn capabilities() -> PinCapabilities<'i>;
}
```

* `set_output_level` / `toggle` / `output_level` - control and read back the pin's output register. If the pin is
  currently an input, these still act on the output register without changing pin mode.
* `input_level` - reads the input register. If the pin is configured as output and its input buffer is disabled,
  the current output level should be returned instead.
* `event` - a counted [stream function](../api/streams.md#stream-functions) of `IoPinEvent`s (rising/falling edge,
  custom sources): the host opens it with the sources to report (`CommonError::Unsupported` for one the pin can't
  generate) and closes it to stop. Its unit is an event: the event head carries the device time of each
  edge and exactly how many edges the device dropped ([stream delivery](../api/stream_reliability.md)).
* `set_mode` / `mode` - switch between push-pull, open-drain, input, high-z, analog or a custom mode, optionally
  setting the initial output level before the switch. Can fail with `CommonError::Unsupported`, or `DifferentModes` on a
  bank whose pins share one mode.
* `pull` - pull resistor configuration, can fail with `CommonError::Unsupported`.
* `speed` - drive strength/slew rate, can fail with `CommonError::Unsupported`.
* `capabilities()` - what this pin supports: modes, pulls, speeds, whether `event` works, and its name, see
  `PinCapabilities` under [Types](#types) below.

### Several pins

`ww_gpio` stays simple and low speed: every call acts on one pin. A host that reads or sets many pins in one
round trip sends the per-pin calls as one [multi request](../api/multi_requests.md) (`MultiCall`, `MultiRead`, `MultiWrite`).
The device still runs them one after another, so pins are not sampled or changed at one instant. A bit-banged
parallel bus, a strobe with its data lines, or anything else that needs several pins at the same instant or with
precise timing is a [signal generator](ww_signal_generator.md) (driving) or a [logic analyzer](ww_logic_analyzer.md)
(sampling).

### Observing a pin level (planned)

A `property!(ro+observe level: Level)` on `Pin`, at one of its reserved indices 4..=6, would let a host watch slow
inputs without opening the `event` stream of every pin. `Pin` sits in the `Bank::pin[]` array, where
[observed properties](../api/observing.md#levels-under-resource-arrays) are marked changed through the server; a pin's
change is seen in an interrupt, so it waits for marks of an element from another task (SRV-59, STD-31).

## Types

* `Level` - `Low` / `High`, `sized`, 1 bit on the wire.
* `Mode` - `PushPullOutput`, `OpenDrainOutput`, `Input`, `HighZ` (like `Input` but may disable the input buffer to
  save power), `Analog`, `Custom(Index<CustomMode, u8>)`; `sized`, 1 nibble.
* `Pull` - `None`, `Up`, `Down`, `Custom(Index<CustomPull, u8>)`; `sized`, 2 bits.
* `Speed` - `Slow`, `Medium`, `Fast`, `VeryFast`, `Custom(Index<CustomSpeed, u8>)`; `sized`, 1 nibble.
* `IoPinEvent` - `RisingEdge`, `FallingEdge`, `Custom(Index<CustomEvent, u8>)`; `sized`, 2 bits.
* `GpioError` - the [common errors](../api/errors.md#common-errors) beside its own `DifferentModes`; unsized
  (`unib32` discriminant), evolvable, `Display` from the doc lines. Each method lists the variants it returns
  (`#[errors]`).
* `IoPinEnabledEvents<'i>` - which sources the `event` stream reports, its argument: `rising: bool`, `falling: bool`,
  and a `custom: RefVec<'i, Index<CustomEvent, u8>>` list of custom source indices.
* `PinCapabilities<'i>` - what one pin supports: `modes: MaxLen<RefVec<'i, Mode>, 8>`,
  `pulls: MaxLen<RefVec<'i, Pull>, 4>`, `speeds: MaxLen<RefVec<'i, Speed>, 5>`, `events: bool` (the `event` stream
  works on this pin), `name: MaxLen<&'i str, 16>` (e.g. `"IO4"`, `"PB5"`).
* `BankConfig` - `io_level: Option<IoLevel>`; the argument and the answer of `Bank::configure`. `IoLevel` is
  `Fixed<Volt, Milli, u16>`, the bank's IO level in mV.
* `BankCapabilities<'i>` - what a bank supports: `io_levels: MaxLen<RefVec<'i, IoLevel>, MAX_IO_LEVELS>`,
  `io_level_min` / `io_level_max: Option<IoLevel>` (an adjustable supply), `push_pull` / `open_drain` / `input` /
  `individually_configurable_pins: bool`, and `custom_mode` / `custom_pull` / `custom_speed: RefVec<'i, &'i str>`
  describing any `Custom` variant, indexed by `CustomMode` / `CustomPull` / `CustomSpeed`.

`CustomMode`, `CustomPull`, `CustomSpeed` and `CustomEvent` are counted markers of this crate (`ww_si::unit!`),
so an `Index<CustomMode, u8>` and a plain `u8` are the same byte, only the model says what it counts.

`Level` and `Mode` also carry small helpers: `Level::is_high()` / `is_low()`, `Mode::is_driving()` (true for
`PushPullOutput` and `OpenDrainOutput`).

All enums and structs are behind `#[derive_shrink_wrap]` with `owned(feature = "std")` and
`cfg_attr_borrowed(feature = "defmt", derive(defmt::Format))`, so owned (`*Owned`) variants and `defmt::Format` are
available by enabling the crate's `std` / `defmt` features.

## Using it in a device

[A GPIO expander](gpio_expander.md) puts `Bank` into an API as `port[]: ww_gpio::Bank`, serves it from a simulated
board and from the Nucleo H743ZI2, and calls it from Rust and Python. Pins are addressed by a `[bank, pin]` pair, as
`port(0).pin(5)`, and each handler gets that pair as its `index` argument.

Many calls go faster as concurrent requests: the client packs independent requests into as few USB packets as it
can. Reading the mode of 16 pins one call after another takes 940 ms on USB Full Speed, and 55 ms with
`futures::future::join_all` over the same calls (361 ms and 38 ms on High Speed). The examples `mode.rs`,
`mode_parallel.rs` and `mode_mega_parallel.rs` of `examples/all_gpio` show the three ways.

## embedded-hal (`ww_gpio_hl`)

`ww_gpio_hl` wraps the generated client in pin types modeled after embassy's: `Flex` (any mode), `PushPullOutput`,
`Input`, `OpenDrainOutput` and `Bank`, in a blocking (`blocking::FlexBlocking`, ...), an async
(`asynchronous::Flex`, ...) and a promise flavor (immediate-mode GUIs). They carry the embedded-hal 1.0 traits, so
a stock driver gets its reset, chip-select or interrupt line from a remote device:

* blocking pins: `embedded_hal::digital::{OutputPin, StatefulOutputPin, InputPin}`. Each call is a round trip,
  `toggle` one call (`Pin::toggle`), not a read and a write. They block, so not inside an async task.
* async pins (`Flex`, `Input`, `OpenDrainOutput`): `embedded_hal_async::digital::Wait`.

```rust
let irq = Flex::new_ignore_mode(device.port(0).pin(4).attachment())?
    .urgent()
    .into_input(Pull::Up)
    .await?;
let mut fusb = Fusb302::new(i2c, irq);
```

A wait doesn't miss an edge that comes after the call, nor count one from before it: it opens the pin's `event`
stream with the edges it needs first, then (`wait_for_high` / `wait_for_low`) reads the input level, then awaits the
edge. The stream is closed when the wait returns or is dropped. Setting up is one round trip, two for a level; a pin
already at the level returns after these. `last_edge_time()` gives the device time of the edge that ended the last
wait (when the device sends one), to measure from the moment the line moved rather than from when the host heard of
it. A gap in the stream (the device dropped events) ends an edge wait, since the edge may be among the lost ones,
and makes a level wait read the level again. `urgent()` / `set_priority()` set the priority of the pin's requests,
the stream open included.

`tests/io_gpio` runs them through generic functions, the way a driver uses them: `eh_async_wait` (a level that is
already there, a falling edge with its device time, an earlier edge that doesn't count, a dropped wait closing the
stream) and `eh_blocking_digital`.
