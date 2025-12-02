//! Hardware Abstraction Module
//!
//! This module handles the low-level configuration of the ATmega328p peripherals.
//! It encapsulates the setup of Serial (USART), GPIOs, and ADC,
//! exposing initialized peripherals via a tuple to the main application.

use ufmt::uwriteln;

/// Initializes the entire hardware stack.
///
/// This function:
/// 1.  Takes ownership of the raw Peripherals.
/// 2.  Configures the Serial (USART0) at 57600 baud.
/// 3.  Configures the LED Pin (D13 / PB5).
/// 4.  Sets up the ADC (Prescaler 128, AVCC ref, Interrupt Enabled).
///
/// # Returns
/// A tuple containing `(LedPin, ADC, Serial)`.
pub fn init() -> (
    arduino_hal::port::Pin<arduino_hal::port::mode::Output, arduino_hal::hal::port::PB5>,
    arduino_hal::pac::ADC,
    arduino_hal::hal::usart::Usart0<arduino_hal::DefaultClock>
) {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    
    // Initial log
    uwriteln!(&mut serial, "Program start").unwrap();

    let led_pin = pins.d13.into_output();
    let adc = dp.ADC;

    // ADC Configuration
    adc.admux().write(|w| w.refs().avcc()); 
    adc.adcsra().write(|w| 
        w.aden().set_bit()
         .adie().set_bit()
         .adps().prescaler_128() 
    );

    (led_pin, adc, serial)
}