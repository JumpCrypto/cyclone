use crate::{Aligned, Flush, ReadWrite, Write};

#[derive(Copy, Clone)]
/// Mock FPGA, all writes are suppressed, all reads return default values.
pub struct Null(());

impl Null {
    pub fn new() -> Self {
        Self(())
    }
}

impl Default for Null {
    fn default() -> Self {
        Self::new()
    }
}

impl Flush for Null {
    fn flush(&mut self) {}
}

impl Write<u32> for Null {
    fn write(&mut self, _: usize, _: &u32) {}
}

impl ReadWrite<u32> for Null {
    fn read(&self, _: usize) -> u32 {
        0
    }
}

/// Null backoff
pub struct Backoff;
impl<F> crate::Backoff<F> for Backoff {
    fn backoff(_: &mut F, _: usize) {}
}

impl<T> Write<Aligned<T>> for Null {
    fn write(&mut self, _: usize, _: &Aligned<T>) {}
}
