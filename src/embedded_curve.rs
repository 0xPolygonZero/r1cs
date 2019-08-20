use num::BigUint;
use std::borrow::Borrow;
use std::fmt::Formatter;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Shl, Sub, SubAssign};
use std::str::FromStr;
use num::bigint::ParseBigIntError;
use crate::field::{Element, Field};
use crate::field::{Bn128, Bls12_381};

/// An embedded twisted edwards curve, defined over
/// the same base field as the constraint system.
pub trait EmbeddedCurve<F: Field> {
    fn d() -> Element<F>;
    fn cofactor() -> u16;
    fn generator() -> CurvePoint<F>;
    fn order() -> BigUint;

}

/// Specification of the babyjubjub curve from
/// https://github.com/barryWhiteHat/baby_jubjub_ecc
impl<F: Field> EmbeddedCurve<F> for Bn128 {
    fn d() -> Element<F> {
        let n = BigUint::from_str(
            "0"
        ).unwrap();
        Element::from(n)
    }
    fn cofactor() -> u16 {
        0
    }
    fn generator() -> CurvePoint<F> {
        let x = BigUint::from_str(
            "1"
        ).unwrap();
        let y = BigUint::from_str(
            "1"
        ).unwrap();
        CurvePoint::from((Element::from(x), Element::from(y)))
    }
    fn order() -> BigUint {
        BigUint::from_str(
            "0"
        ).unwrap()
    }
}

/// Specification of the jubjub curve from
/// https://github.com/zkcrypto/jubjub
impl<F: Field> EmbeddedCurve<F> for Bls12_381 {
    fn d() -> Element<F> {
        let n = BigUint::from_str(
            "0"
        ).unwrap();
        Element::from(n)
    }
    fn cofactor() -> u16 {
        0
    }
    fn generator() -> CurvePoint<F> {
        let x = BigUint::from_str(
            "1"
        ).unwrap();
        let y = BigUint::from_str(
            "1"
        ).unwrap();
        CurvePoint::from((Element::from(x), Element::from(y)))
    }
    fn order() -> BigUint {
        BigUint::from_str(
            "0"
        ).unwrap()
    }
}

/// A point on an elliptic curve. Can be expressed as
/// (u,v) in Edwards form, or (x,y) in birationally equivalent
/// Montgomery form.
#[derive(Debug)]
pub struct CurvePoint<F: Field> {
    x: Element<F>,
    y: Element<F>,
}

impl<F: Field> From<(Element<F>, Element<F>)> for CurvePoint<F> {
    fn from(point: (Element<F>, Element<F>)) -> CurvePoint<F> {
        CurvePoint { x: point.0, y: point.1 }
    }
}
