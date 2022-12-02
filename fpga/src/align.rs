use core::ops::{Deref, DerefMut};

#[repr(align(64))]
#[derive(Copy, Clone, Debug)]
struct Aligner;

/// Aligns entries to 64 byte (512 bit) boundaries.
pub struct Aligned<T> {
    // this 0-sized, 64-byte aligned entry aligns the entire struct
    __: [Aligner; 0],
    value: T,
}

impl<T: Clone> Clone for Aligned<T> {
    fn clone(&self) -> Self {
        Self {
            __: [],
            value: self.value.clone(),
        }
    }
}
impl<T: Copy> Copy for Aligned<T> {}

impl<T> Deref for Aligned<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> DerefMut for Aligned<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

/// Align a value
pub const fn align<T>(value: T) -> Aligned<T> {
    Aligned { __: [], value }
}

impl Default for Aligned<[u64; 8]> {
    fn default() -> Self {
        align([0u64; 8])
    }
}
