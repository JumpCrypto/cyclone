use crate::{align::HalfAligned, Aligned, Error, Flush, ReadWrite, Result, Write};

use core::arch::x86_64::{
    // 256-bit SIMD register, requires avx
    __m256i as u256,
    _mm256_load_si256 as load_u256,
    _mm256_stream_si256 as stream_u256,
};

use cyclone_f1_sys::{
    c_void, fpga_pci_attach, fpga_pci_detach, fpga_pci_get_address, fpga_pci_peek, fpga_pci_poke,
};

#[derive(Clone)]
/// AWS F1 FPGA.
pub struct F1 {
    ctrl_bar: i32,
    ctrl_offset: u64,
    stream_bar: i32,
    stream_slice: &'static [u256],
}

impl Drop for F1 {
    fn drop(&mut self) {
        unsafe {
            fpga_pci_detach(self.ctrl_bar);
            fpga_pci_detach(self.stream_bar);
        }
    }
}

pub type Packet = Aligned<[u64; 8]>;

pub type Stream<'a, B> = crate::Stream<'a, F1, B>;

const FPGA_APP_PF: i32 = 0;
const APP_PF_BAR0: i32 = 0;
const APP_PF_BAR4: i32 = 4;
const BURST_CAPABLE: u32 = 1;

#[cfg(feature = "f1")]
impl F1 {
    pub fn new(
        slot: i32,
        ctrl_offset: usize,
        stream_offset: usize,
        stream_size: usize,
    ) -> Result<Self> {
        unsafe {
            // fpga_pci_init does not actually do anything

            let mut ctrl_bar = 0;
            if 0 != fpga_pci_attach(slot, FPGA_APP_PF, APP_PF_BAR0, 0, &mut ctrl_bar) {
                return Err(Error::SudoRequired);
            }

            let mut stream_bar = 0;
            if 0 != fpga_pci_attach(
                slot,
                FPGA_APP_PF,
                APP_PF_BAR4,
                BURST_CAPABLE,
                &mut stream_bar,
            ) {
                fpga_pci_detach(ctrl_bar);
                return Err(Error::SudoRequired);
            }

            let mut stream_addr: *mut c_void = core::ptr::null_mut();
            fpga_pci_get_address(
                stream_bar,
                stream_offset as u64,
                stream_size as u64,
                &mut stream_addr as *mut _,
            );
            let stream_slice = core::slice::from_raw_parts(stream_addr as *const u256, stream_size);

            Ok(F1 {
                ctrl_bar,
                ctrl_offset: ctrl_offset as u64,
                stream_bar,
                stream_slice,
            })
        }
    }
}

impl Flush for F1 {
    fn flush(&mut self) {
        unsafe {
            core::arch::x86_64::_mm_sfence();
        }
    }
}

impl Write<u32> for F1 {
    fn write(&mut self, index: usize, value: &u32) {
        let offset = (2 << 30) | (index << 2);
        unsafe {
            // the other order does not work.
            fpga_pci_poke(self.ctrl_bar, self.ctrl_offset + 4, *value);
            fpga_pci_poke(self.ctrl_bar, self.ctrl_offset, offset as _);
        }
    }
}

impl ReadWrite<u32> for F1 {
    fn read(&self, index: usize) -> u32 {
        let offset = (1 << 30) | (index << 2);
        let mut value = 0;
        unsafe {
            fpga_pci_poke(self.ctrl_bar, self.ctrl_offset, offset as _);
            fpga_pci_peek(self.ctrl_bar, self.ctrl_offset, &mut value);
        }
        value
    }
}

type HalfPacket = HalfAligned<[u64; 4]>;

impl Packet {
    #[inline(always)]
    fn split(&self) -> (&HalfPacket, &HalfPacket) {
        use core::mem::transmute;
        unsafe { (transmute(&self.value[4]), transmute(&self.value[0])) }
    }
}

impl Write<HalfPacket> for F1 {
    fn write(&mut self, index: usize, packet: &HalfPacket) {
        unsafe {
            let register = load_u256(&packet[0] as *const u64 as *const u256);
            stream_u256(
                &self.stream_slice[index] as *const u256 as *mut u256,
                register,
            );
        }
    }
}

impl Write<Packet> for F1 {
    fn write(&mut self, index: usize, packet: &Packet) {
        // x86 doesn't support 512bit writes, but
        // sometimes the two half-packets are combined into
        // one TLP anyway.
        let (hi, lo) = packet.split();
        self.write(2 * index, lo);
        self.write(2 * index + 1, hi);
    }
}
