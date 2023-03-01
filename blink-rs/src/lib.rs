#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(exclusive_wrapper)]

use core::num::NonZeroU32;
#[cfg(test)]
use core::{
    fmt::Write,
    sync::Exclusive,
};

#[cfg(test)]
static mut SERIAL: Exclusive<Option<USART<Initialized>>> =
    Exclusive::new(None);

#[cfg(test)]
trait Testable {
    fn run(&self);
}

#[cfg(test)]
impl<T: Fn()> Testable for T {
    fn run(&self) {
        write!(
            unsafe { SERIAL.get_mut().as_mut().unwrap() },
            "{}...\t",
            core::any::type_name::<T>(),
        )
        .unwrap();
        self();
        writeln!(
            unsafe { SERIAL.get_mut().as_mut().unwrap() },
            "[ok]",
        )
        .unwrap();
    }
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Testable]) {
    unsafe {
        *SERIAL.get_mut() = Some(USART0.take().initialize());
    }
    writeln!(
        unsafe { SERIAL.get_mut().as_mut().unwrap() },
        "Running {} tests from lib",
        tests.len()
    )
    .unwrap();

    for test in tests {
        test.run();
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    writeln!(
        unsafe { SERIAL.get_mut().as_mut().unwrap() },
        "[failed]",
    )
    .unwrap();
    writeln!(
        unsafe { SERIAL.get_mut().as_mut().unwrap() },
        "Error: {}",
        info,
    )
    .unwrap();
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

use core::marker::PhantomData;

struct Register {
    address: u8,
}

impl Register {
    const fn from(address: u8) -> Self {
        Register { address }
    }
}

impl Register {
    fn read(&self) -> u8 {
        unsafe { core::ptr::read_volatile(self.address as *mut u8) }
    }

    fn write(&mut self, byte: u8) {
        unsafe {
            core::ptr::write_volatile(self.address as *mut u8, byte);
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
    clockrate_hz: u32,
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
    inner: Option<T>,
}

impl<T> Peripheral<T> {
    pub fn take(&mut self) -> T {
        self.inner.take().unwrap()
    }
}

pub static mut USART0: Peripheral<USART<Unitialized>> = Peripheral {
    inner: Some(USART::<Unitialized> {
        udrn: Register::from(0xC6),
        ubrrnl: Register::from(0xC4),
        ubrrnh: Register::from(0xC5),
        ucsrna: Register::from(0xC0),
        ucsrnb: Register::from(0xC1),
        ucsrnc: Register::from(0xC2),
        clockrate_hz: 16_000_000,
        baudrate_scaler: 103,
        stop_bit_select: USARTStopBit::Two,
        char_size: USARTCharSize::EightBit,
        mode: USARTMode::Transmit,
        state: PhantomData,
    }),
};

pub struct IncompatibleSettings;

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
