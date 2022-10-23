#![no_std] // just use core Crate
#![no_main] // manually define the function entry
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::format;
use alloc_cortex_m::CortexMHeap;
use core::panic::PanicInfo;
use cortex_m_rt::entry;
use rotary_encoder_embedded::{Direction, RotaryEncoder, Sensitivity};
use rtt_target::{rprintln, rtt_init_print};

use stm32f7xx_hal::{pac, prelude::*};

use lcd1602::custom_characters::{MAN_DANCING, MAN_STANDING};
use lcd1602::{DelayMs, LCD1602};

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
    // let mut led_3 = gpio_b.pb14.into_push_pull_output();

    // LCD pins
    let rs = gpio_b.pb4.into_push_pull_output();
    let en = gpio_b.pb3.into_push_pull_output();
    let d4 = gpio_b.pb12.into_push_pull_output();
    let d5 = gpio_b.pb13.into_push_pull_output();
    let d6 = gpio_b.pb15.into_push_pull_output();
    let d7 = gpio_b.pb8.into_push_pull_output();

    // Encoder pins
    let encoder_dt = gpio_b.pb9.into_pull_up_input();
    let encoder_clk = gpio_b.pb5.into_pull_up_input();

    rtt_init_print!();

    // LCD setup
    let mut lcd = LCD1602::new(en, rs, d4, d5, d6, d7, d).unwrap();
    // lcd.set_display(true, true, false).unwrap();
    lcd.init_custom_chars().unwrap();

    // Encoder setup
    let mut rotary_encoder = RotaryEncoder::new(encoder_dt, encoder_clk);
    rotary_encoder.set_sensitivity(Sensitivity::Low);
    let mut encoder_val = 0i16;

    rprintln!("Ready!");
    led_1.toggle();

    lcd.set_cursor(0, 0).unwrap();
    lcd.print("Encoder: ").unwrap();

    loop {
        // Update the encoder, which will compute its direction
        rotary_encoder.update();
        match rotary_encoder.direction() {
            Direction::Clockwise => {
                rprintln!("Increment!");
                encoder_val += 1;
            }
            Direction::Anticlockwise => {
                rprintln!("Decrement!");
                encoder_val -= 1;
            }
            Direction::None => {
                // Do nothing
            }
        }

        // Character animation
        if led_2.is_set_high() {
            led_2.set_low();
            lcd.set_cursor(6, 1).unwrap();
            lcd.write_custom_char(MAN_STANDING).unwrap();
            // lcd.write_custom_char(HEART_BORDER).unwrap();
        } else {
            led_2.set_high();
            lcd.set_cursor(6, 1).unwrap();
            lcd.write_custom_char(MAN_DANCING).unwrap();
            // lcd.write_custom_char(HEART_FULL).unwrap();
        }

        // Update timer printed value
        lcd.set_cursor(10, 0).unwrap();
        lcd.print(&format!("{}", encoder_val)).unwrap();

        lcd.delay_ms(500u16);
    }
}
