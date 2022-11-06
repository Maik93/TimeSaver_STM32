#![no_std] // just use core Crate
#![no_main] // manually define the function entry
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::format;
use core::panic::PanicInfo;
use cortex_m::interrupt::free;
use cortex_m_rt::entry;
use rtt_target::{rprintln, rtt_init_print};
use stm32f7xx_hal::gpio::{Edge, ExtiPin};
use stm32f7xx_hal::{interrupt, pac, prelude::*};

use lcd1602::custom_characters::{HEART_FULL, MAN_DANCING, MAN_STANDING};
use lcd1602::{DelayMs, LCD1602};

mod encoder_interface;
mod millis;
mod utilities;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    rprintln!("{}", _info);
    loop {}
}

// static LED_3: Mutex<RefCell<Option<Pin<'B', 14, Output>>>> = Mutex::new(RefCell::new(None));

#[derive(PartialEq, Debug)]
enum TimeSaverState {
    Splash,
    Setting,
    Count,
    Alarm,
}

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
    let tim2_counter = dev_perip.TIM2.counter_us(&clocks);
    millis::init(tim2_counter);

    // I/O setup
    let mut led_1 = gpio_b.pb0.into_push_pull_output();
    let mut led_2 = gpio_b.pb7.into_push_pull_output();
    // let led_3 = gpio_b.pb14.into_push_pull_output();
    // free(|cs| LED_3.borrow(cs).replace(Some(led_3)));

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
    let encoder_pushbutton = gpio_b.pb10.into_pull_up_input();

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

    let mut current_state = TimeSaverState::Splash;
    let previous_state = TimeSaverState::Alarm; // this differs from current_state, in order to perform the first one-time action
    let mut seconds_to_go = 0u32;

    rprintln!("Everything is set up!");
    led_1.toggle();

    loop {
        let now_ms = millis::now().unwrap();
        rprintln!("Now is {} ms", now_ms);

        // Get button state
        let button_short_click = encoder_pushbutton.is_low(); // TODO: improve with debounching
        let button_long_click = button_short_click;

        // Trigger state changes
        if button_short_click {
            current_state = match current_state {
                TimeSaverState::Splash => TimeSaverState::Setting,
                TimeSaverState::Setting => TimeSaverState::Count,
                TimeSaverState::Alarm => TimeSaverState::Splash,
                _ => current_state, // keep same state otherwise (just the case of TimeSaverState::Count)
            }
        } else if button_long_click {
            current_state = match current_state {
                TimeSaverState::Count => TimeSaverState::Setting,
                _ => current_state, // keep same state otherwise
            }
        }
        let state_changed = current_state != previous_state;

        // Perform one-time actions needed when the state has changed
        // NOTE: current_state is the new state just achieved
        if state_changed {
            rprintln!("State moved to {:?}", current_state); // enum name can be printed thanks to the Debug trait
            match current_state {
                TimeSaverState::Splash => {
                    lcd.clear().unwrap();
                    lcd.print("Save your time ").unwrap();
                    lcd.write_custom_char(HEART_FULL).unwrap();
                }

                TimeSaverState::Setting => {
                    lcd.clear().unwrap();
                    lcd.print("Set time:").unwrap();
                }

                TimeSaverState::Count => {
                    lcd.clear().unwrap();
                    lcd.print("Be focus...").unwrap();
                    lcd.set_cursor(1, 4).unwrap();
                    lcd.print("min left...").unwrap();
                }

                TimeSaverState::Alarm => {
                    lcd.clear().unwrap();
                    lcd.set_cursor(0, 2).unwrap();
                    lcd.print("TIME IS UP!!").unwrap();
                }
            }
        }

        // Perform timed actions for the current state
        match current_state {
            TimeSaverState::Setting => {
                // Update encoder selection at 20Hz
                if now_ms % 50 == 0 {
                    lcd.set_cursor(1, 4).unwrap();
                    free(|cs| {
                        lcd.print(&format!(
                            "{: <3} min", // left-aligned with 3 digits (including sign)
                            encoder_interface::ENCODER_VALUE.borrow(cs).get() // TODO: think about scaling the encoder value
                        ))
                        .unwrap()
                    });
                }
            }

            TimeSaverState::Count => {
                // Update remaining time at 1Hz
                if now_ms % 1_000 == 0 {
                    seconds_to_go -= 1;

                    // Update timer printed value
                    lcd.set_cursor(1, 0).unwrap();
                    free(|cs| {
                        // left-aligned with 3 digits (including sign)
                        lcd.print(&format!(
                            "{: <3}",
                            encoder_interface::ENCODER_VALUE.borrow(cs).get()
                        ))
                        .unwrap();
                    });
                }

                // Character animation (bottom-right of the screen) at 2Hz
                if now_ms % 500 == 0 {
                    if led_2.is_set_high() {
                        led_2.set_low();
                        lcd.set_cursor(1, 15).unwrap();
                        lcd.write_custom_char(MAN_STANDING).unwrap();
                        // lcd.write_custom_char(HEART_BORDER).unwrap();
                    } else {
                        led_2.set_high();
                        lcd.set_cursor(1, 15).unwrap();
                        lcd.write_custom_char(MAN_DANCING).unwrap();
                        // lcd.write_custom_char(HEART_FULL).unwrap();
                    }
                }
            }

            TimeSaverState::Alarm => {
                // Blink LCD backlight at 2Hz
                if now_ms % 500 == 0 {
                    // TODO
                }
            }

            _ => {}
        }

        cortex_m::asm::wfi();
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
