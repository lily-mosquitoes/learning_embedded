#![no_std]

use core::{
    marker::PhantomData,
    mem::take,
};

static CLOCK_HZ: u32 = 16_000_000;

struct Register {
    pointer: *mut u8,
}

impl Register {
    const fn from(ptr: *mut u8) -> Self {
        Register { pointer: ptr }
    }
}

impl Register {
    fn read(&self) -> u8 {
        unsafe { core::ptr::read_volatile(self.pointer) }
    }

    fn write(&mut self, byte: u8) {
        unsafe {
            core::ptr::write_volatile(self.pointer, byte);
        }
    }
}

pub struct USART<USARTState> {
    udrn: Register,
    ubrrnl: Register,
    ubrrnh: Register,
    ucsrna: Register,
    ucsrnb: Register,
    ucsrnc: Register,
    baudrate_scaler: u16,
    stop_bit_select: USARTStopBit,
    char_size: USARTCharSize,
    mode: USARTMode,
    state: PhantomData<USARTState>,
}

pub struct Unitialized;
pub struct Initialized;
#[repr(u8)]
#[allow(unused)]
pub enum USARTStopBit {
    One = 0,
    Two = 8,
}
#[repr(u8)]
#[allow(unused)]
pub enum USARTCharSize {
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

pub struct Peripheral<T> {
    pub p: Option<T>,
}

impl<T> Peripheral<T> {
    pub fn take(&mut self) -> T {
        let p = take(&mut self.p);
        p.unwrap()
    }
}

pub static mut USART0: Peripheral<USART<Unitialized>> = Peripheral {
    p: Some(USART::<Unitialized> {
        udrn: Register::from(0xC6 as *mut u8),
        ubrrnl: Register::from(0xC4 as *mut u8),
        ubrrnh: Register::from(0xC5 as *mut u8),
        ucsrna: Register::from(0xC0 as *mut u8),
        ucsrnb: Register::from(0xC1 as *mut u8),
        ucsrnc: Register::from(0xC2 as *mut u8),
        baudrate_scaler: 103,
        stop_bit_select: USARTStopBit::Two,
        char_size: USARTCharSize::EightBit,
        mode: USARTMode::Transmit,
        state: PhantomData,
    }),
};

impl USART<Unitialized> {
    pub fn set_baudrate(mut self, baudrate: u32) -> Self {
        let baudrate_scaler = (CLOCK_HZ / (16 * baudrate)) - 1;
        if (baudrate_scaler > 1_000_000) | (baudrate_scaler < 15) {
            self
        } else {
            self.baudrate_scaler = baudrate_scaler as u16;
            self
        }
    }

    pub fn stop_bit_select(mut self, stop_bit: USARTStopBit) -> Self {
        self.stop_bit_select = stop_bit;
        self
    }

    pub fn char_size(mut self, char_size: USARTCharSize) -> Self {
        self.char_size = char_size;
        self
    }

    pub fn set_mode(mut self, mode: USARTMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn initialize(mut self) -> USART<Initialized> {
        let baudrate_scaler = self.baudrate_scaler.to_le_bytes();
        let ucsr0c =
            self.stop_bit_select as u8 + self.char_size as u8;

        // set baudrate_scaler
        self.ubrrnl.write(baudrate_scaler[0]);
        self.ubrrnh.write(baudrate_scaler[1]);

        // set stop_bit_select and char_size
        self.ucsrnc.write(ucsr0c);

        // set mode (transmit/receive)
        self.ucsrnb.write(self.mode as u8);

        USART::<Initialized> {
            udrn: self.udrn,
            ubrrnl: self.ubrrnl,
            ubrrnh: self.ubrrnh,
            ucsrna: self.ucsrna,
            ucsrnb: self.ucsrnb,
            ucsrnc: self.ucsrnc,
            baudrate_scaler: self.baudrate_scaler,
            stop_bit_select: self.stop_bit_select,
            char_size: self.char_size,
            mode: self.mode,
            state: PhantomData,
        }
    }
}

impl USART<Initialized> {
    fn transmit_byte(&mut self, byte: u8) {
        let test_bit_5 = 0b00010000; // if set we are ready to send
        loop {
            let ucsr0a = self.ucsrna.read();
            if ucsr0a >= test_bit_5 {
                break;
            }
        }

        self.udrn.write(byte)
    }

    pub fn transmit_string(&mut self, string: &str) {
        for byte in string.bytes() {
            self.transmit_byte(byte);
        }
    }
}

impl core::fmt::Write for USART<Initialized> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.transmit_string(s);
        Ok(())
    }
}
