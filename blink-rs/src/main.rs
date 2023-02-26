#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

use blink_rs::USART0;

#[no_mangle]
pub extern "C" fn main() -> ! {
    let usart0 = USART0::new().initialize();

    loop {
        usart0.send_string("Hello there~\r\n")
    }
}
