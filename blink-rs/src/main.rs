#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

use blink_rs::{
    init_usart,
    send_string,
};

#[no_mangle]
pub extern "C" fn main() -> ! {
    init_usart(9600);

    loop {
        send_string("It works!!!\r\n");
    }
}
