use core::ops::{AddAssign, Neg, SubAssign};

use ark_bls12_377::{g1::Parameters, Fq, G1Affine, G1Projective, G1TEProjective};
use ark_ec::{
    models::twisted_edwards::{Affine, Projective, TECurveConfig},
    AffineRepr as _,
};
use ark_ff::{Field as _, MontFp};
use ark_std::{One as _, Zero as _};
use derivative::Derivative;

pub const FQ_S: Fq = MontFp!("10189023633222963290707194929886294091415157242906428298294512798502806398782149227503530278436336312243746741931");
pub const FQ_S_INV: Fq = MontFp!("30567070899668889872121584789658882274245471728719284894883538395508419196346447682510590835309008936731240225793");
pub const FQ_SQRT_MIN_A: Fq = MontFp!("235104237478051516191809091778322087600408126435680774020954291067230236919576441851480900410358060820255406299421");

pub type G1PTEAffine = PreprocessedAffine<Parameters>;

/// Affine coordinates for a point on a twisted Edwards curve, over the
/// base field `P::BaseField`.
///
/// Includes the coordinate `2D*x*y`.
#[derive(Derivative)]
#[derivative(Copy(bound = "P: TECurveConfig"), Clone(bound = "P: TECurveConfig"))]
// #[must_use]
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct PreprocessedAffine<P: TECurveConfig> {
    /// X coordinate of the point represented as a field element
    pub x: P::BaseField,
    /// Y coordinate of the point represented as a field element
    pub y: P::BaseField,
    /// Precomputed product k*X*Y, k=2D - formulas for A=-1
    pub kt: P::BaseField,
}

impl<P: TECurveConfig> PreprocessedAffine<P> {
    pub fn new(x: P::BaseField, y: P::BaseField) -> Self {
        Self {
            x,
            y,
            kt: (P::COEFF_D + P::COEFF_D) * x * y,
        }
    }

    pub const fn zero() -> Self {
        Self {
            x: P::BaseField::ZERO,
            y: P::BaseField::ONE,
            kt: P::BaseField::ZERO,
        }
    }

    pub fn is_zero(&self) -> bool {
        self.x.is_zero() && self.y.is_one()
    }
}

impl<P: TECurveConfig> From<Affine<P>> for PreprocessedAffine<P> {
    fn from(affine: Affine<P>) -> PreprocessedAffine<P> {
        Self::new(affine.x, affine.y)
    }
}

impl<P: TECurveConfig> From<&Affine<P>> for PreprocessedAffine<P> {
    fn from(affine: &Affine<P>) -> PreprocessedAffine<P> {
        Self::new(affine.x, affine.y)
    }
}

impl<P: TECurveConfig> From<PreprocessedAffine<P>> for Affine<P> {
    fn from(pre: PreprocessedAffine<P>) -> Affine<P> {
        Self::new(pre.x, pre.y)
    }
}

impl<P: TECurveConfig> From<&PreprocessedAffine<P>> for Affine<P> {
    fn from(pre: &PreprocessedAffine<P>) -> Affine<P> {
        Self::new(pre.x, pre.y)
    }
}

impl<'a, P: TECurveConfig> AddAssign<&'a PreprocessedAffine<P>> for Projective<P> {
    fn add_assign(&mut self, other: &PreprocessedAffine<P>) {
        if self.is_zero() {
            // not checking this results is 7M vs 1M
            self.x = other.x;
            self.y = other.y;
            self.t = other.x * other.y;
            self.z = P::BaseField::one();
            return;
        }

        /* 8M, we can do 7M
        // See "Twisted Edwards Curves Revisited"
        // Huseyin Hisil, Kenneth Koon-Ho Wong, Gary Carter, and Ed Dawson
        // 3.1 Unified Addition in E^e
        // Source: https://www.hyperelliptic.org/EFD/g1p/data/twisted/extended/addition/madd-2008-hwcd
        // A = X1*X2
        let a = self.x * &other.x;
        // B = Y1*Y2
        let b = self.y * &other.y;
        // C = T1*d*T2
        let c = self.t * &other.kt;

        // D = Z1
        let d = self.z;
        // E = (X1+Y1)*(X2+Y2)-A-B
        let e = (self.x + &self.y) * &(other.x + &other.y) - &a - &b;
        // F = D-C
        let f = d - &c;
        // G = D+C
        let g = d + &c;
        // H = B-a*A
        // let h = b - &P::mul_by_a(&a);
        let h = b + &a;
        // X3 = E*F
        self.x = e * &f;
        // Y3 = G*H
        self.y = g * &h;
        // T3 = E*H
        self.t = e * &h;
        // Z3 = F*G
        self.z = f * &g;
        return;
        */

        let r1 = self.y - self.x;
        let r2 = other.y - other.x;
        let r3 = self.y + self.x;
        let r4 = other.y + other.x;

        // step 2: 1M (if parallel)
        let r5 = r1 * r2;
        let r6 = r3 * r4;
        let r7 = self.t * other.kt;
        let r8 = self.z.double();

        // step 3: 1D
        // R7 = P::COEFF_D * R7;
        // R8 = R8.double();

        // step 4
        let r1b = r6 - r5;
        let r2b = r8 - r7;
        let r3b = r8 + r7;
        let r4b = r6 + r5;

        // step 5
        self.x = r1b * r2b;
        self.y = r3b * r4b;
        self.t = r1b * r4b;
        self.z = r2b * r3b;
    }
}

