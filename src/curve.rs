use std::marker::PhantomData;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::{Element, Evaluable, GroupExpression, Expression, Field, GadgetBuilder, Group, WireValues, BooleanExpression};

pub trait Curve<F: Field> {}

/// Trait used to represent Edwards Curves and Twisted Edwards Curves. Note that the `a`
/// parameter can be set to 1 to represent the less-general non-twisted Edwards Curves.
pub trait EdwardsCurve<F: Field> {
    fn a() -> Element<F>;
    fn d() -> Element<F>;
}

/// An embedded Edwards curve point defined over the same base field as
/// the constraint system, with affine coordinates as elements.
pub struct EdwardsPoint<F: Field, C: EdwardsCurve<F>> {
    x: Element<F>,
    y: Element<F>,
    phantom: PhantomData<*const C>,
}

impl<F: Field, C: EdwardsCurve<F>> Clone for EdwardsPoint<F, C> {
    fn clone(&self) -> Self {
        EdwardsPoint {
            x: self.x.clone(),
            y: self.y.clone(),
            phantom: PhantomData,
        }
    }
}

impl<F: Field, C: EdwardsCurve<F>> Clone for EdwardsExpression<F, C> {
    fn clone(&self) -> Self {
        EdwardsExpression {
            x: self.x.clone(),
            y: self.y.clone(),
            phantom: PhantomData,
        }
    }
}

/// An embedded Montgomery curve point defined over the same base field
/// as the field used in the constraint system, with affine coordinates as
/// expressions.
pub struct MontgomeryExpression<F: Field, C: Curve<F>> {
    x: Expression<F>,
    y: Expression<F>,
    phantom: PhantomData<*const C>,
}

/// An embedded Weierstrass curve point defined over the same base field
/// as the field used in the constraint system, with affine coordinates as
/// expressions.
pub struct WeierstrassExpression<F: Field, C: Curve<F>> {
    x: Expression<F>,
    y: Expression<F>,
    phantom: PhantomData<*const C>,
}

/// An embedded Weierstrass curve point defined over the same base field
/// as the field used in the constraint system, with projective coordinates
/// as expressions.
pub struct ProjWeierstrassExpression<F: Field, C: Curve<F>> {
    x: Expression<F>,
    y: Expression<F>,
    z: Expression<F>,
    phantom: PhantomData<*const C>,
}

impl<F: Field, C: EdwardsCurve<F>> EdwardsPoint<F, C> {
    pub fn new(x: Element<F>, y: Element<F>) -> EdwardsPoint<F, C> {
        assert!(C::a() * &x * &x + &y * &y == Element::one() + C::d() * &x * &x * &y * &y,
                "Point must be contained on the curve.");
        EdwardsPoint { x, y, phantom: PhantomData }
    }

    pub fn compressed_element(&self) -> &Element<F> {
        &self.y
    }
}

pub struct EdwardsExpression<F: Field, C: EdwardsCurve<F>> {
    pub x: Expression<F>,
    pub y: Expression<F>,
    phantom: PhantomData<*const C>,
}

impl<F: Field, C: EdwardsCurve<F>> EdwardsExpression<F, C> {

    /// Safely creates an `EdwardsExpression` from two coordinates of type `EdwardsExpression`.
    /// Automatically generates constraints that assert that the resulting curve point
    /// is contained in the EdwardsCurve.
    pub fn new(
        builder: &mut GadgetBuilder<F>,
        x: Expression<F>,
        y: Expression<F>,
    ) -> EdwardsExpression<F, C> {
        let x_squared = builder.product(&x, &x);
        let y_squared = builder.product(&y, &y);
        let x_squared_y_squared = builder.product(&x_squared, &y_squared);
        builder.assert_equal(&(&x_squared * C::a() + &y_squared),
                             &(&x_squared_y_squared * C::d() + Expression::one()));
        EdwardsExpression::new_unsafe(x, y)
    }

    /// Creates an `EdwardsExpression` from two arbitrary coordinates of type `Expression`.
    /// This method is unsafe and should only be used when the coordinates are proven
    /// to exist on the curve.
    pub fn new_unsafe(x: Expression<F>, y: Expression<F>) -> EdwardsExpression<F, C> {
        EdwardsExpression { x, y, phantom: PhantomData }
    }
}

