//! Host-side application to use the FPGA-side application.
//!
//! Cyclone MSM currently only supports the G1 curve of BLS12-377.
//! MSM instances of size up to 27 are supported.
//!
//! Steps:
//! - preprocess points, stream to FPGA
//! - precompute scalars, stream to FPGA column-wise
//! - gather and aggregate column sums
//!
//! The idea is that in practical use, the points are fixed, whereas the scalars vary per instance.
//! Therefore, the lengthy preprocessing of points is amortized, whereas the precomputation needs
//! to be done efficiently for each instance.

pub mod app;
pub use app::Fpga;

pub mod bls12_377;

pub mod io;

pub mod precompute;

pub mod preprocess;

pub mod testing;

pub mod timing;

use ark_bls12_377::{Fr, G1Projective};

/// Host-side Cyclone MSM application.
pub struct App {
    pub fpga: Fpga,
    len: usize,
    pool: Option<rayon::ThreadPool>,
    carried: Option<Vec<Scalar>>,
}

#[repr(u64)]
/// Commands for MSM column processing.
pub enum Command {
    StartColumn = 1,
    SetDigit = 3,
}

/// Packet of 8 commands, streamed to FPGA during MSM column processing.
pub type Packet = fpga::Aligned<[u64; 8]>;

/// Signed 16-bit digit.
pub type Digit = i16;
/// Unsigned 64-bit limb of a scalar
pub type Limb = u64;
/// 256-bit scalar composed of four limbs, least-significant limb first
pub type Scalar = [Limb; 4];

/// FPGA constructor, independent of "hw" feature.
#[cfg(feature = "hw")]
pub fn fpga() -> fpga::Result<Fpga> {
    Fpga::new(0, 0x500, 0, 0x1_0000_0000)
}
#[cfg(not(feature = "hw"))]
pub fn fpga() -> fpga::Result<Fpga> {
    Ok(Fpga::new())
}
