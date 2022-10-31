#![no_std] // just use core Crate
#![no_main] // manually define the function entry
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::format;
use core::cell::RefCell;
use core::ops::DerefMut;
use core::panic::PanicInfo;
use cortex_m::interrupt::{free, Mutex};
use cortex_m_rt::entry;
use rtt_target::{rprintln, rtt_init_print};
use stm32f7xx_hal::gpio::{Edge, ExtiPin, Output, Pin};
use stm32f7xx_hal::pac::TIM2;
use stm32f7xx_hal::timer::{CounterUs, Event};
use stm32f7xx_hal::{interrupt, pac, prelude::*};

use lcd1602::custom_characters::{MAN_DANCING, MAN_STANDING};
use lcd1602::{DelayMs, LCD1602};

mod encoder_interface;
mod utilities;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    rprintln!("{}", _info);
    loop {}
}

static LED_3: Mutex<RefCell<Option<Pin<'B', 14, Output>>>> = Mutex::new(RefCell::new(None));

static TIM2_COUNTER: Mutex<RefCell<Option<CounterUs<TIM2>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    utilities::init_mem_allocator();

    // Initialize serial console
    rtt_init_print!();

    let core_perip = cortex_m::peripheral::Peripherals::take().unwrap();
    let dev_perip = pac::Peripherals::take().unwrap();
    let gpio_b = dev_perip.GPIOB.split();

    let rcc = dev_perip.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(216.MHz()).freeze(); // lock configurations
    let d = core_perip.SYST.delay(&clocks);

    // External interrupts stuff
    let mut sys_cfg = dev_perip.SYSCFG;
    let mut apb2 = rcc.apb2; // Advanced Peripheral Bus 2 (APB2) registers
    let mut exti = dev_perip.EXTI; // External Interrupt Pin interface

    // Timer interrupt stuff
    let mut tim2_counter = dev_perip.TIM2.counter_us(&clocks);
    tim2_counter.start(50.millis()).unwrap();
    tim2_counter.listen(Event::Update);
    free(|cs| TIM2_COUNTER.borrow(cs).replace(Some(tim2_counter)));
    unsafe { pac::NVIC::unmask(interrupt::TIM2) }

    // I/O setup
    let mut led_1 = gpio_b.pb0.into_push_pull_output();
    let mut led_2 = gpio_b.pb7.into_push_pull_output();
    let led_3 = gpio_b.pb14.into_push_pull_output();
    free(|cs| LED_3.borrow(cs).replace(Some(led_3)));

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
    unsafe { pac::NVIC::unmask(interrupt::EXTI1) } // enable Line1 interrupt (because the pin is PB1)
    encoder_clk.make_interrupt_source(&mut sys_cfg, &mut apb2);
    encoder_clk.trigger_on_edge(&mut exti, Edge::RisingFalling);
    encoder_clk.enable_interrupt(&mut exti);
    unsafe { pac::NVIC::unmask(interrupt::EXTI9_5) } // enable Line5 interrupt (because the pin is PB5)

    encoder_interface::init_encoder(encoder_dt, encoder_clk);

    // LCD setup
    let mut lcd = LCD1602::new(en, rs, d4, d5, d6, d7, d).unwrap();
    // lcd.set_display(true, true, false).unwrap();
    lcd.init_custom_chars().unwrap();
    let mut lcd_scaler = 0u8;

    rprintln!("Ready!");
    led_1.toggle();

    lcd.set_cursor(0, 0).unwrap();
    lcd.print("Encoder: ").unwrap();

    loop {
        // Character animation at 2Hz
        lcd_scaler += 1;
        if lcd_scaler % 25 == 0 {
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
        free(|cs| {
            // left-aligned with 3 digits (including sign)
            lcd.print(&format!(
                "{: <3}",
                encoder_interface::ENCODER_VALUE.borrow(cs).get()
            ))
            .unwrap();
        });

        lcd.delay_ms(50u8); // loop at around 20Hz
    }
}

#[interrupt]
fn EXTI1() {
    encoder_interface::handle_encoder_interrupt(encoder_interface::InterruptedPin::DtPin);
}

#[interrupt]
fn EXTI9_5() {
    encoder_interface::handle_encoder_interrupt(encoder_interface::InterruptedPin::ClkPin);
}

#[interrupt]
fn TIM2() {
    free(|cs| {
        // Clear pending interrupt
        if let Some(ref mut tim2_counter) = TIM2_COUNTER.borrow(cs).borrow_mut().deref_mut() {
            tim2_counter.clear_interrupt(Event::Update);
        }

        // Toggle LED 3
        if let Some(ref mut led_3) = LED_3.borrow(cs).borrow_mut().deref_mut() {
            led_3.toggle();
        }
    })
}
