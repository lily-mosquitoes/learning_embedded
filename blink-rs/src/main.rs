#![no_std]
#![no_main]
#![feature(exclusive_wrapper)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::{
    fmt::Write,
    sync::Exclusive,
};

use blink_rs::usart::{
    Initialized,
    USART,
    USART0,
};

static mut SERIAL: Exclusive<Option<USART<Initialized>>> =
    Exclusive::new(None);

fn get_serial() -> &'static mut USART<Initialized> {
    unsafe {
        let serial = SERIAL.get_mut();
        if serial.is_none() {
            *serial = Some(USART0.take().initialize());
        }
        serial.as_mut().unwrap()
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    writeln!(get_serial(), "Program panicked!\nError: {}", info,)
        .unwrap();

    loop {}
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() -> ! {
    writeln!(get_serial(), "Hello there~, {}!", "world").unwrap();

    loop {}
}

#[cfg(test)]
trait Testable {
    fn run(&self);
}

#[cfg(test)]
impl<T: Fn()> Testable for T {
    fn run(&self) {
        write!(get_serial(), "{}...\t", core::any::type_name::<T>(),)
            .unwrap();
        self();
        writeln!(get_serial(), "[ok]",).unwrap();
    }
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Testable]) {
    writeln!(get_serial(), "Running {} tests:", tests.len()).unwrap();

    for test in tests {
        test.run();
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    writeln!(get_serial(), "[failed]",).unwrap();
    writeln!(get_serial(), "Error: {}", info,).unwrap();
    loop {}
}

#[cfg(test)]
#[no_mangle]
extern "C" fn main() -> ! {
    test_main();
    loop {}
}

#[cfg(test)]
pub mod tests {
    #[test_case]
    fn passing() {
        assert_eq!(1, 1);
    }

    #[test_case]
    fn failing() {
        assert_eq!(1, 2);
    }
}
