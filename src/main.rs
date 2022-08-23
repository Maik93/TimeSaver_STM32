#![no_std] // just use core Crate
#![no_main] // manually define the function entry

use core::panic::PanicInfo;
use cortex_m_rt::entry;
// use core::panic::PanicInfo;
use rtt_target::{rtt_init_print, rprintln};

use stm32f7xx_hal::{pac, prelude::*};

use lcd1602::{LCD1602, DelayMs};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    rprintln!("{}", _info);
    loop {}
}

#[entry]
fn main() -> ! {
    let core_perip = cortex_m::peripheral::Peripherals::take().unwrap();
    let dev_perip = pac::Peripherals::take().unwrap();

    let rcc = dev_perip.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(216.MHz()).freeze(); // lock configurations
    let d = core_perip.SYST.delay(&clocks);

    let gpio_b = dev_perip.GPIOB.split();
    let mut led_1 = gpio_b.pb0.into_push_pull_output();
    let mut led_2 = gpio_b.pb7.into_push_pull_output();
    let mut led_3 = gpio_b.pb14.into_push_pull_output();

    let rs = gpio_b.pb4.into_push_pull_output();
    let en = gpio_b.pb3.into_push_pull_output();
    let d4 = gpio_b.pb12.into_push_pull_output();
    let d5 = gpio_b.pb13.into_push_pull_output();
    let d6 = gpio_b.pb15.into_push_pull_output();
    let d7 = gpio_b.pb8.into_push_pull_output();

    let mut lcd = LCD1602::new(en, rs, d4, d5, d6, d7, d).unwrap();

    rtt_init_print!();

    rprintln!("Ready to blink!");

    loop {
        led_1.toggle();
        lcd.print("Ready!").ok();
        // d.delay_ms(500u16);
        lcd.delay_ms(1_000u16);

        led_2.toggle();
        lcd.print("Go!").ok();
        // d.delay_ms(1_000u16);
        lcd.delay_ms(2_000u16);

        led_3.toggle();
        lcd.clear().ok();
        // d.delay_ms(500u16);
        lcd.delay_ms(500u16);
    }
}
