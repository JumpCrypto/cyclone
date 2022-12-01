use crate::{Error, Fpga, Result, Write};
// TODO: improve
use crate::{Backoff, SendBuffer8, SendBuffer64, Streamable};

pub use fpga_sys as sys;

#[derive(Copy, Clone)]
pub struct F1 {
    __: (),
}

#[cfg(feature = "f1")]
impl F1 {
    pub fn new(slot: i32, offset: i32) -> Result<Self> {
        (unsafe { sys::init_f1(slot, offset) } == 0)
            .then_some(Self { __: () })
            .ok_or(Error::SudoRequired)
    }
}

#[cfg(feature = "f1")]
impl Fpga for F1 {
    fn read_register(&self, index: u32) -> u32 {
        let offset = index << 2;
        // println!("reading");
        unsafe { sys::read_32_f1(offset) }
    }

    fn write_register(&mut self, index: u32, value: u32) {
        let offset = index << 2;
        unsafe { sys::write_32_f1(offset, value) };
    }

    // write an aligned 64-byte chunk of data to an offset
    // since offset must be 64B-aligned too, use index parameter,
    // using offset = index << 6
    fn write8(&mut self, index: usize, buffer: &SendBuffer8) {
        let offset = index << 6;
        let slice: &[u8] = &**buffer;
        unsafe { sys::write_512_f1(offset as u64, &slice[0] as *const _ as _) };
    }

    fn write64(&mut self, index: usize, buffer: &SendBuffer64) {
        let offset = index << 6;
        let slice: &[u64] = &**buffer;
        unsafe { sys::write_512_f1(offset as u64, &slice[0] as *const _ as _) };
    }

    fn flush(&self) {
        unsafe { sys::write_flush() };
    }
}

impl Write<SendBuffer8> for F1 {
    fn write(&mut self, index: usize, buffer: &SendBuffer8) {
        let offset = index << 6;
        let slice: &[u8] = &**buffer;
        unsafe { sys::write_512_f1(offset as u64, &slice[0] as *const _ as _) };
    }
}

impl Write<SendBuffer64> for F1 {
    fn write(&mut self, index: usize, buffer: &SendBuffer64) {
        let offset = index << 6;
        let slice: &[u64] = &**buffer;
        unsafe { sys::write_512_f1(offset as u64, &slice[0] as *const _ as _) };
    }
}

pub struct Stream<'a, B: Backoff<F1>> {
    fpga: &'a mut F1,
    offset: usize,
    backoff: core::marker::PhantomData<B>,
}

impl<'a, B: Backoff<F1>> Streamable<'a, Stream<'a, B>, B> for F1 {
    fn stream(&'a mut self, offset: usize) -> Stream<'a, B> {
        Stream { fpga: self, offset, backoff: core::marker::PhantomData }
    }
}

impl<'a, B: Backoff<F1>> crate::Stream<'a> for Stream<'a, B> {
    type Packet = SendBuffer64;
    fn write(&mut self, packet: &Self::Packet) {
        self.fpga.write64(self.offset, packet);
        self.offset += 1;
        B::backoff(&mut self.fpga, self.offset);
    }
    // #[inline(always)]
    fn offset(&self) -> usize {
        self.offset
    }
    fn flush(&mut self) {
        unsafe { sys::write_flush() };
    }
}

