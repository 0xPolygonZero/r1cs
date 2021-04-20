use std::marker::PhantomData;

use crate::{Element, Expression, Field};

/// A Montgomery curve.
pub trait MontgomeryCurve<F: Field> {
    fn a() -> Element<F>;
    fn b() -> Element<F>;
}

/// An embedded Montgomery curve point defined over the same base field as
/// the constraint system, with affine coordinates as elements.
pub struct MontgomeryPoint<F: Field, C: MontgomeryCurve<F>> {
    pub x: Element<F>,
    pub y: Element<F>,
    phantom: PhantomData<*const C>,
}

/// An embedded Montgomery curve point defined over the same base field
/// as the field used in the constraint system, with affine coordinates as
/// expressions.
pub struct MontgomeryExpression<F: Field, C: MontgomeryCurve<F>> {
    pub x: Expression<F>,
    pub y: Expression<F>,
    phantom: PhantomData<*const C>,
}
