#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(asm_experimental_arch)]

// --- Imports ---
use core::cell::RefCell;
use critical_section::Mutex;
use panic_halt as _;
use ufmt::uwriteln;

// --- Modules ---
mod blinky_fsm;
use blinky_fsm::{AppState, BlinkyContext, BlinkyEvent, BlinkyFsm};

mod hardware;

// --- Shared State ---
// Improvement: Unified global state to avoid double locking
static GLOBAL_STATE: Mutex<RefCell<Option<AppState>>> = Mutex::new(RefCell::new(None));

// --- Interrupt Handlers ---
#[avr_device::interrupt(atmega328p)]
fn ADC() {
    // Safe access to peripheral in ISR
    let adc_ptr = unsafe { &*arduino_hal::pac::ADC::PTR };
    let value = adc_ptr.adc().read().bits();

    // Dispatch AdcResult Event directly to global FSM using Critical Section
    critical_section::with(|cs| {
        if let Some(state) = GLOBAL_STATE.borrow_ref_mut(cs).as_mut() {
            state.fsm.dispatch(&mut state.ctx, &BlinkyEvent::AdcResult(value));
        }
    });
}

// --- Main ---
#[arduino_hal::entry]
fn main() -> ! {
    // 1. Initialize Hardware Stack (Tuple destructuring - Improvement 2)
    let (led_pin, adc, mut serial) = hardware::init();

    // 2. Initialize Application State (FSM)
    let mut ctx = BlinkyContext {
        led: led_pin,
        wait_ticks: 0,
        last_adc_value: 0,
        adc_peripheral: adc,
    };
    let mut fsm = BlinkyFsm::LedOff;
    
    // Initialize FSM with Context (entry actions run here)
    fsm.init(&mut ctx);

    // 3. Publish to Unified Global State (Improvement 1 & 3)
    critical_section::with(|cs| {
        GLOBAL_STATE.borrow_ref_mut(cs).replace(AppState { fsm, ctx });
    });

    // Enable global interrupts
    unsafe {
        avr_device::interrupt::enable();
    }

    // 4. Main Application Loop
    loop {
        arduino_hal::delay_ms(200);

        let mut current_state_str = "Unknown";

        // Access Unified State
        critical_section::with(|cs| {
            if let Some(state) = GLOBAL_STATE.borrow_ref_mut(cs).as_mut() {
                state.fsm.dispatch(&mut state.ctx, &BlinkyEvent::TimerTick);

                match state.fsm {
                    BlinkyFsm::LedOff => current_state_str = "OFF",
                    BlinkyFsm::LedOn => current_state_str = "ON",
                    BlinkyFsm::HighValueWait => current_state_str = "WAIT_HIGH_VALUE",
                }
            }
        });

        // Log state to Serial
        uwriteln!(&mut serial, "State: {}", current_state_str).unwrap();
    }
}
