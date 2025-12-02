// use arduino_hal::prelude::*;
use ufmt::uwriteln;

// --- Hardware Abstraction ---

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

    // ADC Configuration (Same as before)
    adc.admux().write(|w| w.refs().avcc()); 
    adc.adcsra().write(|w| 
        w.aden().set_bit()
         .adie().set_bit()
         .adps().prescaler_128() 
    );

    (led_pin, adc, serial)
}