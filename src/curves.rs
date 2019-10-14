use std::marker::PhantomData;

use crate::{Element, Evaluable, Expression, Field, GadgetBuilder, Group, WireValues};

pub trait TwistedEdwardsCurveParams<F: Field> {
    fn a() -> Element<F>;
    fn d() -> Element<F>;
}

pub struct AffineTwistedEdwardsCurve<F: Field, P: TwistedEdwardsCurveParams<F>> {
    phantom_f: PhantomData<*const F>,
    phantom_p: PhantomData<*const P>,
}

pub struct AffineTwistedEdwardsPoint<F: Field, P: TwistedEdwardsCurveParams<F>> {
    pub x: Element<F>,
    pub y: Element<F>,
    phantom: PhantomData<*const P>,
}

impl<F: Field, P: TwistedEdwardsCurveParams<F>> AffineTwistedEdwardsPoint<F, P> {
    pub fn new(x: Element<F>, y: Element<F>) -> AffineTwistedEdwardsPoint<F, P> {
        // TODO: Membership check.
        AffineTwistedEdwardsPoint { x, y, phantom: PhantomData }
    }
}

pub struct AffineTwistedEdwardsExpression<F: Field, P: TwistedEdwardsCurveParams<F>> {
    pub x: Expression<F>,
    pub y: Expression<F>,
    phantom: PhantomData<*const P>,
}

impl<F: Field, P: TwistedEdwardsCurveParams<F>> AffineTwistedEdwardsExpression<F, P> {
    pub fn new(x: Expression<F>, y: Expression<F>) -> AffineTwistedEdwardsExpression<F, P> {
        // TODO: Add constraints to verify membership.
        AffineTwistedEdwardsExpression::new_unsafe(x, y)
    }

    pub fn new_unsafe(x: Expression<F>, y: Expression<F>) -> AffineTwistedEdwardsExpression<F, P> {
        AffineTwistedEdwardsExpression { x, y, phantom: PhantomData }
    }
}

impl<F: Field, P: TwistedEdwardsCurveParams<F>>
From<&AffineTwistedEdwardsPoint<F, P>> for AffineTwistedEdwardsExpression<F, P> {
    fn from(point: &AffineTwistedEdwardsPoint<F, P>) -> Self {
        AffineTwistedEdwardsExpression {
            x: Expression::from(&point.x),
            y: Expression::from(&point.y),
            phantom: PhantomData,
        }
    }
}

impl<F: Field, P: TwistedEdwardsCurveParams<F>>
Evaluable<F, AffineTwistedEdwardsPoint<F, P>> for AffineTwistedEdwardsExpression<F, P> {
    fn evaluate(
        &self,
        wire_values: &WireValues<F>,
    ) -> AffineTwistedEdwardsPoint<F, P> {
        AffineTwistedEdwardsPoint {
            x: self.x.evaluate(wire_values),
            y: self.y.evaluate(wire_values),
            phantom: PhantomData,
        }
    }
}

impl<F: Field, P: TwistedEdwardsCurveParams<F>> Group<F> for AffineTwistedEdwardsCurve<F, P> {
    type GroupElement = AffineTwistedEdwardsPoint<F, P>;
    type GroupExpression = AffineTwistedEdwardsExpression<F, P>;

    fn identity_element() -> Self::GroupElement {
        AffineTwistedEdwardsPoint::new(Element::zero(), Element::one())
    }

    // TODO: improve the constraint count by using an improved addition algorithm
    fn add_expressions(
        builder: &mut GadgetBuilder<F>,
        lhs: &Self::GroupExpression,
        rhs: &Self::GroupExpression,
    ) -> Self::GroupExpression {
        let AffineTwistedEdwardsExpression { x: x1, y: y1, phantom: _ } = lhs;
        let AffineTwistedEdwardsExpression { x: x2, y: y2, phantom: _ } = rhs;
        let x1y2 = builder.product(&x1, &y2);
        let x2y1 = builder.product(&y1, &x2);
        let x1x2 = builder.product(&x1, &x2);
        let x1x2y1y2 = builder.product(&x1y2, &x2y1);
        let y1y2 = builder.product(&y1, &y2);
        let x3 = builder.quotient(
            &(x1y2 + x2y1),
            &(&x1x2y1y2 * &P::d() + Expression::one()));
        let y3 = builder.quotient(
            &(y1y2 - &x1x2 * &P::a()),
            &(&x1x2y1y2 * -&P::d() + Expression::one()));
        AffineTwistedEdwardsExpression::new_unsafe(x3, y3)
    }
}