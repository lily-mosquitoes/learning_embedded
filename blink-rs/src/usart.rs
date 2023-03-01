use core::{
    fmt::Display,
    marker::PhantomData,
    num::NonZeroU32,
};

use crate::{
    peripheral::Peripheral,
    register::Register,
};

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

pub struct Unitialized;

pub struct Initialized;

pub struct USART<State> {
    udrn: Register,
    ubrrnl: Register,
    ubrrnh: Register,
    ucsrna: Register,
    ucsrnb: Register,
    ucsrnc: Register,
    clockrate_hz: u32,
    baudrate_scaler: u16,
    stop_bit_select: USARTStopBit,
    char_size: USARTCharSize,
    mode: USARTMode,
    state: PhantomData<State>,
}

#[derive(Debug, Clone)]
pub struct IncompatibleSettings;

impl Display for IncompatibleSettings {
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        write!(f, "incompatible settings")
    }
}

impl USART<Unitialized> {
    pub fn set_clockrate_hz_and_baudrate(
        mut self,
        clockrate_hz: NonZeroU32,
        baudrate: NonZeroU32,
    ) -> Result<Self, IncompatibleSettings> {
        let baudrate_scaler =
            (clockrate_hz.get() / (16 * baudrate.get())) - 1;
        if baudrate_scaler > 4095 {
            Err(IncompatibleSettings)
        } else {
            self.clockrate_hz = clockrate_hz.get();
            self.baudrate_scaler = baudrate_scaler as u16;
            Ok(self)
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
            clockrate_hz: self.clockrate_hz,
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

        self.udrn.write(byte);
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

pub static mut USART0: Peripheral<USART<Unitialized>> = Peripheral {
    inner: Some(USART::<Unitialized> {
        udrn: Register::from(0xc6),
        ubrrnl: Register::from(0xc4),
        ubrrnh: Register::from(0xc5),
        ucsrna: Register::from(0xc0),
        ucsrnb: Register::from(0xc1),
        ucsrnc: Register::from(0xc2),
        clockrate_hz: 16_000_000,
        baudrate_scaler: 103,
        stop_bit_select: USARTStopBit::Two,
        char_size: USARTCharSize::EightBit,
        mode: USARTMode::Transmit,
        state: PhantomData,
    }),
};
