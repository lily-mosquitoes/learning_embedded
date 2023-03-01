pub(crate) struct Register {
    address: u8,
}

impl Register {
    pub(crate) const fn from(address: u8) -> Self {
        Register { address }
    }
}

impl Register {
    pub(crate) fn read(&self) -> u8 {
        unsafe { core::ptr::read_volatile(self.address as *mut u8) }
    }

    pub(crate) fn write(&mut self, byte: u8) {
        unsafe {
            core::ptr::write_volatile(self.address as *mut u8, byte);
        }
    }
}
