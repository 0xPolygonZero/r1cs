use std::borrow::Borrow;
use std::fmt::Formatter;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Shl, Sub, SubAssign};
use std::str::FromStr;

use num::bigint::ParseBigIntError;
use num::BigUint;
use num::pow;

use crate::{Expression, GadgetBuilder, BooleanExpression};
use crate::embedded_curve::EmbeddedCurve;
use crate::field::{Element, Field};

pub trait Curve<F: Field> {}

pub trait CurvePoint<F: Field, C: Curve<F>> {}

/// An embedded twisted Edwards curve defined over the same base field
/// as the field used in the constraint system
pub trait EdwardsCurve<F: Field> {
    fn a() -> Element<F>;
    fn d() -> Element<F>;
    fn subgroup_generator() -> (Element<F>, Element<F>);
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
    /// edwards_point.compressed()
    ///
    /// Returns the Y coordinate of an Edwards Point Expression
    pub fn compressed(&self) -> &Expression<F> {
        &self.y
    }

    /// EdwardsPointExpression::add(builder, e1, e2)
    ///
    /// Assumes that the EdwardsPointExpressions are known to be contained on the curve
    /// (and omits a membership check), so the non-deterministic inversion method is valid.
    // TODO: improve the constraint count by using an improved addition algorithm
    pub fn add(
        builder: &mut GadgetBuilder<F>,
        point_1: &EdwardsPointExpression<F, C>,
        point_2: &EdwardsPointExpression<F, C>,
    ) -> EdwardsPointExpression<F, C> {
        let d = C::d();
        let a = C::a();
        // TODO: better method for specifying variables
        let EdwardsPointExpression { x: x1, y: y1, phantom } = point_1;
        let EdwardsPointExpression { x: x2, y: y2, phantom } = point_2;
        let x1y2 = builder.product(&x1, &y2);
        let x2y1 = builder.product(&y1, &x2);
        let x1x2 = builder.product(&x1, &x2);
        let x1x2y1y2 = builder.product(&x1y2, &x2y1);
        let y1y2 = builder.product(&y1, &y2);
        let x3 = builder.quotient(&(x1y2 + x2y1), &(&x1x2y1y2 * &d + Expression::one()));
        let y3 = builder.quotient(&(y1y2 - &x1x2 * &a), &(&x1x2y1y2 * -&d + Expression::one()));
        EdwardsPointExpression::new_unsafe(x3, y3)
    }

    // TODO: improve constraint count
    /// Naive implementation of the doubling algorithm for twisted Edwards curves.
    ///
    /// Assuming that EdwardsPointExpressions are on the curve, so the non-deterministic
    /// division method is acceptable, as the denominator is non-zero.
    ///
    /// Note that this algorithm requires the point to be of odd order, which, in the case
    /// of prime-order subgroups of Edwards curves, is satisfied.
    pub fn double(
        builder: &mut GadgetBuilder<F>,
        point: &EdwardsPointExpression<F, C>,
    ) -> EdwardsPointExpression<F, C> {
        let EdwardsPointExpression { x, y, phantom } = point;
        let a = C::a();

        let xy = builder.product(&x, &y);
        let xx = builder.product(&x, &x);
        let yy = builder.product(&y, &y);
        let x_2 = builder.quotient(&(&xy * Element::from(2u8)), &(&xx * &a + &yy));
        let y_2 = builder.quotient(&(&yy - &xx * &a),
                                   &(-&xx * &a - &yy + Expression::from(2u8)));

        EdwardsPointExpression::new_unsafe(x_2, y_2)
    }

