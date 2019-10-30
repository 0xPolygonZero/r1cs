use std::marker::PhantomData;

use crate::{Element, Expression, Field};

/// A short Weierstrass curve.
pub trait WeierstrassCurve<F: Field> {
    fn a() -> Element<F>;
    fn b() -> Element<F>;
}

/// An embedded Weierstrass curve point defined over the same base field as
/// the constraint system, with affine coordinates as elements.
pub struct WeierstrassPoint<F: Field, C: WeierstrassCurve<F>> {
    pub x: Element<F>,
    pub y: Element<F>,
    phantom: PhantomData<*const C>,
}

/// An embedded Weierstrass curve point defined over the same base field
/// as the field used in the constraint system, with affine coordinates as
/// expressions.
pub struct WeierstrassExpression<F: Field, C: WeierstrassCurve<F>> {
    pub x: Expression<F>,
    pub y: Expression<F>,
    phantom: PhantomData<*const C>,
}

/// An embedded Weierstrass curve point defined over the same base field
/// as the field used in the constraint system, with projective coordinates
/// as expressions.
pub struct ProjWeierstrassExpression<F: Field, C: WeierstrassCurve<F>> {
    pub x: Expression<F>,
    pub y: Expression<F>,
    pub z: Expression<F>,
    phantom: PhantomData<*const C>,
}
