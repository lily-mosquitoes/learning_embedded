#![no_std]

// definitions for the ATmega328p microprocessor
// see: https://content.arduino.cc/assets/Atmel-7810-Automotive-Microcontrollers-ATmega328P_Datasheet.pdf?_gl=1*1y8m4t*_ga*NTQ5Mjg5MDUuMTY3NDQxMTg4Mg..*_ga_NEXN8H46L5*MTY3NDQyNDk4NC4zLjEuMTY3NDQyNTUxOS4wLjAuMA..#G1446728

const UBRR0L: *mut u8 = 0xC4 as *mut u8;
const UBRR0H: *mut u8 = 0xC5 as *mut u8;
const UDR0: *mut u8 = 0xC6 as *mut u8;

const UCSR0A: *mut u8 = 0xC0 as *mut u8;
const UCSR0B: *mut u8 = 0xC1 as *mut u8;
const UCSR0C: *mut u8 = 0xC2 as *mut u8;

static CLOCK_HZ: u32 = 16_000_000;

pub fn init_usart(baud_rate: u32) {
    // baud_rate max: 1_000_000, min: 15
    let baud_rate_scaler = ((CLOCK_HZ / (16 * baud_rate)) - 1) as u16;
    let baud_rate_scaler = baud_rate_scaler.to_le_bytes();
    unsafe {
        core::ptr::write_volatile(UBRR0L, baud_rate_scaler[0]);
        core::ptr::write_volatile(UBRR0H, baud_rate_scaler[1]);
    }
    // enable trasmit bit
    let txne0: u8 = 3;
    unsafe {
        core::ptr::write_volatile(UCSR0B, 1 << txne0);
    }
    // set 8 bits of message (bit 2:1 = 11)
    // set 2 stop bits (bit 3 = 1)
    unsafe {
        core::ptr::write_volatile(UCSR0C, 7 << 1);
    }
}

unsafe fn send_byte(byte: u8) {
    let test_bit_5 = 0b00010000; // if set we are ready to send
    loop {
        let ucsr0a = core::ptr::read_volatile(UCSR0A);
        if ucsr0a >= test_bit_5 {
            break;
        }
    }

    core::ptr::write_volatile(UDR0, byte);
}

pub fn send_string(string: &str) {
    for byte in string.bytes() {
        unsafe {
            send_byte(byte);
        }
    }
}