impl<F: Field, C: EdwardsCurve<F>> GroupExpression<F> for EdwardsExpression<F, C> {
    fn compressed(&self) -> &Expression<F> { &self.y }
    fn to_components(&self) -> Vec<Expression<F>> { vec![self.x.clone(), self.y.clone()] }

    /// Given two group components of type `Expression`, creates an `EdwardsExpression`. Used
    /// in the generic implementation of scalar multiplication for groups.
    fn from_component_expression_unsafe(mut components: Vec<Expression<F>>) -> Self {
        let x = components.remove(0);
        let y = components.remove(0);
        Self::new_unsafe(x, y)
    }
}

impl<F: Field, C: EdwardsCurve<F>> From<&EdwardsPoint<F, C>> for EdwardsExpression<F, C> {
    fn from(point: &EdwardsPoint<F, C>) -> Self {
        EdwardsExpression {
            x: Expression::from(&point.x),
            y: Expression::from(&point.y),
            phantom: PhantomData,
        }
    }
}

impl<F: Field, C: EdwardsCurve<F>> From<(Element<F>, Element<F>)> for EdwardsExpression<F, C> {
    fn from(coordinates: (Element<F>, Element<F>)) -> Self {
        let point = EdwardsPoint::new(coordinates.0, coordinates.1);
        EdwardsExpression::from(&point)
    }
}

impl<F: Field, C: EdwardsCurve<F>> Evaluable<F, EdwardsPoint<F, C>> for EdwardsExpression<F, C> {
    fn evaluate(
        &self,
        wire_values: &WireValues<F>,
    ) -> EdwardsPoint<F, C> {
        EdwardsPoint {
            x: self.x.evaluate(wire_values),
            y: self.y.evaluate(wire_values),
            phantom: PhantomData,
        }
    }
}

impl<F: Field, C: EdwardsCurve<F>> Group<F> for C {
    type GroupElement = EdwardsPoint<F, C>;
    type GroupExpression = EdwardsExpression<F, C>;

    fn identity_element() -> Self::GroupElement {
        EdwardsPoint::new(Element::zero(), Element::one())
    }

    /// Adds two points on an `EdwardsCurve` using the standard algorithm for Twisted Edwards
    /// Curves.
    // TODO: improve the constraint count by using an improved addition algorithm
    fn add_expressions(
        builder: &mut GadgetBuilder<F>,
        lhs: &Self::GroupExpression,
        rhs: &Self::GroupExpression,
    ) -> Self::GroupExpression {
        let EdwardsExpression { x: x1, y: y1, phantom: _ } = lhs;
        let EdwardsExpression { x: x2, y: y2, phantom: _ } = rhs;
        let x1y2 = builder.product(&x1, &y2);
        let x2y1 = builder.product(&y1, &x2);
        let x1x2 = builder.product(&x1, &x2);
        let x1x2y1y2 = builder.product(&x1y2, &x2y1);
        let y1y2 = builder.product(&y1, &y2);
        let x3 = builder.quotient(
            &(x1y2 + x2y1),
            &(&x1x2y1y2 * &C::d() + Expression::one()));
        let y3 = builder.quotient(
            &(y1y2 - &x1x2 * &C::a()),
            &(&x1x2y1y2 * -&C::d() + Expression::one()));
        EdwardsExpression::new_unsafe(x3, y3)
    }

    // TODO: improve constraint count
    /// Naive implementation of the doubling algorithm for twisted Edwards curves.
    ///
    /// Assuming that `EdwardsPointExpressions` are on the curve, so the non-deterministic
    /// division method is acceptable, as the denominator is non-zero.
    ///
    /// Note that this algorithm requires the point to be of odd order, which, in the case
    /// of prime-order subgroups of Edwards curves, is satisfied.
    fn double_expression(
        builder: &mut GadgetBuilder<F>,
        point: &Self::GroupExpression,
    ) -> Self::GroupExpression {
        let EdwardsExpression { x, y, phantom: _ } = point;
        let a = C::a();

        let xy = builder.product(&x, &y);
        let xx = builder.product(&x, &x);
        let yy = builder.product(&y, &y);
        let x_2 = builder.quotient(&(&xy * Element::from(2u8)), &(&xx * &a + &yy));
        let y_2 = builder.quotient(&(&yy - &xx * &a),
                                   &(-&xx * &a - &yy + Expression::from(2u8)));

        EdwardsExpression::new_unsafe(x_2, y_2)
    }

