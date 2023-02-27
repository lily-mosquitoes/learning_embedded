#![no_std]
#![no_main]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

use core::fmt::Write;

use blink_rs::USART0;

#[no_mangle]
pub extern "C" fn main() -> ! {
    let mut usart0 = unsafe { USART0.take().initialize() };
    writeln!(usart0, "Hello there~, {}!", "Emilia").unwrap();

    loop {
        // writeln!(usart0, "Hello there~, {}!", "Emilia").unwrap();
    }
}