impl<'a, P: TECurveConfig> SubAssign<&'a PreprocessedAffine<P>> for Projective<P> {
    fn sub_assign(&mut self, other: &'a PreprocessedAffine<P>) {
        self.add_assign(&(-*other))
    }
}

impl<P: TECurveConfig> Neg for PreprocessedAffine<P> {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: self.y,
            kt: -self.kt,
        }
    }
}

impl From<&G1Affine> for G1PTEAffine {
    /// convert affine Weierstrass to affine extended Twisted Edwards
    fn from(point: &G1Affine) -> G1PTEAffine {
        if point.is_zero() {
            return G1PTEAffine::new(Fq::ZERO, Fq::ONE);
        }

        let xpo = point.x + Fq::ONE;
        let sxpo = xpo * FQ_S;
        let axpo = xpo * FQ_SQRT_MIN_A;
        let syxpo = sxpo * point.y;

        let x = (sxpo + Fq::ONE) * axpo;
        let y = syxpo - point.y;
        let z = syxpo + point.y;
        let z_inv = Fq::ONE / z;

        G1PTEAffine::new(x * z_inv, y * z_inv)
    }
}

/// convert affine Weierstrass to affine extended Twisted Edwards in batch
pub fn batch_preprocess(a: &[G1Affine], b: &mut [G1PTEAffine]) {
    debug_assert!(a.len() == b.len());

    let mut x = vec![Fq::ZERO; a.len()];
    let mut y = vec![Fq::ZERO; a.len()];
    let mut z = vec![Fq::ZERO; a.len()];

    for (i, p) in a.iter().enumerate() {
        if p.is_zero() {
            x[i] = Fq::ZERO;
            y[i] = Fq::ONE;
            z[i] = Fq::ONE;
            continue;
        }

        let xpo = p.x + Fq::ONE;
        let sxpo = xpo * FQ_S;
        let axpo = xpo * FQ_SQRT_MIN_A;
        let syxpo = sxpo * p.y;

        x[i] = (sxpo + Fq::ONE) * axpo;
        y[i] = syxpo - p.y;
        z[i] = syxpo + p.y;
    }

    ark_ff::batch_inversion(&mut z);

    for i in 0..a.len() {
        b[i] = G1PTEAffine::new(x[i] * z[i], y[i] * z[i]);
    }
}

// pub fn into_twisted(p: &G1Affine) -> G1TEAffine {
//     if p.is_zero() {
//         return G1TEAffine::new(Fq::ZERO, Fq::ONE);
//     }

//     let xpo = p.x + Fq::ONE;
//     let sxpo = xpo * FQ_S;
//     let axpo = xpo * FQ_SQRT_MIN_A;
//     let syxpo = sxpo * p.y;

//     let x = (sxpo + Fq::ONE) * axpo;
//     let y = syxpo - p.y;
//     let z = syxpo + p.y;
//     let z_inv = Fq::ONE / z;

//     G1TEAffine::new(x * z_inv, y * z_inv)
// }

// impl Into<G1Projective> for &G1TEProjective {
//     fn into(self) -> G1Projective {
//         into_weierstrass(self)
//     }
// }

/// convert projective extended Twisted Edwards to projective Weierstrass
pub fn into_weierstrass(point: &G1TEProjective) -> G1Projective {
    if point.is_zero() {
        return G1Projective::zero();
    }

    let z_inv = Fq::ONE / point.z;
    // let check = p.x / point.z;
    let aff_x = point.x * z_inv;
    let aff_y = point.y * z_inv;

    let p = Fq::ONE + aff_y;
    let m = Fq::ONE - aff_y;
    let u = p / m;
    let v = u / aff_x;

    let x = (u * FQ_S_INV) - Fq::ONE;
    let y = v * FQ_S_INV * FQ_SQRT_MIN_A;

    G1Projective::new(x, y, Fq::ONE)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn conversions() {
        let size = 3;
        let points = crate::random_points(size);
        for point in points.iter() {
            let projective: G1Projective = (*point).into();
            let affine: G1Affine = projective.into();
            assert_eq!(point, &affine);
        }
    }
}
