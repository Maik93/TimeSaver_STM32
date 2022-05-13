#![no_std] // just use core Crate
#![no_main] // manually define the function entry

use cortex_m_rt::entry;
use core::panic::PanicInfo;

use stm32f7xx_hal::{pac, prelude::*};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { loop {} }

#[entry]
fn main() -> ! {
    let core_perip = cortex_m::peripheral::Peripherals::take().unwrap();
    let dev_perip = pac::Peripherals::take().unwrap();

    let rcc = dev_perip.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(216.MHz()).freeze(); // lock configurations
    let mut d = core_perip.SYST.delay(&clocks);

    let gpio_b = dev_perip.GPIOB.split();
    let mut led_1 = gpio_b.pb0.into_push_pull_output();
    let mut led_2 = gpio_b.pb7.into_push_pull_output();
    let mut led_3 = gpio_b.pb14.into_push_pull_output();

    loop {
        led_1.set_high();
        led_2.set_high();
        led_3.set_high();

        d.delay_us(1000_000_u32);

        led_1.set_low();
        led_2.set_low();
        led_3.set_low();

        d.delay_us(1000_000_u32);
    }
}
