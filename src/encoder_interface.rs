use core::cell::{Cell, RefCell};
use core::ops::DerefMut;
use cortex_m::interrupt::{free, Mutex};
use rotary_encoder_embedded::{standard::StandardMode, Direction, RotaryEncoder};
use stm32f7xx_hal::gpio::{ExtiPin, Input, Pin, PullUp};

/// Interrupt-safe Encoder handler.
pub static ROTARY_ENCODER: Mutex<
    RefCell<
        Option<RotaryEncoder<StandardMode, Pin<'B', 1, Input<PullUp>>, Pin<'B', 5, Input<PullUp>>>>,
    >,
> = Mutex::new(RefCell::new(None));

static ENCODER_VALUE: Mutex<Cell<i32>> = Mutex::new(Cell::new(0i32));

/// Initialize memory-safe encoder handler for DT -> PB1 and CLK -> PB5.
pub fn init_encoder(dt: Pin<'B', 1, Input<PullUp>>, clk: Pin<'B', 5, Input<PullUp>>) {
    // Encoder setup
    free(|cs| {
        ROTARY_ENCODER
            .borrow(cs)
            .replace(Some(RotaryEncoder::new(dt, clk).into_standard_mode()));
    });
}

/// Get current encoder value.
pub fn get() -> i32 {
    free(|cs| ENCODER_VALUE.borrow(cs).get())
}

/// Set encoder internal value to desired value.
pub fn set(v: i32) {
    free(|cs| ENCODER_VALUE.borrow(cs).replace(v));
}

/// Reset encoder value to 0.
pub fn reset() {
    set(0);
}

/// Interrupted pin, used for interrupt resets.
pub enum InterruptedPin {
    DtPin,
    ClkPin,
}

/// Update current encoder values during External Interrupt (rising and falling edges on DT and CLK pins).
pub fn handle_encoder_interrupt(interrupted_pin: InterruptedPin) {
    // Retrieve Rotary Encoder from safely stored static global
    free(|cs| {
        if let Some(ref mut rotary_encoder) = ROTARY_ENCODER.borrow(cs).borrow_mut().deref_mut() {
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
                    let cell = ENCODER_VALUE.borrow(cs);
                    cell.replace(cell.get() + 1);
                }
                Direction::Anticlockwise => {
                    let cell = ENCODER_VALUE.borrow(cs);
                    cell.replace(cell.get() - 1);
                }
                Direction::None => {
                    // Do nothing
                }
            }
        }
    });
}
