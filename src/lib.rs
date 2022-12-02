pub mod app;
pub use app::{App, Command, Fpga, Packet};

pub mod digits;
pub use digits::{limb_carries, Digit, Limb, Scalar};

pub mod preprocess;
pub use preprocess::G1PTEAffine;

pub mod timing;
pub use timing::{always_timed, timed};

use ark_bls12_377::{Fr, G1Affine, G1Projective};
use ark_ec::AffineRepr as _;

#[cfg(feature = "hw")]
pub fn fpga() -> fpga::Result<Fpga> {
    Fpga::new(0, 0x500)
}
#[cfg(not(feature = "hw"))]
pub fn fpga() -> fpga::Result<Fpga> {
    Ok(Fpga::new())
}

pub fn random_points(size: u8) -> Vec<G1Affine> {
    use rand_core::SeedableRng;
    let mut rng = rand::prelude::StdRng::from_entropy();

    use ark_std::UniformRand;
    let points: Vec<_> = timed("generating random projective points", || {
        (0..(1 << size))
            .map(|_| G1Projective::rand(&mut rng))
            .collect()
    });

    use ark_ec::CurveGroup as _;
    timed("batch converting to affine", || {
        G1Projective::normalize_batch(&points)
    })
}

pub fn not_so_random_points(size: u8, actual: u8) -> Vec<G1Affine> {
    use rand_core::SeedableRng;
    let rng = rand::prelude::StdRng::from_entropy();

    timed("generating not-so-random projective points", || {
        let random_points = random_points(actual);

        use rand::distributions::Slice;
        let slice = Slice::new(&random_points).unwrap();
        use rand::Rng;
        rng.sample_iter(&slice).take(1 << size).copied().collect()
    })
}

pub fn not_so_random_preprocessed_points(size: u8, actual: u8) -> Vec<G1PTEAffine> {
    use rand_core::SeedableRng;
    let rng = rand::prelude::StdRng::from_entropy();

    timed("generating not-so-random projective points", || {
        let random_points: Vec<_> = random_points(actual)
            .iter()
            .map(|point| point.into())
            .collect();

        use rand::distributions::Slice;
        let slice = Slice::new(&random_points).unwrap();
        use rand::Rng;
        rng.sample_iter(&slice).take(1 << size).copied().collect()
    })
}

/// Generates points of the form {P_i} = {\beta^i * g}, where g = basepoint
pub fn harness_points(size: u8) -> (Fr, Vec<G1PTEAffine>) {
    use rand_core::SeedableRng;
    let length = 1 << size;

    use ark_std::{One, UniformRand};
    let mut rng = rand::prelude::StdRng::from_entropy();

    let beta = Fr::rand(&mut rng);
    eprintln!("using beta: {}", beta);

    let scalars = timed("scalar gen", || {
        let mut scalars = Vec::with_capacity(length as _);
        scalars.push(Fr::one());
        scalars.push(beta);
        let mut last = beta;
        for _ in 2..length {
            last *= beta;
            scalars.push(last);
        }
        scalars
    });

    use ark_ec::scalar_mul::fixed_base::FixedBase;

    let points = timed("point gen", || {
        let scalar_bits = 256;
        let g = G1Affine::generator();
        let window = FixedBase::get_mul_window_size(length);
        let table = FixedBase::get_window_table::<G1Projective>(scalar_bits, window, g.into());
        FixedBase::msm::<G1Projective>(scalar_bits, window, &table, &scalars)
    });

    use ark_ec::CurveGroup as _;
    let points = timed("Projective -> Affine", || {
        G1Projective::normalize_batch(&points)
    });

    // the slow part
    (
        beta,
        timed("Affine -> PTEAffine", || preprocess_points(&points)),
    )
}

pub fn preprocess_points(points: &[G1Affine]) -> Vec<G1PTEAffine> {
    let mut ppoints = vec![G1PTEAffine::zero(); points.len()];

    const CHUNK: usize = 1 << 16;
    for (chunk_in, chunk_out) in points
        .chunks(CHUNK)
        .zip(ppoints.as_mut_slice().chunks_mut(CHUNK))
    {
        crate::preprocess::batch_preprocess(chunk_in, chunk_out);
    }
    ppoints
}

pub fn load_beta(name: &str) -> Fr {
    let beta_name = format!("{}.beta", name);
    let mut beta = Fr::default();
    load(&mut beta, &beta_name);
    beta
}

pub fn load_points(size: u8, name: &str) -> Vec<G1PTEAffine> {
    let points_name = format!("{}.points", name);
    let mut points = always_timed("allocating points", || vec![G1PTEAffine::zero(); 1 << size]);
    always_timed("loading points", || load_slice(&mut points, &points_name));
    points
}

pub fn digits_to_scalars(digits: &[Digit]) -> Vec<Fr> {
    digits
        .iter()
        .copied()
        .map(|digit| {
            if digit >= 0 {
                Fr::from(digit as u16)
            } else {
                -Fr::from((-(digit as i32)) as u16)
            }
        })
        .collect()
}

