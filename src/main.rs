#![no_std] // just use core Crate
#![no_main] // manually define the function entry

use stm32f7xx_hal as hal;

use cortex_m_rt::entry;
use core::panic::PanicInfo;
use crate::hal::{pac, prelude::*};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { loop {} }

#[entry]
fn main() -> ! {
    let dev_perip = pac::Peripherals::take().unwrap();

    let gpio_b = dev_perip.GPIOB.split();
    let mut led_1 = gpio_b.pb0.into_push_pull_output();
    let mut led_2 = gpio_b.pb7.into_push_pull_output();
    let mut led_3 = gpio_b.pb14.into_push_pull_output();

    loop {
        for _ in 0..10_000 {
            led_1.set_high();
            led_2.set_high();
            led_3.set_high();
        }

        for _ in 0..10_000 {
            led_1.set_low();
            led_2.set_low();
            led_3.set_low();
        }
    }
}
