use crate::{Aligned, Flush, ReadWrite, Write};

#[derive(Copy, Clone)]
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
    type Index = u32;

    fn write(&mut self, _: u32, _: &u32) {}
}

impl ReadWrite<u32> for Null {
    fn read(&self, _: u32) -> u32 { 0 }
}

/// Null backoff
pub struct Backoff;
impl<F, I> crate::Backoff<F, I> for Backoff {
    fn backoff(_: &mut F, _: I) {}
}

impl<T> Write<Aligned<T>> for Null {
    type Index = usize;

    fn write(&mut self, _: usize, _: &Aligned<T>) {}
}

