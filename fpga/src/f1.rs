use crate::{Aligned, Error, Flush, ReadWrite, Result, Write};

pub use cyclone_fpga_sys as sys;

#[derive(Copy, Clone)]
/// AWS F1 FPGA.
pub struct F1(());

pub type Packet = Aligned<[u64; 8]>;

pub type Stream<'a, B> = crate::Stream<'a, Packet, F1, B>;

#[cfg(feature = "f1")]
impl F1 {
    pub fn new(slot: i32, offset: i32) -> Result<Self> {
        (unsafe { sys::init_f1(slot, offset) } == 0)
            .then_some(Self(()))
            .ok_or(Error::SudoRequired)
    }
}

impl Flush for F1 {
    fn flush(&mut self) {
        unsafe { sys::write_flush() };
    }
}

impl Write<u32> for F1 {
    type Index = u32;

    fn write(&mut self, index: u32, value: &u32) {
        let offset = index << 2;
        unsafe { sys::write_32_f1(offset, *value) };
    }
}

impl ReadWrite<u32> for F1 {
    fn read(&self, index: u32) -> u32 {
        let offset = index << 2;
        unsafe { sys::read_32_f1(offset) }
    }
}

impl Write<Packet> for F1 {
    type Index = usize;

    fn write(&mut self, index: usize, packet: &Packet) {
        let offset = index << 6;
        let slice: &[u64] = &**packet;
        unsafe { sys::write_512_f1(offset as u64, &slice[0] as *const _ as _) };
    }
}
