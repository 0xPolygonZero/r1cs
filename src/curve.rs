use std::borrow::Borrow;
use std::fmt::Formatter;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Shl, Sub, SubAssign};
use std::str::FromStr;

use num::bigint::ParseBigIntError;
use num::BigUint;
use num::pow;

use crate::{Expression, GadgetBuilder};
use crate::embedded_curve::EmbeddedCurve;
use crate::field::{Element, Field};

pub trait Curve<F: Field> {}

pub trait CurvePoint<F: Field, C: Curve<F>> {}

/// An embedded Edwards curve defined over the same base field
/// as the field used in the constraint system
pub trait EdwardsCurve<F: Field> {
    fn a() -> Element<F>;
    fn d() -> Element<F>;
}

/// An embedded Edwards curve point defined over the same base field
/// as the field used in the constraint system, with affine coordinates as
/// expressions.
pub struct EdwardsPointExpression<F: Field, C: EdwardsCurve<F>> {
    x: Expression<F>,
    y: Expression<F>,
    phantom: PhantomData<*const C>,
}

/// An embedded Montgomery curve point defined over the same base field
/// as the field used in the constraint system, with affine coordinates as
/// expressions.
pub struct MontgomeryPointExpression<F: Field> {
    x: Expression<F>,
    y: Expression<F>,
}

/// An embedded Weierstrass curve point defined over the same base field
/// as the field used in the constraint system, with affine coordinates as
/// expressions.
pub struct WeierstrassPointExpression<F: Field> {
    x: Expression<F>,
    y: Expression<F>,
}

/// An embedded Weierstrass curve point defined over the same base field
/// as the field used in the constraint system, with projective coordinates
/// as expressions.
pub struct ProjWeierstrassPointExpression<F: Field> {
    x: Expression<F>,
    y: Expression<F>,
}

impl<F: Field, C: EdwardsCurve<F>> EdwardsPointExpression<F, C> {
    /// EdwardsPointExpression::add(builder, e1, e2)
    pub fn add(
        builder: &mut GadgetBuilder<F>,
        point_1: &EdwardsPointExpression<F, C>,
        point_2: &EdwardsPointExpression<F, C>,
    ) -> EdwardsPointExpression<F, C> {
        let d = C::d();
        let EdwardsPointExpression { x: x1, y: y1, phantom } = point_1;
        let EdwardsPointExpression { x: x2, y: y2, phantom } = point_2;
        let x1y2 = builder.product(&x1, &y2);
        let x2y1 = builder.product(&y1, &x2);
        let x1y2x2y1 = builder.product(&x1y2, &x2y1);
        let x1x2 = builder.product(&x1, &x2);
        let y1y2 = builder.product(&y1, &y2);
        let x3 = builder.quotient(&(x1y2 + x2y1), &(&x1y2x2y1 * &d + Expression::one()));
        let y3 = builder.quotient(&(y1y2 - x1x2), &(&x1y2x2y1 * -&d + Expression::one()));
        EdwardsPointExpression::new_unsafe(x3, y3)
    }

    pub fn from_elements(x: Element<F>, y: Element<F>) -> EdwardsPointExpression<F, C> {
        assert!(C::a() * &x * &x + &y * &y == Element::one() + C::d() * &x * &x * &y * &y,
                "Point must be contained on the curve.");
        EdwardsPointExpression::new_unsafe(Expression::from(x), Expression::from(y))
    }

    pub fn from_expressions(builder: &mut GadgetBuilder<F>, x: Expression<F>, y: Expression<F>) -> EdwardsPointExpression<F, C> {
        let x_squared = builder.product(&x, &x);
        let y_squared = builder.product(&y, &y);
        let x_squared_y_squared = builder.product(&x_squared, &y_squared);
        builder.assert_equal(&(&x_squared * C::a() + &y_squared),
                             &(&x_squared_y_squared * C::d() + Expression::one()));
        EdwardsPointExpression::new_unsafe(x, y)
    }

    pub fn new_unsafe(x: Expression<F>, y: Expression<F>) -> EdwardsPointExpression<F, C> {
        EdwardsPointExpression { x, y, phantom: PhantomData }
    }
}

/*
impl<F: Field> EdwardsCurve<F> {
    fn contains_point(self, p: &EdwardsPoint<F>) -> bool {
        let ref x_squared = &p.x * &p.x;
        let ref y_squared = &p.y * &p.y;
        self.a * x_squared + y_squared == Element::<F>::one() + self.d * x_squared * y_squared
    }
}

impl<F: Field> From<(Element<F>, Element<F>)> for EdwardsCurve<F> {
    fn from(params: (Element<F>, Element<F>)) -> EdwardsCurve<F> {
        EdwardsCurve {
            a: params.0,
            d: params.1,
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
            y: coordinates.1,
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
        EdwardsPoint {
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
    use num::BigUint;

    use crate::curve::{EdwardsCurve, EdwardsPoint};
    use crate::embedded_curve::EmbeddedCurve;
    use crate::field::{Bls12_381, Bn128, Element, Field};

    #[test]
    fn check_point_on_curve() {
        type F = Bls12_381;

        let curve = EdwardsCurve {
            a: EdwardsCurve::<F>::parameters().0,
            d: EdwardsCurve::<F>::parameters().1,
        };
        let x = Element::from(BigUint::from_str(
            "11076627216317271660298050606127911965867021807910416450833192264015104452986"
        ).unwrap());
        let y = Element::from(BigUint::from_str(
            "44412834903739585386157632289020980010620626017712148233229312325549216099227"
        ).unwrap());
        let point = EdwardsPoint::from((x, y));
        assert!(curve.contains_point(&point))
    }

    fn check_point_not_on_curve() {
        type F = Bls12_381;

        let curve = EdwardsCurve {
            a: EdwardsCurve::<F>::parameters().0,
            d: EdwardsCurve::<F>::parameters().1,
        };
        let x = Element::from(BigUint::from_str(
            "11076627216317271660298050606127911965867021807910416450833192264015104452985"
        ).unwrap());
        let y = Element::from(BigUint::from_str(
            "44412834903739585386157632289020980010620626017712148233229312325549216099227"
        ).unwrap());
        let point = EdwardsPoint::from((x, y));
        assert!(!curve.contains_point(&point))
    }

    fn check_add_and_negate() {
        type F = Bls12_381;

        let curve = EdwardsCurve {
            a: EdwardsCurve::<F>::parameters().0,
            d: EdwardsCurve::<F>::parameters().1,
        };
        let x = Element::<F>::from(BigUint::from_str(
            "11076627216317271660298050606127911965867021807910416450833192264015104452985"
        ).unwrap());
        let y = Element::<F>::from(BigUint::from_str(
            "44412834903739585386157632289020980010620626017712148233229312325549216099227"
        ).unwrap());
        let point = EdwardsPoint::from((x, y));
        let inverse = -&point;
        assert!(point + inverse == EdwardsPoint::identity())
    }
}

*/