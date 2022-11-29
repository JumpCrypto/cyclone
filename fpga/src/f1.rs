use core::ptr;
use crate::{Error, Fpga, Result, Write};
// TODO: improve
use crate::{ReceiveBuffer, SendBuffer8, SendBuffer64};

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
    fn read(&mut self, buffer: &mut ReceiveBuffer) {
        // blocking
        let p = unsafe { sys::dma_wait_512() };
        // p points to 16 u32s, where the first and the 8th are sequence numbers.
        for i in 0..7 {
            buffer[i << 2..][..4].copy_from_slice(
                unsafe { ptr::read_volatile(p.add(i + 1)) }
                    .to_le_bytes()
                    .as_slice(),
            );
            buffer[(i + 7) << 2..][..4].copy_from_slice(
                unsafe { ptr::read_volatile(p.add(i + 9)) }
                    .to_le_bytes()
                    .as_slice(),
            );
        }
    }

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
