pub struct Peripheral<T> {
    pub(crate) inner: Option<T>,
}

impl<T> Peripheral<T> {
    pub fn take(&mut self) -> T {
        self.inner.take().unwrap()
    }
}
