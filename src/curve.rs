use num::BigUint;
use std::borrow::Borrow;
use std::fmt::Formatter;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Shl, Sub, SubAssign};
use std::str::FromStr;
use num::bigint::ParseBigIntError;
use num::pow;
use crate::field::{Element, Field};
use crate::embedded_curve::{EmbeddedCurve};
use std::marker::PhantomData;

pub trait Curve<F: Field> {
}

/// An embedded Edwards curve defined over the same base field
/// as the field used in the constraint system
pub struct EdwardsCurve<F: Field> {
    a: Element<F>,
    d: Element<F>,
}

impl<F: Field> EdwardsCurve<F> {
    fn contains_point(self, p: &EdwardsPoint<F>) -> bool {
        let ref x_squared = pow(p.x.clone(), 2);
        let ref y_squared = pow(p.y.clone(), 2);
        self.a * x_squared + y_squared == Element::<F>::one() + self.d * x_squared * y_squared
    }
}

impl<F: Field> From<(Element<F>, Element<F>)> for EdwardsCurve<F> {
    fn from(params: (Element<F>, Element<F>)) -> EdwardsCurve<F> {
        EdwardsCurve {
            a: params.0,
            d: params.1
        }
    }
}

/// A point on an embedded Edwards curve in affine coordinates
pub struct EdwardsPoint<F: Field> {
    x: Element<F>,
    y: Element<F>,
}

impl<F: Field> From<(Element<F>, Element<F>)> for EdwardsPoint<F>
    where EdwardsCurve<F>: EmbeddedCurve<F> {
    fn from(coordinates: (Element<F>, Element<F>)) -> EdwardsPoint<F> {
        let c = EdwardsCurve::<F>::from(EdwardsCurve::<F>::parameters());
        let p = EdwardsPoint {
            x: coordinates.0,
            y: coordinates.1
        };
        assert!(c.contains_point(&p), "Point is not on the curve.");
        p
    }
}

impl<F: Field> EdwardsPoint<F> {
    fn is_identity(&self) -> bool {
        self.x == Element::<F>::zero() && self.y == Element::<F>::one()
    }

    fn identity() -> Self {
        EdwardsPoint{
            x: Element::<F>::zero(),
            y: Element::<F>::one(),
        }
    }
}

impl<F: Field> Neg for EdwardsPoint<F> where EdwardsCurve<F>: EmbeddedCurve<F> {
    type Output = EdwardsPoint<F>;

    fn neg(self) -> EdwardsPoint<F> {
        -&self
    }
}

/// Negates an edwards curve point by negating the x coordinate
impl<F: Field> Neg for &EdwardsPoint<F> where EdwardsCurve<F>: EmbeddedCurve<F> {
    type Output = EdwardsPoint<F>;

    fn neg(self) -> EdwardsPoint<F> {
        if self.is_identity() {
            EdwardsPoint::identity()
        } else {
            EdwardsPoint::from((-self.x.clone(), self.y.clone()))
        }
    }
}

impl<F: Field> Add<EdwardsPoint<F>> for EdwardsPoint<F>
    where EdwardsCurve<F>: EmbeddedCurve<F> {
    type Output = EdwardsPoint<F>;

    fn add(self, rhs: EdwardsPoint<F>) -> EdwardsPoint<F> {
        &self + &rhs
    }
}

impl<F: Field> Add<&EdwardsPoint<F>> for EdwardsPoint<F>
    where EdwardsCurve<F>: EmbeddedCurve<F> {
    type Output = EdwardsPoint<F>;

    fn add(self, rhs: &EdwardsPoint<F>) -> EdwardsPoint<F> {
        &self + rhs
    }
}

impl<F: Field> Add<EdwardsPoint<F>> for &EdwardsPoint<F>
    where EdwardsCurve<F>: EmbeddedCurve<F> {
    type Output = EdwardsPoint<F>;

    fn add(self, rhs: EdwardsPoint<F>) -> EdwardsPoint<F> {
        self + &rhs
    }
}

