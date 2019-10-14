use std::marker::PhantomData;

use crate::{BooleanExpression, Element, Expression, Field, GadgetBuilder, WireValues, Evaluable};

/// An embedded twisted Edwards curve defined over the same base field
/// as the field used in the constraint system
pub struct TwistedEdwardsCurve<F: Field> {
    a: Element<F>,
    d: Element<F>,
}

pub struct AffineCurvePoint<F: Field> {
    x: Element<F>,
    y: Element<F>,
}

pub struct AffineCurveExpression<F: Field> {
    x: Expression<F>,
    y: Expression<F>,
}

impl<F: Field> Evaluable<F, AffineCurvePoint<F>> for AffineCurveExpression<F> {
    fn evaluate(&self, wire_values: &WireValues<F>) -> AffineCurvePoint<F> {
        AffineCurvePoint {
            x: self.x.evaluate(wire_values),
            y: self.y.evaluate(wire_values),
        }
    }
}

impl<F: Field> From<&AffineCurvePoint<F>> for AffineCurveExpression<F> {
    fn from(point: &AffineCurvePoint<F>) -> AffineCurveExpression<F> {
        AffineCurveExpression {
            x: Expression::from(&point.x),
            y: Expression::from(&point.y),
        }
    }
}

pub struct ProjectiveCurvePoint<F: Field> {
    x: Element<F>,
    y: Element<F>,
    z: Element<F>,
}

pub struct ProjectiveCurveExpression<F: Field> {
    x: Expression<F>,
    y: Expression<F>,
    z: Expression<F>,
}

impl<F: Field> Group<F> for TwistedEdwardsCurve<F> {
    type GroupElement = AffineCurvePoint<F>;
    type GroupExpression = AffineCurveExpression<F>;

    fn add_expressions(
        &self,
        builder: &mut GadgetBuilder<F>,
        a: &Self::GroupExpression,
        b: &Self::GroupExpression,
    ) -> Self::GroupExpression {
        let AffineCurveExpression { x: x1, y: y1 } = a;
        let AffineCurveExpression { x: x2, y: y2 } = b;
        let x1y2 = builder.product(&x1, &y2);
        let x2y1 = builder.product(&y1, &x2);
        let x1x2 = builder.product(&x1, &x2);
        let x1x2y1y2 = builder.product(&x1y2, &x2y1);
        let y1y2 = builder.product(&y1, &y2);
        let x3 = builder.quotient(
            &(x1y2 + x2y1),
            &(&x1x2y1y2 * &self.d + Expression::one()));
        let y3 = builder.quotient(
            &(y1y2 - &x1x2 * &self.a),
            &(&x1x2y1y2 * -&self.d + Expression::one()));
        AffineCurveExpression { x: x3, y: y3 }
    }
}

impl<F: Field, C: EdwardsCurve<F>> EdwardsPoint<F, C> {
    /// Like `scalar_mult`, but actually evaluates the compression function rather than just adding it
    /// to a `GadgetBuilder`.
    pub fn scalar_mult_evaluate(&self, scalar: &Element<F>) -> EdwardsPoint<F, C> {
        let mut builder = GadgetBuilder::new();
        let new_point = EdwardsPointExpression::scalar_mult(
            &mut builder,
            &EdwardsPointExpression::from_edwards_point(self.clone()),
            &Expression::from(scalar),
        );
        let mut values = WireValues::new();
        builder.build().execute(&mut values);
        new_point.evaluate(&values)
    }

    /// Given an `x` and `y` coordinate, checks that they constitute a point on the curve
    /// and returns an `EdwardsPoint`
    pub fn from_elements(x: Element<F>, y: Element<F>) -> EdwardsPoint<F, C> {
        assert!(C::a() * &x * &x + &y * &y == Element::one() + C::d() * &x * &x * &y * &y,
                "Point must be contained on the curve.");
        EdwardsPoint { x, y, phantom: PhantomData }
    }

    /// Returns the Y coordinate of an `EdwardsPoint`
    pub fn compressed(&self) -> &Element<F> {
        &self.y
    }
}


impl<F: Field, C: EdwardsCurve<F>> EdwardsPointExpression<F, C> {
    /// Returns the Y coordinate of an `EdwardsPointExpression`
    pub fn compressed(&self) -> &Expression<F> {
        &self.y
    }

