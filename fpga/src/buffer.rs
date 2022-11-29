use core::ops;
use crate::ReceiveBuffer;

impl ops::Deref for ReceiveBuffer {
    type Target = [u8; 56];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for ReceiveBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for ReceiveBuffer {
    fn default() -> Self {
        Self([0u8; 56])
    }
}

