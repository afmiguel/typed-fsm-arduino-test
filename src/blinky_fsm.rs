// use arduino_hal::prelude::*;
use typed_fsm::{state_machine, Transition};

pub type LedPin = arduino_hal::port::Pin<arduino_hal::port::mode::Output, arduino_hal::hal::port::PB5>;

pub struct BlinkyContext {
    pub led: LedPin,
    pub wait_ticks: u32,
    pub last_adc_value: u16,
    // Extra field for Arduino specifics (not in remote but needed for logic)
    pub adc_peripheral: arduino_hal::pac::ADC, 
}

impl BlinkyContext {
    // Helper to trigger ADC (matches local requirement, used in FSM)
    fn trigger_adc(&mut self) {
        self.adc_peripheral.adcsra().modify(|_, w| w.adsc().set_bit());
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BlinkyEvent {
    TimerTick,
    AdcResult(u16),
}

// Wrapper for global state simplification
pub struct AppState {
    pub fsm: BlinkyFsm,
    pub ctx: BlinkyContext,
}

// --- State Machine ---
state_machine! {
    Name: BlinkyFsm,
    Context: BlinkyContext,
    Event: BlinkyEvent,
    QueueCapacity: 8, // Keep capacity from local project

    States: {
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

        HighValueWait => {
            entry: |ctx| {
                ctx.led.set_low();
                ctx.wait_ticks = 0;
            }
            process: |ctx, evt| {
                match evt {
                    BlinkyEvent::TimerTick => {
                        ctx.wait_ticks += 1;
                        
                        // Fix for Arduino/Logic: Trigger ADC to ensure we get new values
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