    /// Assumes that the `EdwardsPointExpressions` are known to be contained on the curve
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
        let EdwardsPointExpression { x: x1, y: y1, phantom: _ } = point_1;
        let EdwardsPointExpression { x: x2, y: y2, phantom: _ } = point_2;
        let x1y2 = builder.product(&x1, &y2);
        let x2y1 = builder.product(&y1, &x2);
        let x1x2 = builder.product(&x1, &x2);
        let x1x2y1y2 = builder.product(&x1y2, &x2y1);
        let y1y2 = builder.product(&y1, &y2);
        let x3 = builder.quotient(&(x1y2 + x2y1), &(&x1x2y1y2 * &d + Expression::one()));
        let y3 = builder.quotient(&(y1y2 - &x1x2 * &a), &(&x1x2y1y2 * -&d + Expression::one()));
        EdwardsPointExpression::from_expressions_unsafe(x3, y3)
    }

    // TODO: improve constraint count
    /// Naive implementation of the doubling algorithm for twisted Edwards curves.
    ///
    /// Assuming that `EdwardsPointExpressions` are on the curve, so the non-deterministic
    /// division method is acceptable, as the denominator is non-zero.
    ///
    /// Note that this algorithm requires the point to be of odd order, which, in the case
    /// of prime-order subgroups of Edwards curves, is satisfied.
    pub fn double(
        builder: &mut GadgetBuilder<F>,
        point: &EdwardsPointExpression<F, C>,
    ) -> EdwardsPointExpression<F, C> {
        let EdwardsPointExpression { x, y, phantom: _ } = point;
        let a = C::a();

        let xy = builder.product(&x, &y);
        let xx = builder.product(&x, &x);
        let yy = builder.product(&y, &y);
        let x_2 = builder.quotient(&(&xy * Element::from(2u8)), &(&xx * &a + &yy));
        let y_2 = builder.quotient(&(&yy - &xx * &a),
                                   &(-&xx * &a - &yy + Expression::from(2u8)));

        EdwardsPointExpression::from_expressions_unsafe(x_2, y_2)
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
        EdwardsPointExpression::from_expressions_unsafe(x, y)
    }

    /// Identity element for twisted Edwards Curve
    pub fn identity() -> EdwardsPointExpression<F, C> {
        EdwardsPointExpression::from_expressions_unsafe(Expression::zero(), Expression::one())
    }

    /// Takes two elements as coordinates, checks that they're on the curve without adding
    /// constraints, and then returns an EdwardsPointExpression
    pub fn from_elements(x: Element<F>, y: Element<F>) -> EdwardsPointExpression<F, C> {
        let p = EdwardsPoint::<F, C>::from_elements(x, y);
        EdwardsPointExpression::from_edwards_point(p)
    }

    /// Converts an EdwardsPoint into an EdwardsPointExpression. Assumes that the coordinates
    /// of the EdwardsPoint have already been verified on the curve
    pub fn from_edwards_point(p: EdwardsPoint<F, C>) -> EdwardsPointExpression<F, C> {
        EdwardsPointExpression::from_expressions_unsafe(Expression::from(p.x), Expression::from(p.y))
    }

    /// Takes two expressions as coordinates, adds constraints verifying that the coordinates
    /// are contained on the specified curve, and then returns an EdwardsPointExpression
    pub fn from_expressions(builder: &mut GadgetBuilder<F>, x: Expression<F>, y: Expression<F>) -> EdwardsPointExpression<F, C> {
        let x_squared = builder.product(&x, &x);
        let y_squared = builder.product(&y, &y);
        let x_squared_y_squared = builder.product(&x_squared, &y_squared);
        builder.assert_equal(&(&x_squared * C::a() + &y_squared),
                             &(&x_squared_y_squared * C::d() + Expression::one()));
        EdwardsPointExpression::from_expressions_unsafe(x, y)
    }

    /// Takes two expressions as coordinates, does not perform a check or add constraints
    /// to check that the coordinates are on the specified curve, and then returns an
    /// EdwardsPointExpression
    pub fn from_expressions_unsafe(x: Expression<F>, y: Expression<F>) -> EdwardsPointExpression<F, C> {
        EdwardsPointExpression { x, y, phantom: PhantomData }
    }

    /// Evaluates the EdwardsPointExpression by evaluating the expression in each coordinate
    pub fn evaluate(&self, values: &WireValues<F>) -> EdwardsPoint<F, C> {
        let x = self.x.evaluate(values);
        let y = self.y.evaluate(values);
        EdwardsPoint::from_elements(x, y)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{Expression, GadgetBuilder, WireValues};
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
        let p = EdwardsPointExpression::<Bls12_381, JubJub>::from_expressions(
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
        let p = EdwardsPointExpression::<Bls12_381, JubJub>::from_expressions(
            &mut builder, x_exp, y_exp);

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

        EdwardsPointExpression::<Bls12_381, JubJub>::from_elements(x, y);
    }

    #[test]
    fn add_and_negate() {
        let x1 = Element::<Bls12_381>::from_str(
            "11076627216317271660298050606127911965867021807910416450833192264015104452986"
        ).unwrap();
        let y1 = Element::<Bls12_381>::from_str(
            "44412834903739585386157632289020980010620626017712148233229312325549216099227"
        ).unwrap();

        let p1 = EdwardsPointExpression::<Bls12_381, JubJub>::from_elements(x1, y1);

        let p2 = EdwardsPointExpression::<Bls12_381, JubJub>::from_expressions_unsafe(-p1.x.clone(), p1.y.clone());

        let mut builder = GadgetBuilder::<Bls12_381>::new();
        let p3 = EdwardsPointExpression::<Bls12_381, JubJub>::add(&mut builder, &p1, &p2);
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