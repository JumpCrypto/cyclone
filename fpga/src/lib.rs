//! # fpga
//!
//! Rust traits to interact with FPGAs.
//!
//! Implemented for AWS F1 FPGAs.

use core::marker::PhantomData;
use thiserror::Error;

pub mod align;
pub use align::Aligned;

#[cfg(feature = "f1")]
pub mod f1;
#[cfg(feature = "f1")]
pub use f1::F1;

pub mod null;
pub use null::Null;

#[derive(Debug, Error)]
pub enum Error {
    #[error("FPGA drivers require running as root.")]
    SudoRequired,
}

pub type Result<T> = core::result::Result<T, Error>;

/// Flush communications
pub trait Flush {
    /// flush communications
    fn flush(&mut self);
}

/// Index-based writes to an FPGA.
pub trait Write<V>: Flush {
    /// write value to index
    fn write(&mut self, index: usize, value: &V);
}

/// Index-based read/writes to an FPGA.
pub trait ReadWrite<V>: Write<V> {
    /// read value at index
    fn read(&self, index: usize) -> V;
}

/// App-specific backoff mechanism used in streaming.
pub trait Backoff<FPGA> {
    fn backoff(fpga: &mut FPGA, offset: usize);
}

/// Streaming writes to an FPGA.
///
/// The backoff depends on the FPGA app's implementation of streaming.
pub struct Stream<'a, P, FPGA: Write<P>, B = null::Backoff> {
    fpga: &'a mut FPGA,
    offset: usize,
    __: PhantomData<(B, P)>,
}

/// Marker trait for FPGAs supporting streaming writes.
pub trait Streamable<'a, P, B: Backoff<Self> = null::Backoff>: Sized + Write<P> {
    /// initialize new stream
    fn stream(&'a mut self, offset: usize) -> Stream<'a, P, Self, B>;
}

impl<'a, P, FPGA: Write<P>, B: Backoff<FPGA>> Streamable<'a, P, B> for FPGA {
    fn stream(&'a mut self, offset: usize) -> Stream<'a, P, FPGA, B> {
        Stream {
            fpga: self,
            offset,
            __: PhantomData,
        }
    }
}

impl<'a, P, FPGA: Flush + Write<P>, B> Flush for Stream<'a, P, FPGA, B> {
    fn flush(&mut self) {
        self.fpga.flush()
    }
}

impl<'a, P, FPGA: Write<P>, B> Stream<'a, P, FPGA, B> {
    pub fn fpga(&mut self) -> &mut FPGA {
        self.fpga
    }

    #[inline(always)]
    pub fn offset(&self) -> usize {
        self.offset
    }
}

impl<'a, P, FPGA: Write<P>, B: Backoff<FPGA>> Stream<'a, P, FPGA, B> {
    #[inline(always)]
    pub fn write(&mut self, packet: &P) {
        self.fpga.write(self.offset, packet);
        self.offset += 1;
        B::backoff(self.fpga, self.offset);
    }
}
