//! Preprocessed form of BLS12-377 G1 curve.

use crate::preprocess::{Parameters, PreprocessedAffine};
use ark_bls12_377::{Fq, G1Affine, G1Projective, G1TEProjective};
use ark_ec::AffineRepr as _;
use ark_ff::{Field as _, MontFp};
use ark_std::Zero as _;

pub const FQ_S: Fq = MontFp!("10189023633222963290707194929886294091415157242906428298294512798502806398782149227503530278436336312243746741931");
pub const FQ_S_INV: Fq = MontFp!("30567070899668889872121584789658882274245471728719284894883538395508419196346447682510590835309008936731240225793");
pub const FQ_SQRT_MIN_A: Fq = MontFp!("235104237478051516191809091778322087600408126435680774020954291067230236919576441851480900410358060820255406299421");

pub type G1PTEAffine = PreprocessedAffine<Parameters>;

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
        let points = crate::testing::random_points(size);
        for point in points.iter() {
            let projective: G1Projective = (*point).into();
            let affine: G1Affine = projective.into();
            assert_eq!(point, &affine);
        }
    }
}
