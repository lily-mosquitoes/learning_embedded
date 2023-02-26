#![no_std]

use core::marker::PhantomData;

// definitions for the ATmega328p microprocessor
// see: https://content.arduino.cc/assets/Atmel-7810-Automotive-Microcontrollers-ATmega328P_Datasheet.pdf?_gl=1*1y8m4t*_ga*NTQ5Mjg5MDUuMTY3NDQxMTg4Mg..*_ga_NEXN8H46L5*MTY3NDQyNDk4NC4zLjEuMTY3NDQyNTUxOS4wLjAuMA..#G1446728
// USART0 I/O data register
const UDR0: *mut u8 = 0xC6 as *mut u8;
// USART0 Baudrate (Scaler) registers
const UBRR0L: *mut u8 = 0xC4 as *mut u8;
const UBRR0H: *mut u8 = 0xC5 as *mut u8;
// USART0 Control and Status Registers C, B and A
const UCSR0C: *mut u8 = 0xC2 as *mut u8;
const UCSR0B: *mut u8 = 0xC1 as *mut u8;
const UCSR0A: *mut u8 = 0xC0 as *mut u8;

static CLOCK_HZ: u32 = 16_000_000;

pub struct USART0<State> {
    baudrate_scaler: u16,
    stop_bit_select: StopBit,
    char_size: CharSize,
    mode: USARTMode,
    state: PhantomData<State>,
}

pub struct Unitialized;
pub struct Initialized;
#[repr(u8)]
#[allow(unused)]
pub enum StopBit {
    One = 0,
    Two = 8,
}
#[repr(u8)]
#[allow(unused)]
pub enum CharSize {
    FiveBit = 0,
    SixBit = 2,
    SevenBit = 4,
    EightBit = 6,
}
#[repr(u8)]
#[allow(unused)]
pub enum USARTMode {
    Disabled = 0,
    Transmit = 8,
    Receive = 16,
    TransmitAndReceive = 24,
}

impl USART0<Unitialized> {
    pub fn new() -> Self {
        USART0 {
            baudrate_scaler: 103,
            stop_bit_select: StopBit::Two,
            char_size: CharSize::EightBit,
            mode: USARTMode::Transmit,
            state: PhantomData,
        }
    }

    pub fn set_baudrate(mut self, baudrate: u32) -> Self {
        let baudrate_scaler = (CLOCK_HZ / (16 * baudrate)) - 1;
        if (baudrate_scaler > 1_000_000) | (baudrate_scaler < 15) {
            self
        } else {
            self.baudrate_scaler = baudrate_scaler as u16;
            self
        }
    }

    pub fn stop_bit_select(mut self, stop_bit: StopBit) -> Self {
        self.stop_bit_select = stop_bit;
        self
    }

    pub fn char_size(mut self, char_size: CharSize) -> Self {
        self.char_size = char_size;
        self
    }

    pub fn set_mode(mut self, mode: USARTMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn initialize(self) -> USART0<Initialized> {
        let baudrate_scaler = self.baudrate_scaler.to_le_bytes();
        let ucsr0c =
            self.stop_bit_select as u8 + self.char_size as u8;
        assert_eq!(ucsr0c, 14);

        unsafe {
            // set baudrate_scaler
            core::ptr::write_volatile(UBRR0L, baudrate_scaler[0]);
            core::ptr::write_volatile(UBRR0H, baudrate_scaler[1]);

            // set stop_bit_select and char_size
            core::ptr::write_volatile(UCSR0C, ucsr0c);

            // set mode (transmit/receive)
            core::ptr::write_volatile(UCSR0B, self.mode as u8);
        }

        USART0::<Initialized> {
            baudrate_scaler: self.baudrate_scaler,
            stop_bit_select: self.stop_bit_select,
            char_size: self.char_size,
            mode: self.mode,
            state: PhantomData,
        }
    }
}

impl USART0<Initialized> {
    unsafe fn send_byte(&self, byte: u8) {
        let test_bit_5 = 0b00010000; // if set we are ready to send
        loop {
            let ucsr0a = core::ptr::read_volatile(UCSR0A);
            if ucsr0a >= test_bit_5 {
                break;
            }
        }

        core::ptr::write_volatile(UDR0, byte);
    }

    pub fn send_string(&self, string: &str) {
        for byte in string.bytes() {
            unsafe {
                self.send_byte(byte);
            }
        }
    }
}
