#![no_std] // just use core Crate
#![no_main] // manually define the function entry
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::format;
use alloc_cortex_m::CortexMHeap;
use core::panic::PanicInfo;
use cortex_m_rt::entry;
use rotary_encoder_embedded::{RotaryEncoder, Direction, Sensitivity};
use rtt_target::{rtt_init_print, rprintln};

use stm32f7xx_hal::{pac, prelude::*};

use lcd1602::{LCD1602, DelayMs};
use lcd1602::custom_characters::{MAN_STANDING, MAN_DANCING, HEART_BORDER, HEART_FULL};

#[alloc_error_handler]
fn oom(_: core::alloc::Layout) -> ! {
    rprintln!("Allocation error");
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    rprintln!("{}", _info);
    loop {}
}

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[entry]
fn main() -> ! {
    // Initialize the allocator BEFORE you use it
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { ALLOCATOR.init(HEAP.as_ptr() as usize, HEAP_SIZE) }
    }

    let core_perip = cortex_m::peripheral::Peripherals::take().unwrap();
    let dev_perip = pac::Peripherals::take().unwrap();

    let rcc = dev_perip.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(216.MHz()).freeze(); // lock configurations
    let d = core_perip.SYST.delay(&clocks);

    let gpio_b = dev_perip.GPIOB.split();
    let mut led_1 = gpio_b.pb0.into_push_pull_output();
    let mut led_2 = gpio_b.pb7.into_push_pull_output();
    let mut led_3 = gpio_b.pb14.into_push_pull_output();

    // LCD pins
    let rs = gpio_b.pb4.into_push_pull_output();
    let en = gpio_b.pb3.into_push_pull_output();
    let d4 = gpio_b.pb12.into_push_pull_output();
    let d5 = gpio_b.pb13.into_push_pull_output();
    let d6 = gpio_b.pb15.into_push_pull_output();
    let d7 = gpio_b.pb8.into_push_pull_output();

    // Encoder pins
    let encoder_dt = gpio_b.pb1.into_pull_up_input(); // TODO: set the right pin number
    let encoder_clk = gpio_b.pb2.into_pull_up_input(); // TODO: set the right pin number

    rtt_init_print!();

    // LCD setup
    let mut lcd = LCD1602::new(en, rs, d4, d5, d6, d7, d).unwrap();
    // lcd.set_display(true, true, false).unwrap();
    lcd.init_custom_chars().unwrap();

    // Encoder setup
    let mut rotary_encoder = RotaryEncoder::new(encoder_dt, encoder_clk);
    rotary_encoder.set_sensitivity(Sensitivity::Low);

    rprintln!("Ready to blink!");

    led_1.toggle();
    lcd.print("Ready!").unwrap();
    lcd.delay_ms(1_000u16);

    loop {
        // led_2.toggle();
        // lcd.set_cursor(6, 1).unwrap();
        // lcd.write_custom_char(MAN_STANDING).unwrap();
        // lcd.write_custom_char(HEART_BORDER).unwrap();
        lcd.delay_ms(500u16);

        // led_3.toggle();
        // // lcd.clear().ok();
        // lcd.set_cursor(6, 1).unwrap();
        // lcd.write_custom_char(MAN_DANCING).unwrap();
        // lcd.write_custom_char(HEART_FULL).unwrap();
        // lcd.delay_ms(500u16);

        // Update the encoder, which will compute its direction
        rotary_encoder.update();
        match rotary_encoder.direction() {
            Direction::Clockwise => {
                // Increment some value
            }
            Direction::Anticlockwise => {
                // Decrement some value
            }
            Direction::None => {
                // Do nothing
            }
        }
    }
}