    /// Multiplies an EdwardsPointExpression by a scalar using a naive approach consisting of
    /// multiplication by doubling.
    // TODO: implement Daira's algorithm from https://github.com/zcash/zcash/issues/3924
    pub fn scalar_mult(
        builder: &mut GadgetBuilder<F>,
        point: &EdwardsPointExpression<F, C>,
        scalar: &Expression<F>
    ) -> EdwardsPointExpression<F, C> {
        let scalar_binary = builder.split_allowing_ambiguity(&scalar);

        let mut sum = Self::identity();
        let mut current = point;
        for bit in scalar_binary.bits {
            let boolean_product= &Self::boolean_mult(builder, current, &bit);
            sum = Self::add(builder, &sum,boolean_product);
            let current = &Self::double(builder, current);
        }
        sum
    }

    /// Given a boolean element, return the given element if element is on, otherwise
    /// return the identity.
    fn boolean_mult(
        builder: &mut GadgetBuilder<F>,
        point: &EdwardsPointExpression<F, C>,
        boolean: &BooleanExpression<F>,
    ) -> EdwardsPointExpression<F, C> {
        let x = builder.selection(boolean, &point.x, &Expression::zero());
        let y = builder.selection(boolean, &point.y, &Expression::one());
        EdwardsPointExpression::new_unsafe(x, y)
    }

    /// Identity element for twisted Edwards Curve
    pub fn identity() -> EdwardsPointExpression<F, C> {
        EdwardsPointExpression::new_unsafe(Expression::zero(), Expression::one())
    }

    /// Takes two elements as coordinates, checks that they're on the curve without adding
    /// constraints, and then returns an EdwardsPointExpression
    pub fn from_elements(x: Element<F>, y: Element<F>) -> EdwardsPointExpression<F, C> {
        assert!(C::a() * &x * &x + &y * &y == Element::one() + C::d() * &x * &x * &y * &y,
                "Point must be contained on the curve.");
        EdwardsPointExpression::new_unsafe(Expression::from(x), Expression::from(y))
    }

    /// Takes two expressions as coordinates, adds constraints verifying that the coordinates
    /// are contained on the specified curve, and then returns an EdwardsPointExpression
    pub fn from_expressions(builder: &mut GadgetBuilder<F>, x: Expression<F>, y: Expression<F>) -> EdwardsPointExpression<F, C> {
        let x_squared = builder.product(&x, &x);
        let y_squared = builder.product(&y, &y);
        let x_squared_y_squared = builder.product(&x_squared, &y_squared);
        builder.assert_equal(&(&x_squared * C::a() + &y_squared),
                             &(&x_squared_y_squared * C::d() + Expression::one()));
        EdwardsPointExpression::new_unsafe(x, y)
    }

    /// Takes two expressions as coordinates, does not perform a check or add constraints
    /// to check that the coordinates are on the specified curve, and then returns an
    /// EdwardsPointExpression
    pub fn new_unsafe(x: Expression<F>, y: Expression<F>) -> EdwardsPointExpression<F, C> {
        EdwardsPointExpression { x, y, phantom: PhantomData }
    }
}

#[cfg(test)]
mod tests {
    use std::iter;
    use std::str::FromStr;

    use itertools::assert_equal;
    use num::BigUint;

    use crate::curve::{EdwardsCurve};
    use crate::field::{Bls12_381, Bn128, Element, Field};
    use crate::{EdwardsPointExpression, Expression, GadgetBuilder, WireValues};

    struct JubJub {}

    impl EdwardsCurve<Bls12_381> for JubJub {
        fn a() -> Element<Bls12_381> {
            -Element::one()
        }

        fn d() -> Element<Bls12_381> {
            Element::from_str(
                "19257038036680949359750312669786877991949435402254120286184196891950884077233"
            ).unwrap()
        }

        fn subgroup_generator() -> (Element<Bls12_381>, Element<Bls12_381>) {
            let x = Element::from_str(
                "16540640123574156134436876038791482806971768689494387082833631921987005038935"
            ).unwrap();
            let y = Element::from_str(
                "20819045374670962167435360035096875258406992893633759881276124905556507972311"
            ).unwrap();

            (x ,y)
        }
    }

