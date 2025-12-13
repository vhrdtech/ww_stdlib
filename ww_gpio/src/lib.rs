use wire_weaver::prelude::*;

/// A bank of related IO pins.
/// Each pin in a bank is using the same reference voltage, that can be adjusted if bank supports it.
#[ww_trait]
trait GpioBank {
    /// 0. Array of individual pins.
    ww_impl!(pins[]: Gpio);

    // 1-7. Reserved
    reserved!();
    reserved!();
    reserved!();
    reserved!();
    reserved!();
    reserved!();
    reserved!();

    /// Range or list of available pins, each pin is identified by an u32 index.
    fn available() -> AvailablePins<'i>;
    /// Capabilities that each pin of the bank supports.
    fn capabilities() -> GpioBankCapabilities<'i>;

    /// Reference voltage currently in use.
    fn reference_voltage() -> f32;
    /// Set reference voltage to the requested value.
    fn set_reference_voltage(value: f32) -> Result<(), GpioError>;

    /// User-friendly bank name.
    fn name() -> &'i str;
}

/// One pin from a GPIO bank.
///
/// Commonly used operations are defined first, to get more compact resource paths.
#[ww_trait]
pub trait Gpio {
    /// 0. Set the output as high.
    fn set_high();
    /// 1. Set the output as low.
    fn set_low();
    /// 2. Toggle the output level.
    fn toggle();
    /// 3. Returns true, if output was previously set to high.
    fn is_set_high() -> bool;
    /// 4. Returns true, if output was previously set to low.
    fn is_set_low() -> bool;

    /// 5. Returns true if pin input level is high.
    fn is_high() -> bool;
    /// 6. Returns true if pin input level is low.
    fn is_low() -> bool;

    /// 7. Asynchronous stream of events (rising / falling edge), if enabled by [configure_events]
    stream!(event: IoPinEvent);

    /// Set the output level.
    ///
    /// If the pin is currently configured as input, this level should only be written to control register, without changing pin mode.
    fn set_output_level(level: Level);

    /// Get the input level.
    ///
    /// Note that when a pin is configured as output, input buffer might be disabled, resulting in incorrect input level reported.
    /// If this is the case, current output level must be returned.
    fn get_input_level() -> Level;

    /// Mode configuration, input, push-pull, open-drain or custom.
    property!(mode: GpioMode);
    /// Pull resistors selection.
    property!(pull: Pull);
    /// Drive strength selection.
    property!(speed: Speed);

    /// Enable or disable asynchronous events sent through the stream.
    fn configure_events(enabled: IoPinEnabledEvents) -> Result<(), GpioError>;

    // fn pulse() -> Result<(), GpioError>;
    // fn set_duty(duty: ?) -> Result<(), GpioError>;
    // fn set_frequency(frequency: ?) -> Result<(), GpioError>;
    // fn set_pwm(frequency, duty)?;
}

pub enum Level {
    High,
    Low
}

pub enum GpioMode {
    PushPullOutput,
    OpenDrainOutput,
    Input,
    Custom(u8),
}

pub enum GpioError {
    UnsupportedMode,
    UnsupportedPull,
    UnsupportedSpeed,
    UnsupportedEventType,
    UnsupportedReferenceVoltage,
}

pub enum Pull {
    None,
    Up,
    Down,
    Custom(u8),
}

pub enum Speed {
    Slow,
    Medium,
    Fast,
    VeryFast,
    Custom(u8),
}

pub enum IoPinEvent {
    RisingEdge,
    FallingEdge,
}

pub struct IoPinEnabledEvents {
    pub rising: bool,
    pub falling: bool,
}

pub enum AvailablePins<'i> {
    Range(u32, u32),
    List(RefVec<'i, u32>)
}

pub struct GpioBankCapabilities<'i> {
    // adjustable_voltage: bool,
    pub supported_voltage: RefVec<'i, f32>,

    pub push_pull_capable: bool,
    pub open_drain_capable: bool,
    pub input_capable: bool,

    pub custom_mode: RefVec<'i, &'i str>,
    pub custom_pull: RefVec<'i, &'i str>,
    pub custom_speed: RefVec<'i, &'i str>,
}

// PWMOutput(PWMConfig?),
// PWMInput(PWMConfig?),
// PulseOutput,
// PulseInput,
// StepperOutput,
// UartTx,
// UartRx,
// UartRts,
// UartCts,
// I2cSda,
// I2cScl,
// SpiMosi,
// SpiMiso,
// SpiSck,
// SpiCs,
// OneWire,

// AnalogOutput,
// AnalogInput,
// analog modes? encoder?
