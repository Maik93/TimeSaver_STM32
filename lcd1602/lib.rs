//! # LCD1602
//! A simple embedded-hal driver for a 1602 LCD screens.

#![no_std]

use stm32f7xx_hal::timer::SysDelay;

mod error;
mod lcd1602;


pub struct LCD1602<EN, RS, D4, D5, D6, D7> {
    en: EN,
    rs: RS,
    d4: D4,
    d5: D5,
    d6: D6,
    d7: D7,
    delay_handler: SysDelay,
}