    #[test]
    fn check_point_on_curve() {

        let x = Element::from_str(
            "11076627216317271660298050606127911965867021807910416450833192264015104452986"
        ).unwrap();
        let y = Element::from_str(
            "44412834903739585386157632289020980010620626017712148233229312325549216099227"
        ).unwrap();

        let x_exp = Expression::from(x);
        let y_exp = Expression::from(y);

        let mut builder = GadgetBuilder::<Bls12_381>::new();
        let p = EdwardsPointExpression::<Bls12_381, JubJub>::from_expressions(
            &mut builder, x_exp, y_exp);

        let gadget = builder.build();
        assert!(gadget.execute(&mut WireValues::new()));
    }

    #[test]
    fn check_point_not_on_curve_with_expressions() {

        let x = Element::from_str(
            "11076627216317271660298050606127911965867021807910416450833192264015104452986"
        ).unwrap();
        let y = Element::from_str(
            "44412834903739585386157632289020980010620626017712148233229312325549216099226"
        ).unwrap();

        let x_exp = Expression::from(x);
        let y_exp = Expression::from(y);

        let mut builder = GadgetBuilder::<Bls12_381>::new();
        let p = EdwardsPointExpression::<Bls12_381, JubJub>::from_expressions(
            &mut builder, x_exp, y_exp);

        let gadget = builder.build();
        assert!(!gadget.execute(&mut WireValues::new()));
    }

    #[test]
    #[should_panic]
    fn check_point_not_on_curve() {
        let x = Element::from_str(
            "11076627216317271660298050606127911965867021807910416450833192264015104452985"
        ).unwrap();

        let y = Element::from_str(
            "44412834903739585386157632289020980010620626017712148233229312325549216099227"
        ).unwrap();

        EdwardsPointExpression::<Bls12_381, JubJub>::from_elements(x, y);
    }

    #[test]
    fn check_add_and_negate() {
        let x1 = Element::<Bls12_381>::from_str(
            "11076627216317271660298050606127911965867021807910416450833192264015104452986"
        ).unwrap();
        let y1 = Element::<Bls12_381>::from_str(
            "44412834903739585386157632289020980010620626017712148233229312325549216099227"
        ).unwrap();

        let p1 = EdwardsPointExpression::<Bls12_381, JubJub>::from_elements(x1, y1);

        let p2= EdwardsPointExpression::<Bls12_381, JubJub>::new_unsafe(-p1.x.clone(), p1.y.clone());

        let mut builder = GadgetBuilder::<Bls12_381>::new();
        let p3 = EdwardsPointExpression::<Bls12_381, JubJub>::add(&mut builder, &p1, &p2);
        let gadget = builder.build();
        let mut values = WireValues::new();
        gadget.execute(&mut values);
        assert_eq!(p3.x.evaluate(&values), Element::zero());
        assert_eq!(p3.y.evaluate(&values), Element::one());
    }

    #[test]
    fn check_double() {
        let x1 = Element::<Bls12_381>::from_str(
            "11076627216317271660298050606127911965867021807910416450833192264015104452986"
        ).unwrap();
        let y1 = Element::<Bls12_381>::from_str(
            "44412834903739585386157632289020980010620626017712148233229312325549216099227"
        ).unwrap();

        let scalar = Expression::<Bls12_381>::from(
            Element::<Bls12_381>::from_str(
                "444128349033229312325549216099227444128349033229312325549216099220000000"
            ).unwrap()
        );

        let p1 = EdwardsPointExpression::<Bls12_381, JubJub>::from_elements(x1, y1);

        let mut builder = GadgetBuilder::<Bls12_381>::new();
        let p3 = EdwardsPointExpression::<Bls12_381, JubJub>::scalar_mult(
            &mut builder,
            &p1,
            &scalar,
        );
        let gadget = builder.build();
        let mut values = WireValues::new();
        gadget.execute(&mut values);

        // TODO: include assertion
    }
}