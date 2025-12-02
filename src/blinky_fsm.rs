//! Typed Finite State Machine Module
//!
//! This module defines the application logic using the `typed-fsm` macro.
//! It contains the State Machine (`BlinkyFsm`), the Context (`BlinkyContext`),
//! and the Events (`BlinkyEvent`).

// use arduino_hal::prelude::*;
use typed_fsm::{state_machine, Transition};

/// Alias for the LED Pin type (PB5 on ATmega328p).
pub type LedPin = arduino_hal::port::Pin<arduino_hal::port::mode::Output, arduino_hal::hal::port::PB5>;

/// FSM Context.
/// Holds the resources and state variables required by the State Machine.
pub struct BlinkyContext {
    pub led: LedPin,
    pub wait_ticks: u32,
    pub last_adc_value: u16,
    // ADC peripheral access required for triggering conversions on ATmega328p
    pub adc_peripheral: arduino_hal::pac::ADC, 
}

impl BlinkyContext {
    /// Helper to trigger a new ADC conversion.
    fn trigger_adc(&mut self) {
        self.adc_peripheral.adcsra().modify(|_, w| w.adsc().set_bit());
    }
}

/// FSM Events.
#[derive(Clone, Copy, Debug)]
pub enum BlinkyEvent {
    /// Periodic timer tick (simulated in main loop).
    TimerTick,
    /// Result of an ADC conversion (from ISR).
    AdcResult(u16),
}

/// Wrapper struct to unify FSM and Context into a single global resource.
pub struct AppState {
    pub fsm: BlinkyFsm,
    pub ctx: BlinkyContext,
}

// State Machine Definition
state_machine! {
    Name: BlinkyFsm,
    Context: BlinkyContext,
    Event: BlinkyEvent,
    QueueCapacity: 8,

    States: {
        // State: LED Off
        LedOff => {
            entry: |ctx| {
                ctx.led.set_low();
            }
            process: |_ctx, evt| {
                match evt {
                    BlinkyEvent::TimerTick => Transition::To(BlinkyFsm::LedOn),
                    BlinkyEvent::AdcResult(_) => Transition::None,
                }
            }
        },

        // State: LED On
        LedOn => {
            entry: |ctx| {
                ctx.led.set_high();
                ctx.trigger_adc();
            }
            process: |_ctx, evt| {
                match evt {
                    BlinkyEvent::TimerTick => Transition::To(BlinkyFsm::LedOff),
                    BlinkyEvent::AdcResult(val) => {
                        if *val > 70 {
                            Transition::To(BlinkyFsm::HighValueWait)
                        } else {
                            Transition::None
                        }
                    }
                }
            }
        },

        // State: High Value Wait (Cooldown)
        HighValueWait => {
            entry: |ctx| {
                ctx.led.set_low();
                ctx.wait_ticks = 0;
            }
            process: |ctx, evt| {
                match evt {
                    BlinkyEvent::TimerTick => {
                        ctx.wait_ticks += 1;
                        
                        // Trigger new ADC conversion to check exit condition
                        ctx.trigger_adc(); 

                        if ctx.wait_ticks >= 10 && ctx.last_adc_value <= 70 {
                            Transition::To(BlinkyFsm::LedOff) 
                        } else {
                            Transition::None 
                        }
                    },
                    BlinkyEvent::AdcResult(val) => {
                        ctx.last_adc_value = *val;
                        Transition::None 
                    },
                }
            }
        }
    }
}