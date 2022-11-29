//! # fpga
//!
//! Rust bindings to interact with AWS F1 FPGAs.

use thiserror::Error;

pub mod align;
pub use align::{aligned, Aligned};

mod buffer;
#[cfg(feature = "f1")]
mod f1;
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

#[derive(Copy, Clone, Debug)]
pub struct ReceiveBuffer([u8; 56]);
impl ReceiveBuffer {
    pub fn as_u64_slice(&self) -> &[u64] {
        unsafe { core::slice::from_raw_parts(&self.0 as *const u8 as *const u64, 7) }
    }
}

#[allow(unused_variables)]
pub trait Fpga {
    // type Read: AsRef<[u8]>;
    // type Write: AsMut<[u8]>;

    fn read_register(&self, index: u32) -> u32;
    /// NB: still using offset, not index
    /// figure out if we need this
    fn write_register(&mut self, index: u32, value: u32);

    fn read(&mut self, buffer: &mut ReceiveBuffer) {}
    // fn receive_alloc(&mut self) -> ReceiveBuffer {
    //     let mut buffer = ReceiveBuffer::default();
    //     self.receive(&mut buffer);
    //     buffer
    // }
    fn write8(&mut self, index: usize, buffer: &SendBuffer8) {}
    fn write64(&mut self, index: usize, buffer: &SendBuffer64) {}

    fn flush(&self) {}
}

// pub trait Write<Buffer: AsRef<[u8]>>: Fpga {
pub trait Write<Buffer>: Fpga {
    fn write(&mut self, index: usize, buffer: &Buffer);
}
