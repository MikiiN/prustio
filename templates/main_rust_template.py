def get_sample_code_content():
    return r"""#![no_std]
#![no_main]

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut led = pins.d13.into_output();
    loop {
        led.toggle();
        arduino_hal::delay_ms(500);
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}"""


def get_minimal_content():
    return r"""#![no_std]
#![no_main]

#[arduino_hal::entry]
fn main() -> ! {
    loop {
        
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}"""