//! # fpga
//!
//! Rust bindings to interact with AWS F1 FPGAs.

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
    type Index: Copy;

    /// write value to index
    fn write(&mut self, index: Self::Index, value: &V);
}

/// Index-based read/writes to an FPGA.
pub trait ReadWrite<V>: Write<V> {
    /// read value at index
    fn read(&self, index: Self::Index) -> V;
}

/// App-specific backoff mechanism used in streaming.
pub trait Backoff<FPGA, I> {
    fn backoff(fpga: &mut FPGA, offset: I);
}

/// Streaming writes to an FPGA.
///
/// The backoff depends on the FPGA app's implementation of streaming.
pub struct Stream<'a, P, FPGA: Write<P>, B = null::Backoff> {
    fpga: &'a mut FPGA,
    offset: FPGA::Index,
    backoff: core::marker::PhantomData<B>,
}

/// Marker trait for FPGAs supporting streaming writes.
pub trait Streamable<'a, P, B: Backoff<Self, Self::Index> = null::Backoff>:
    Sized + Write<P>
{
    /// initialize new stream
    fn stream(&'a mut self, offset: Self::Index) -> Stream<'a, P, Self, B>;
}

impl<'a, P, FPGA: Write<P>, B: Backoff<FPGA, FPGA::Index>> Streamable<'a, P, B> for FPGA {
    fn stream(&'a mut self, offset: FPGA::Index) -> Stream<'a, P, FPGA, B> {
        Stream {
            fpga: self,
            offset,
            backoff: core::marker::PhantomData,
        }
    }
}

impl<'a, P, FPGA: Flush + Write<P>, B> Flush for Stream<'a, P, FPGA, B> {
    fn flush(&mut self) {
        self.fpga.flush()
    }
}

/// Helper trait to keep track of index offsets in streaming.
pub trait Incrementable: Copy {
    fn increment(&mut self);
}

impl Incrementable for u32 {
    fn increment(&mut self) {
        *self += 1;
    }
}

impl Incrementable for u64 {
    fn increment(&mut self) {
        *self += 1;
    }
}

impl Incrementable for usize {
    fn increment(&mut self) {
        *self += 1;
    }
}

impl<'a, P, FPGA: Write<P>, B> Stream<'a, P, FPGA, B> {
    pub fn fpga(&mut self) -> &mut FPGA {
        self.fpga
    }

    #[inline(always)]
    pub fn offset(&self) -> FPGA::Index {
        self.offset
    }
}

impl<'a, P, FPGA: Write<P>, B: Backoff<FPGA, FPGA::Index>> Stream<'a, P, FPGA, B>
where
    FPGA::Index: Incrementable,
{
    #[inline(always)]
    pub fn write(&mut self, packet: &P) {
        self.fpga.write(self.offset, packet);
        self.offset.increment();
        B::backoff(self.fpga, self.offset);
    }
}
