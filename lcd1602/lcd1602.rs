use embedded_hal::digital::v2::OutputPin;
use embedded_hal::prelude::_embedded_hal_blocking_delay_DelayMs;
use stm32f7xx_hal::timer::SysDelay;

use crate::{LCD1602, DelayMs, Error};
use crate::lcd1602::PackType::{Command, Data};

pub enum Direction {
    LeftToRight,
    RightToLeft,
}

enum PackType {
    Command,
    Data,
}

impl<EN, RS, D4, D5, D6, D7, E> LCD1602<EN, RS, D4, D5, D6, D7>
    where
        EN: OutputPin<Error=E>, RS: OutputPin<Error=E>,
        D4: OutputPin<Error=E>, D5: OutputPin<Error=E>,
        D6: OutputPin<Error=E>, D7: OutputPin<Error=E> {
    /// Create and initialise a new LCD1602 interface.
    pub fn new(en: EN, rs: RS, d4: D4, d5: D5, d6: D6, d7: D7, delay_handler: SysDelay)
               -> Result<LCD1602<EN, RS, D4, D5, D6, D7>, Error<E>> {
        let mut lcd = LCD1602 { en, rs, d4, d5, d6, d7, delay_handler };
        lcd.init()?;
        Ok(lcd)
    }

    /// Initialise the LCD.
    fn init(&mut self)
            -> Result<(), Error<E>> {
        // make 3 pings to the LCD to initialise communication for 4-bit mode
        self.send(Command, 0x03)?;
        self.delay_ms(5u8);
        self.send(Command, 0x03)?;
        self.delay_ms(5u8);
        self.send(Command, 0x03)?;
        self.delay_ms(5u8);
        self.send(Command, 0x02)?; // 4-bit mode

        let mut config_cmd = 0x00; // 5x8 dots per character
        config_cmd |= 0x08; // 2 lines
        self.send(Command, 0x20 | config_cmd)?; // function set command

        self.set_display(true, false, false)?;
        self.set_entry_mode(Direction::LeftToRight, false)?;
        self.clear()?;
        Ok(())
    }

    /// Configure text direction.
    pub fn set_entry_mode(&mut self, text_direction: Direction, auto_move_cursor: bool)
                          -> Result<(), Error<E>> {
        let mut cmd = 0x04; // entry mode set command
        match text_direction {
            Direction::LeftToRight => cmd |= 0x02,
            Direction::RightToLeft => cmd |= 0x00,
        }
        if auto_move_cursor { cmd |= 0x01; }
        self.send(Command, cmd)?;
        Ok(())
    }

    /// Configure display status, cursor and its blinking.
    pub fn set_display(&mut self, on: bool, show_cursor: bool, blink_cursor: bool)
                       -> Result<(), Error<E>> {
        let mut cmd = 0x08; // display control command
        if on { cmd |= 0x04; }
        if show_cursor { cmd |= 0x02; }
        if blink_cursor { cmd |= 0x01; }
        self.send(Command, cmd)?;
        Ok(())
    }

    /// Clear screen and set cursor to start.
    pub fn clear(&mut self)
                 -> Result<(), Error<E>> {
        self.send(Command, 0x01)?;
        self.delay_ms(2u8); // slowest displays need at least 1.53ms
        Ok(())
    }

    /// Just move cursor at starting position, without any erase.
    pub fn home(&mut self)
                -> Result<(), Error<E>> {
        self.send(Command, 0x02)?;
        self.delay_ms(2u8); // slowest displays need at least 1.53ms
        Ok(())
    }

    /// Write a given string. TODO: check if the string fits the LCD!
    pub fn print(&mut self, s: &str)
                 -> Result<(), Error<E>> {
        for ch in s.chars() {
            self.send(Data, ch as u8)?;
        }
        Ok(())
    }

    /// Send desired 8bits, either as command or data, as two 4bits packets through the bus.
    fn send(&mut self, comm_type: PackType, payload: u8)
            -> Result<(), Error<E>> {
        match comm_type {
            Command => self.rs.set_low()?, // write in instruction register
            Data => self.rs.set_high()?, // write in data register
        }
        self.write_bus(payload >> 4)?;
        self.write_bus(payload)?;
        Ok(())
    }

    /// Write 4bits data in D4-D7 pins.
    fn write_bus(&mut self, data: u8)
                 -> Result<(), Error<E>> {
        match (data & 0x1) > 0 {
            true => self.d4.set_high()?,
            false => self.d4.set_low()?,
        };
        match (data & 0x2) > 0 {
            true => self.d5.set_high()?,
            false => self.d5.set_low()?,
        };
        match (data & 0x4) > 0 {
            true => self.d6.set_high()?,
            false => self.d6.set_low()?,
        };
        match (data & 0x8) > 0 {
            true => self.d7.set_high()?,
            false => self.d7.set_low()?,
        };

        self.en.set_high()?;
        self.delay_ms(1u8); // enable pulse must be > 450ns
        self.en.set_low()?;
        self.delay_ms(1u8); // commands need > 37us to settle
        Ok(())
    }
}

impl<EN, RS, D4, D5, D6, D7> DelayMs<u8> for LCD1602<EN, RS, D4, D5, D6, D7> {
    fn delay_ms(&mut self, ms: u8) -> () {
        self.delay_handler.delay_ms(ms);
    }
}

impl<EN, RS, D4, D5, D6, D7> DelayMs<u16> for LCD1602<EN, RS, D4, D5, D6, D7> {
    fn delay_ms(&mut self, ms: u16) -> () {
        self.delay_handler.delay_ms(ms);
    }
}
