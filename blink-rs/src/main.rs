#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    let mut usart0 = unsafe { USART0.take().initialize() };
    writeln!(usart0, "Running tests from main").unwrap();
    for test in tests {
        test();
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

use core::fmt::Write;

use blink_rs::USART0;

#[no_mangle]
pub extern "C" fn main() -> ! {
    #[cfg(test)]
    test_main();
    let mut usart0 = unsafe { USART0.take().initialize() };
    writeln!(usart0, "Hello there~, {}!", "world").unwrap();

    loop {}
}

#[test_case]
fn trivial() {
    assert_eq!(1, 1);
}
