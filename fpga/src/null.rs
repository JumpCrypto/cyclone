use crate::Fpga;

#[derive(Copy, Clone)]
pub struct Null {
    __: (),
}

impl Null {
    pub fn new() -> Self {
        Self { __: () }
    }
}

impl Default for Null {
    fn default() -> Self {
        Self::new()
    }
}

impl Fpga for Null {
    fn read_register(&self, _: u32) -> u32 {
        0
    }
    fn write_register(&mut self, _: u32, _: u32) {
    }
}

