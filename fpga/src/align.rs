use core::ops::{Deref, DerefMut};

#[repr(align(32))]
#[derive(Copy, Clone, Debug)]
struct Align256;

#[repr(align(64))]
#[derive(Copy, Clone, Debug)]
struct Align512;

/// Aligns entries to 64 byte (512 bit) boundaries.
pub struct Aligned<T> {
    // this 0-sized, 64-byte aligned entry aligns the entire struct
    __: [Align512; 0],
    pub(crate) value: T,
}

/// Aligns entries to 64 byte (512 bit) boundaries.
pub struct HalfAligned<T> {
    // this 0-sized, 32-byte aligned entry aligns the entire struct
    __: [Align256; 0],
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

impl<T> Deref for HalfAligned<T> {
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