    // TODO: implement Daira's algorithm from https://github.com/zcash/zcash/issues/3924
    // TODO: optimize for fixed-base multiplication using windowing, given a constant expression
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{EdwardsExpression, Expression, GadgetBuilder, WireValues, Group};
    use crate::jubjub::JubJubPrimeSubgroup;
    use crate::field::{Bls12_381, Element};

    #[test]
    fn point_on_curve() {
        let x = Element::from_str(
            "11076627216317271660298050606127911965867021807910416450833192264015104452986"
        ).unwrap();
        let y = Element::from_str(
            "44412834903739585386157632289020980010620626017712148233229312325549216099227"
        ).unwrap();

        let x_exp = Expression::from(x);
        let y_exp = Expression::from(y);

        let mut builder = GadgetBuilder::<Bls12_381>::new();
        let p = EdwardsExpression::<Bls12_381, JubJubPrimeSubgroup>::new(
            &mut builder, x_exp, y_exp);

        let gadget = builder.build();
        assert!(gadget.execute(&mut WireValues::new()));
    }

    #[test]
    fn point_not_on_curve_with_expressions() {
        let x = Element::from_str(
            "11076627216317271660298050606127911965867021807910416450833192264015104452986"
        ).unwrap();
        let y = Element::from_str(
            "44412834903739585386157632289020980010620626017712148233229312325549216099226"
        ).unwrap();

        let x_exp = Expression::from(x);
        let y_exp = Expression::from(y);

        let mut builder = GadgetBuilder::<Bls12_381>::new();
        let p
            = EdwardsExpression::<Bls12_381, JubJubPrimeSubgroup>::new(
            &mut builder,
            x_exp,
            y_exp
        );

        let gadget = builder.build();
        assert!(!gadget.execute(&mut WireValues::new()));
    }

    #[test]
    #[should_panic]
    fn point_not_on_curve() {
        let x = Element::from_str(
            "11076627216317271660298050606127911965867021807910416450833192264015104452985"
        ).unwrap();

        let y = Element::from_str(
            "44412834903739585386157632289020980010620626017712148233229312325549216099227"
        ).unwrap();

        EdwardsExpression::<Bls12_381, JubJubPrimeSubgroup>::from((x, y));
    }

    #[test]
    fn add_and_negate() {
        let x1 = Element::<Bls12_381>::from_str(
            "11076627216317271660298050606127911965867021807910416450833192264015104452986"
        ).unwrap();
        let y1 = Element::<Bls12_381>::from_str(
            "44412834903739585386157632289020980010620626017712148233229312325549216099227"
        ).unwrap();

        let p1
            = EdwardsExpression::<Bls12_381, JubJubPrimeSubgroup>::from((x1, y1));

        let p2
            = EdwardsExpression::<Bls12_381, JubJubPrimeSubgroup>::new_unsafe(-p1.x.clone(), p1.y.clone());

        let mut builder = GadgetBuilder::<Bls12_381>::new();
        let p3 = JubJubPrimeSubgroup::add_expressions(&mut builder, &p1, &p2);
        let gadget = builder.build();
        let mut values = WireValues::new();
        gadget.execute(&mut values);
        assert_eq!(p3.x.evaluate(&values), Element::zero());
        assert_eq!(p3.y.evaluate(&values), Element::one());
    }

    #[test]
    fn scalar_mult() {
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

        let p1
            = EdwardsExpression::<Bls12_381, JubJubPrimeSubgroup>::from((x1, y1));

        let mut builder = GadgetBuilder::<Bls12_381>::new();
        let p3 = JubJubPrimeSubgroup::scalar_mult_expression(
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