/// Uses the non-optimized algorithm for Edwards curve addition in affine coordinates.
impl<F: Field> Add<&EdwardsPoint<F>> for &EdwardsPoint<F>
    where EdwardsCurve<F>: EmbeddedCurve<F> {
    type Output = EdwardsPoint<F>;

    fn add(self, rhs: &EdwardsPoint<F>) -> EdwardsPoint<F> {
        let x1y1 = &self.x * &self.y;
        let x2y2 = &rhs.x * &rhs.y;
        let x1x2 = &self.x * &rhs.x;
        let y1y2 = &self.y * &rhs.y;
        let x1y2 = &self.x * &rhs.y;
        let y1x2 = &self.y * &rhs.x;

        let x = (&x1y1 + &x2y2) / (x1x2 + y1y2);
        let y = (&x1y1 - &x2y2) / (x1y2 - y1x2);

        EdwardsPoint::from((x, y))
    }
}

impl<F: Field> AddAssign for EdwardsPoint<F> where EdwardsCurve<F>: EmbeddedCurve<F> {
    fn add_assign(&mut self, rhs: EdwardsPoint<F>) {
        *self += &rhs;
    }
}

impl<F: Field> AddAssign<&EdwardsPoint<F>> for EdwardsPoint<F>
    where EdwardsCurve<F>: EmbeddedCurve<F> {
    fn add_assign(&mut self, rhs: &EdwardsPoint<F>) {
        *self = &*self + rhs;
    }
}

impl<F: Field> PartialEq for EdwardsPoint<F> {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl<F: Field> Eq for EdwardsPoint<F> {}

#[cfg(test)]
mod tests {
    use std::iter;
    use std::str::FromStr;

    use itertools::assert_equal;

    use crate::field::{Bn128, Bls12_381, Element, Field};
    use crate::curve::{EdwardsCurve, EdwardsPoint};
    use crate::embedded_curve::{EmbeddedCurve};
    use num::BigUint;

    #[test]
    fn check_point_on_curve() {
        type F = Bls12_381;

        let curve = EdwardsCurve{
            a: EdwardsCurve::<F>::parameters().0,
            d: EdwardsCurve::<F>::parameters().1
        };
        let x = Element::from(BigUint::from_str(
            "11076627216317271660298050606127911965867021807910416450833192264015104452986"
        ).unwrap());
        let y = Element::from(BigUint::from_str(
            "44412834903739585386157632289020980010620626017712148233229312325549216099227"
        ).unwrap());
        let point = EdwardsPoint::from((x ,y));
        assert!(curve.contains_point(&point))
    }

    fn check_point_not_on_curve() {
        type F = Bls12_381;

        let curve = EdwardsCurve{
            a: EdwardsCurve::<F>::parameters().0,
            d: EdwardsCurve::<F>::parameters().1
        };
        let x = Element::from(BigUint::from_str(
            "11076627216317271660298050606127911965867021807910416450833192264015104452985"
        ).unwrap());
        let y = Element::from(BigUint::from_str(
            "44412834903739585386157632289020980010620626017712148233229312325549216099227"
        ).unwrap());
        let point = EdwardsPoint::from((x ,y));
        assert!(!curve.contains_point(&point))
    }

    fn check_add_and_negate() {
        type F = Bls12_381;

        let curve = EdwardsCurve{
            a: EdwardsCurve::<F>::parameters().0,
            d: EdwardsCurve::<F>::parameters().1
        };
        let x = Element::<F>::from(BigUint::from_str(
            "11076627216317271660298050606127911965867021807910416450833192264015104452985"
        ).unwrap());
        let y = Element::<F>::from(BigUint::from_str(
            "44412834903739585386157632289020980010620626017712148233229312325549216099227"
        ).unwrap());
        let point = EdwardsPoint::from((x ,y));
        let inverse = -&point;
        assert!(point + inverse == EdwardsPoint::identity())
    }
}
