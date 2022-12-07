//! Generate test instances.

use crate::{
    bls12_377::G1PTEAffine, preprocess::preprocess_points, timing::timed, Digit, Fr, Scalar,
};
use ark_bls12_377::{G1Affine, G1Projective};
use ark_ec::AffineRepr as _;

pub fn random_digits(size: u8) -> Vec<Digit> {
    use rand_core::{RngCore, SeedableRng};
    let mut rng = rand::prelude::StdRng::from_entropy();

    (0..(1 << size)).map(|_| rng.next_u32() as i16).collect()
}

pub fn random_fr(size: u8) -> Vec<Fr> {
    use ark_std::UniformRand;
    use rand_core::SeedableRng;
    let mut rng = rand::prelude::StdRng::from_entropy();

    (0..(1 << size)).map(|_| Fr::rand(&mut rng)).collect()
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

pub fn harness_fr(beta: &Fr, size: u8) -> (Vec<Fr>, G1Projective) {
    use ark_std::{One, Zero};

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

    (scalars, result)
}

pub fn harness_bigints(beta: &Fr, size: u8) -> (Vec<ark_ff::BigInt<4>>, G1Projective) {
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

    let scalars: Vec<_> = scalars.iter().map(|scalar| scalar.into_bigint()).collect();

    (scalars, result)
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

pub fn noconflict_digits(size: u8) -> Vec<i16> {
    (0..(1usize << size)).map(|i| (i % 1024) as i16).collect()
}

/// "Fast" calculation of MSM in SW via MSM of the scalars.
pub fn harness_digits_conflictfree(beta: &Fr, size: usize) -> (Vec<i16>, G1Projective) {
    use ark_std::{One, Zero};

    // use our_ec::AffineCurve;
    let g = G1Affine::generator();

    let digits = noconflict_digits(size as u8);
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