/// "Fast" calculation of MSM in SW via MSM of the scalars.
pub fn noconflict_harness_digits(beta: &Fr, size: usize) -> (Vec<i16>, G1Projective) {
    use ark_std::{One, Zero};

    // use our_ec::AffineCurve;
    let g = G1Affine::generator();

    let digits = noconflict_column16(size as u8);
    let scalars = digits_to_scalars(&digits);

    // calculate expected result "in the exponent"
    let result = timed("SW MSM via betas", || {
        let mut beta_i = Fr::one();
        let mut prod = Fr::zero();
        for &scalar in &scalars {
            prod += scalar * beta_i;
            beta_i *= beta;
        }
        g * prod
    });

    (digits, result)
}

pub fn random_fr(size: u8) -> Vec<Fr> {
    use ark_std::UniformRand;
    use rand_core::SeedableRng;
    let mut rng = rand::prelude::StdRng::from_entropy();

    (0..(1 << size)).map(|_| Fr::rand(&mut rng)).collect()
}

pub fn harness_scalars(beta: &Fr, size: u8) -> (Vec<Scalar>, G1Projective) {
    use ark_std::{One, Zero};

    use ark_ff::PrimeField;
    let g = G1Affine::generator();

    let scalars = random_fr(size as u8);

    // calculate expected result "in the exponent"
    // \Sum_i (scalar_i * beta^i)
    let result = timed("SW MSM via betas", || {
        let mut beta_i = Fr::one();
        let mut prod = Fr::zero();
        for &scalar in &scalars {
            prod += scalar * beta_i;
            beta_i *= beta;
        }
        g * prod
    });

    let scalars: Vec<Scalar> = scalars
        .iter()
        .map(|scalar| scalar.into_bigint().0)
        .collect();

    (scalars, result)
}

pub fn harness_digits(beta: &Fr, size: u8) -> (Vec<i16>, G1Projective) {
    use ark_std::{One, Zero};

    // use our_ec::AffineCurve;
    let g = G1Affine::generator();

    let digits = random_digits(size as u8);
    let scalars = digits_to_scalars(&digits);

    // calculate expected result "in the exponent"
    // \Sum_i (scalar_i * beta^i)
    let result = timed("SW MSM via betas", || {
        let mut beta_i = Fr::one();
        let mut prod = Fr::zero();
        for &scalar in &scalars {
            prod += scalar * beta_i;
            beta_i *= beta;
        }
        g * prod
    });

    (digits, result)
}

pub fn random_digits(size: u8) -> Vec<Digit> {
    use rand_core::{RngCore, SeedableRng};
    let mut rng = rand::prelude::StdRng::from_entropy();

    (0..(1 << size)).map(|_| rng.next_u32() as i16).collect()
}

pub fn noconflict_column16(size: u8) -> Vec<i16> {
    (0..(1usize << size)).map(|i| (i % 1024) as i16).collect()
}

pub fn random_scalars(size: u8) -> Vec<Scalar> {
    use rand_core::{RngCore, SeedableRng};
    let mut rng = rand::prelude::StdRng::from_entropy();

    (0..(1 << size))
        .map(|_| {
            let mut scalar = Scalar::default();
            for limb in scalar.iter_mut() {
                *limb = rng.next_u64();
            }
            scalar
        })
        .collect()
}

pub fn zero_scalars(size: u8) -> Vec<Scalar> {
    (0..(1 << size))
        .map(|_| {
            let mut scalar = Scalar::default();
            for limb in scalar.iter_mut() {
                *limb = 0;
            }
            scalar
        })
        .collect()
}

pub fn noconflict_scalars(size: u8) -> Vec<Scalar> {
    (0..(1 << size))
        .map(|i| {
            let mut scalar = Scalar::default();
            for limb in scalar.iter_mut() {
                let i = (i as u16 % 1024) as u64;
                *limb = (i << 48) | (i << 32) | (i << 16) | i
            }
            scalar
        })
        .collect()
}

pub fn store_slice<T: Sized>(slice: &[T], name: &str) {
    use std::io::Write as _;
    let slice_data_size = std::mem::size_of::<T>() * slice.len();
    std::fs::File::create(name)
        .unwrap()
        .write_all(unsafe {
            std::slice::from_raw_parts(slice.as_ptr() as *const u8, slice_data_size)
        })
        .unwrap();
    println!("store {}B to {}", slice_data_size, name);
}

pub fn load_slice<T: Sized>(slice: &mut [T], name: &str) {
    use std::io::Read as _;
    let slice_data_size = std::mem::size_of::<T>() * slice.len();
    std::fs::File::open(name)
        .unwrap()
        .read_exact(unsafe {
            std::slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut u8, slice_data_size)
        })
        .unwrap();
    println!("load {}B from {}", slice_data_size, name);
}
pub fn store<T: Sized>(data: &T, name: &str) {
    use std::io::Write as _;
    let size = std::mem::size_of::<T>();
    std::fs::File::create(name)
        .unwrap()
        .write_all(unsafe { std::slice::from_raw_parts(data as *const T as *const u8, size) })
        .unwrap();
    println!("store {}B to {}", size, name);
}

pub fn load<T: Sized>(data: &mut T, name: &str) {
    use std::io::Read as _;
    let size = std::mem::size_of::<T>();
    std::fs::File::open(name)
        .unwrap()
        .read_exact(unsafe { std::slice::from_raw_parts_mut(data as *mut T as *mut u8, size) })
        .unwrap();
    println!("load {}B from {}", size, name);
}
