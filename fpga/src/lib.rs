//! # fpga
//!
//! Rust bindings to interact with AWS F1 FPGAs.

use thiserror::Error;

pub mod align;
pub use align::{aligned, Aligned};

#[cfg(feature = "f1")]
pub mod f1;
#[cfg(feature = "f1")]
pub use f1::F1;

mod null;
pub use null::Null;

#[derive(Debug, Error)]
pub enum Error {
    #[error("FPGA drivers require running as root.")]
    SudoRequired,
}

pub type Result<T> = core::result::Result<T, Error>;

pub type SendBuffer8 = Aligned<[u8; 64]>;
pub type SendBuffer64 = Aligned<[u64; 8]>;

#[allow(unused_variables)]
pub trait Fpga {
    // type Read: AsRef<[u8]>;
    // type Write: AsMut<[u8]>;

    fn read_register(&self, index: u32) -> u32;
    /// NB: still using offset, not index
    /// figure out if we need this
    fn write_register(&mut self, index: u32, value: u32);

    fn write8(&mut self, index: usize, buffer: &SendBuffer8) {}
    fn write64(&mut self, index: usize, buffer: &SendBuffer64) {}

    fn flush(&self) {}
}

pub trait ReadWrite<FPGA>: Sized {
    fn new(fpga: FPGA, offset: u32, len: u32) -> Result<Self>;
    fn read(&self, index: u32) -> u32;
    fn write(&mut self, index: u32, value: u32);
}

pub trait Backoff<F> {
    fn backoff(fpga: &mut F, offset: usize);
}

pub trait Streamable<'a, S: Stream<'a>, B: Backoff<Self>>: Sized {
    fn stream(&'a mut self, offset: usize) -> S;
}

pub trait Stream<'a> {
    type Packet;
    fn write(&mut self, buffer: &Self::Packet);
    fn flush(&mut self);
    fn offset(&self) -> usize;
}

pub trait Write<Buffer>: Fpga {
    fn write(&mut self, index: usize, buffer: &Buffer);
}
