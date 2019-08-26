use num::BigUint;
use std::borrow::Borrow;
use std::fmt::Formatter;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Shl, Sub, SubAssign};
use std::str::FromStr;
use num::bigint::ParseBigIntError;
use crate::field::{Element, Field};
use crate::field::{Bn128, Bls12_381};
use crate::curve::{EdwardsCurve, Curve, EdwardsPoint};

/// Families of "embedded" curves, defined over
/// the same base field as the constraint system.
pub trait EmbeddedCurve<F: Field> {
    fn parameters() -> (Element<F>, Element<F>);
}

/// Specification of the babyjubjub curve from
/// https://github.com/barryWhiteHat/baby_jubjub_ecc
impl EmbeddedCurve<Bn128> for EdwardsCurve<Bn128> {
    fn parameters() -> (Element<Bn128>, Element<Bn128>) {
        let a = Element::from(BigUint::from_str("168700").unwrap());
        let d = Element::from(BigUint::from_str("168696").unwrap());
        (a, d)
    }
}

/// Specification of the jubjub curve from
/// https://github.com/zkcrypto/jubjub
impl EmbeddedCurve<Bls12_381> for EdwardsCurve<Bls12_381> {
    fn parameters() -> (Element<Bls12_381>, Element<Bls12_381>) {
        let a = -Element::<Bls12_381>::one();
        let d = Element::from(BigUint::from_str(
            "19257038036680949359750312669786877991949435402254120286184196891950884077233"
        ).unwrap());
        (a, d)
    }
}

#[cfg(test)]
mod tests {
    use std::iter;
    use std::str::FromStr;

    use itertools::assert_equal;

    use crate::field::{Bn128, Bls12_381, Element};

    #[test]
    fn addition() {
        type F = Bls12_381;
    }
}
