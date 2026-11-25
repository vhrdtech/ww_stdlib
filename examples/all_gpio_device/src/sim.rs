//! A board that exists only in software, for the simulated device and the tests.

use wire_weaver::prelude::CommonError;
use ww_device::Instant;
use ww_gpio::{Level, Mode, Pull};

use crate::{Board, PinId};

/// The banks of the simulated board, 16 pins each.
const BANKS: [&str; 3] = ["PA", "PB", "PC"];
/// Pins in each bank.
const PINS: u8 = 16;
/// The LED: lit while PA5 drives high.
const LED: PinId = PinId { bank: 0, pin: 5 };
/// The button: PC13, pulled up on the board, low while pressed.
const BUTTON: PinId = PinId { bank: 2, pin: 13 };
/// The button is pressed for the first second of every three.
const BUTTON_PERIOD_US: u64 = 3_000_000;
/// How long each press lasts.
const BUTTON_PRESSED_US: u64 = 1_000_000;

/// What the board knows of one pin.
#[derive(Debug, Clone, Copy)]
struct PinState {
    mode: Mode,
    pull: Pull,
    output: Level,
}

impl PinState {
    /// As after reset: an input without a pull, its output register low.
    const RESET: PinState = PinState {
        mode: Mode::Input,
        pull: Pull::None,
        output: Level::Low,
    };
}

// --8<-- [start:sim]
/// A board with three banks of 16 pins, PA, PB and PC. An LED sits on PA5 and lights while the pin drives high. A
/// button on PC13 has a pull-up on the board and pulls the pin low while it is pressed: for the first second of
/// every three, the same at every run. Every other pin is left open and reads its pull resistor, low without one.
// --8<-- [end:sim]
#[derive(Debug, Clone)]
pub struct SimBoard {
    pins: [[PinState; PINS as usize]; BANKS.len()],
}

impl Default for SimBoard {
    fn default() -> Self {
        SimBoard {
            pins: [[PinState::RESET; PINS as usize]; BANKS.len()],
        }
    }
}

impl SimBoard {
    /// Whether the LED on PA5 is lit.
    pub fn led(&self) -> bool {
        let state = self.state(LED);
        state.mode == Mode::PushPullOutput && state.output == Level::High
    }

    /// Whether the button on PC13 is pressed at `now`.
    pub fn button_pressed(now: Instant) -> bool {
        now.as_micros() % BUTTON_PERIOD_US < BUTTON_PRESSED_US
    }

    fn state(&self, pin: PinId) -> &PinState {
        &self.pins[pin.bank as usize][pin.pin as usize]
    }

    fn state_mut(&mut self, pin: PinId) -> &mut PinState {
        &mut self.pins[pin.bank as usize][pin.pin as usize]
    }
}

impl Board for SimBoard {
    fn banks(&self) -> &[&'static str] {
        &BANKS
    }

    fn pins(&self, bank: u8) -> u8 {
        if (bank as usize) < BANKS.len() {
            PINS
        } else {
            0
        }
    }

    fn set_output(&mut self, pin: PinId, level: Level) {
        self.state_mut(pin).output = level;
    }

    fn output(&self, pin: PinId) -> Level {
        self.state(pin).output
    }

    fn input(&self, pin: PinId, now: Instant) -> Level {
        let state = self.state(pin);
        match state.mode {
            Mode::PushPullOutput => return state.output,
            Mode::OpenDrainOutput if state.output == Level::Low => return Level::Low,
            _ => {}
        }
        if pin == BUTTON {
            return if Self::button_pressed(now) {
                Level::Low
            } else {
                Level::High
            };
        }
        match state.pull {
            Pull::Up => Level::High,
            _ => Level::Low,
        }
    }

    fn set_mode(&mut self, pin: PinId, mode: Mode) -> Result<(), CommonError<'static>> {
        match mode {
            Mode::PushPullOutput | Mode::OpenDrainOutput | Mode::Input => {
                self.state_mut(pin).mode = mode;
                Ok(())
            }
            _ => Err(CommonError::Unsupported),
        }
    }

    fn mode(&self, pin: PinId) -> Mode {
        self.state(pin).mode
    }

    fn set_pull(&mut self, pin: PinId, pull: Pull) -> Result<(), CommonError<'static>> {
        match pull {
            Pull::None | Pull::Up | Pull::Down => {
                self.state_mut(pin).pull = pull;
                Ok(())
            }
            Pull::Custom(_) => Err(CommonError::Unsupported),
        }
    }

    fn pull(&self, pin: PinId) -> Pull {
        self.state(pin).pull
    }
}
