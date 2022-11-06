//! Millis
//! Count milliseconds from start time, through interrupts on TIM2.

use core::cell::{Cell, RefCell};
use core::ops::DerefMut;

use cortex_m::interrupt::{free, Mutex};
use stm32f7xx_hal::pac::TIM2;
use stm32f7xx_hal::timer::{CounterUs, Event};
use stm32f7xx_hal::{interrupt, pac, prelude::*};

/// Accuracy of the counter, in milliseconds.
const ACCURACY_MS: u32 = 5;

static MILLIS: Mutex<RefCell<Option<MillisStruct>>> = Mutex::new(RefCell::new(None));

struct MillisStruct {
    current_ms: Cell<u32>,
    tim2: RefCell<CounterUs<TIM2>>,
}

#[derive(Debug)]
pub enum Error {
    TimerNotInitialised,
}

/// Initialise millis counting, using TIM2.
pub fn init(mut tim2_counter: CounterUs<TIM2>) {
    // Set timer counts
    tim2_counter.start(ACCURACY_MS.millis()).unwrap();
    tim2_counter.listen(Event::Update);

    // Initialise static struct
    free(|cs| {
        let millis = MILLIS.borrow(cs);
        millis.replace(Some(MillisStruct {
            current_ms: Cell::new(0u32),
            tim2: RefCell::new(tim2_counter),
        }));
    });

    // Enable timer interrupts
    unsafe { pac::NVIC::unmask(interrupt::TIM2) };
}

/// Get current milliseconds.
pub fn now() -> Result<u32, Error> {
    free(|cs| {
        match MILLIS.borrow(cs).borrow_mut().deref_mut() {
            Some(millis) => Ok(millis.current_ms.get()),
            None => Err(Error::TimerNotInitialised),
        }
    })
}

#[interrupt]
fn TIM2() {
    free(|cs| {
        if let Some(millis) = MILLIS.borrow(cs).borrow_mut().deref_mut() {
            // Increment millis counter
            millis.current_ms.replace(millis.current_ms.get() + ACCURACY_MS);

            // Clear pending interrupt
            millis.tim2.borrow_mut().clear_interrupt(Event::Update);
        }
    })
}
