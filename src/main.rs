#![no_std] // just use core Crate
#![no_main] // manually define the function entry
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::format;
use alloc_cortex_m::CortexMHeap;
use core::cell::RefCell;
use core::ops::DerefMut;
use core::panic::PanicInfo;
use cortex_m::interrupt::{free, Mutex};
use cortex_m_rt::entry;
use rotary_encoder_embedded::{standard::StandardMode, Direction, RotaryEncoder};
use rtt_target::{rprintln, rtt_init_print};
use stm32f7xx_hal::gpio::{Edge, ExtiPin, Input, Pin, PullUp};
use stm32f7xx_hal::{interrupt, pac, prelude::*};

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

// Interrupt-safe Encoder handler
static ROTARY_ENCODER: Mutex<
    RefCell<
        Option<RotaryEncoder<StandardMode, Pin<'B', 1, Input<PullUp>>, Pin<'B', 5, Input<PullUp>>>>,
    >,
> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    // Initialize the allocator BEFORE you use it
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { ALLOCATOR.init(HEAP.as_ptr() as usize, HEAP_SIZE) }
    }

    // Initialize serial console
    rtt_init_print!();

    let core_perip = cortex_m::peripheral::Peripherals::take().unwrap();
    let dev_perip = pac::Peripherals::take().unwrap();
    let gpio_b = dev_perip.GPIOB.split();

    let rcc = dev_perip.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(216.MHz()).freeze(); // lock configurations
    let d = core_perip.SYST.delay(&clocks);

    // interrupt-related stuff
    let mut sys_cfg = dev_perip.SYSCFG;
    let mut apb2 = rcc.apb2; // Advanced Peripheral Bus 2 (APB2) registers
    let mut exti = dev_perip.EXTI; // External Interrupt Pin interface

    // I/O setup
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
    let mut encoder_dt = gpio_b.pb1.into_pull_up_input();
    let mut encoder_clk = gpio_b.pb5.into_pull_up_input();
    // let mut encoder_pushbutton = gpio_b.pb10.into_pull_up_input();

    // Enable external interrupts on rotary encoder pins (ALTERNATIVE: pool them with a timer at ~900Hz)
    // - External interrupts: https://stackoverflow.com/questions/56179131/cannot-receive-interrupt-on-pe0-stm32
    // - EXTI register logics: https://stm32f4-discovery.net/2014/08/stm32f4-external-interrupts-tutorial/
    encoder_dt.make_interrupt_source(&mut sys_cfg, &mut apb2);
    encoder_dt.trigger_on_edge(&mut exti, Edge::RisingFalling);
    encoder_dt.enable_interrupt(&mut exti);
    unsafe { pac::NVIC::unmask(interrupt::EXTI1) } // Line1 interrupt (because the pin is PB1)
    encoder_clk.make_interrupt_source(&mut sys_cfg, &mut apb2);
    encoder_clk.trigger_on_edge(&mut exti, Edge::RisingFalling);
    encoder_clk.enable_interrupt(&mut exti);
    unsafe { pac::NVIC::unmask(interrupt::EXTI9_5) } // Line1 interrupt (because the pin is PB5)

    // LCD setup
    let mut lcd = LCD1602::new(en, rs, d4, d5, d6, d7, d).unwrap();
    // lcd.set_display(true, true, false).unwrap();
    lcd.init_custom_chars().unwrap();
    let mut lcd_scaler = 0u8;

    // Encoder setup
    free(|cs| {
        ROTARY_ENCODER.borrow(cs).replace(Some(
            RotaryEncoder::new(encoder_dt, encoder_clk).into_standard_mode(),
        ));
    });

    let mut encoder_val = 0i16;

    rprintln!("Ready!");
    led_1.toggle();

    lcd.set_cursor(0, 0).unwrap();
    lcd.print("Encoder: ").unwrap();

    loop {
        // Character animation
        lcd_scaler += 1;
        if lcd_scaler % 50 == 0 {
            if led_2.is_set_high() {
                led_2.set_low();
                lcd.set_cursor(15, 1).unwrap();
                lcd.write_custom_char(MAN_STANDING).unwrap();
                // lcd.write_custom_char(HEART_BORDER).unwrap();
            } else {
                led_2.set_high();
                lcd.set_cursor(15, 1).unwrap();
                lcd.write_custom_char(MAN_DANCING).unwrap();
                // lcd.write_custom_char(HEART_FULL).unwrap();
            }

            lcd_scaler = 0;
        }

        // Update timer printed value
        lcd.set_cursor(9, 0).unwrap();
        lcd.print(&format!("{: <3}", encoder_val)).unwrap(); // left-aligned with 3 digits (including sign)

        lcd.delay_ms(1u8); // loop at around 1kHz
    }
}

enum InterruptedPin {
    DtPin,
    ClkPin,
}

fn handle_encoder_interrupt(interrupted_pin: InterruptedPin) {
    // Retrieve Rotary Encoder from safely stored static global
    free(|cs| {
        if let Some(ref mut rotary_encoder) = ROTARY_ENCODER.borrow(&cs).borrow_mut().deref_mut() {
            // Borrow the pins to clear the pending interrupt bit
            let (dt, clk) = rotary_encoder.pins_mut();
            match interrupted_pin {
                InterruptedPin::DtPin => {
                    dt.clear_interrupt_pending_bit();
                }
                InterruptedPin::ClkPin => {
                    clk.clear_interrupt_pending_bit();
                }
            }

            // Update the encoder, which will compute its direction
            rotary_encoder.update();
            match rotary_encoder.direction() {
                Direction::Clockwise => {
                    rprintln!("Increment!");
                    // Increment some value
                }
                Direction::Anticlockwise => {
                    rprintln!("Decrement!");
                    // Decrement some value
                }
                Direction::None => {
                    // Do nothing
                }
            }
        }
    });
}

#[interrupt]
fn EXTI1() {
    rprintln!("EXTI1 interrupt!");
    handle_encoder_interrupt(InterruptedPin::DtPin);
}

#[interrupt]
fn EXTI9_5() {
    rprintln!("EXTI9_5 interrupt!");
    handle_encoder_interrupt(InterruptedPin::ClkPin);
}
