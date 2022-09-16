//! # LCD1602
//! A simple embedded-hal driver for a 1602 LCD screens.

#![no_std]

use stm32f7xx_hal::timer::SysDelay;

mod lcd1602;
pub mod custom_characters;

pub struct LCD1602<EN, RS, D4, D5, D6, D7> {
    en: EN,
    rs: RS,
    d4: D4,
    d5: D5,
    d6: D6,
    d7: D7,
    delay_handler: SysDelay,
}

pub trait DelayMs<UintType> {
    fn delay_ms(&mut self, ms: UintType);
}

pub enum TextDirection {
    LeftToRight,
    RightToLeft,
}

#[derive(Debug)]
pub enum Error<GPIO> {
    GPIOError(GPIO),
    InvalidCursorPosition,
    InvalidCGRAMLocation,
}

/// Implement 'From' for the custom Error type defined above.
impl<E> From<E> for Error<E> {
    fn from(gpio_err: E) -> Self {
        Self::GPIOError(gpio_err)
    }
}
