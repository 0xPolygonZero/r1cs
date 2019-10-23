#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use std::marker::PhantomData;

use num::BigUint;

use crate::{BooleanExpression, Element, Evaluable, Expression, Field, GadgetBuilder, WireValues};

pub trait Group<F: Field> where Self::GroupExpression: for<'a> From<&'a Self::GroupElement>,
                                Self::GroupExpression: Evaluable<F, Self::GroupElement>,
                                Self::GroupExpression: GroupExpression<F>,
                                Self::GroupExpression: Clone {
    type GroupElement;
    type GroupExpression;

    fn identity_element() -> Self::GroupElement;

    fn identity_expression() -> Self::GroupExpression {
        Self::GroupExpression::from(&Self::identity_element())
    }

    fn add_expressions(
        builder: &mut GadgetBuilder<F>,
        lhs: &Self::GroupExpression,
        rhs: &Self::GroupExpression,
    ) -> Self::GroupExpression;

    fn add_elements(
        lhs: &Self::GroupElement,
        rhs: &Self::GroupElement,
    ) -> Self::GroupElement {
        let lhs_exp = Self::GroupExpression::from(lhs);
        let rhs_exp = Self::GroupExpression::from(rhs);

        let mut builder = GadgetBuilder::new();
        let sum = Self::add_expressions(&mut builder, &lhs_exp, &rhs_exp);
        let mut values = WireValues::new();
        builder.build().execute(&mut values);
        sum.evaluate(&values)
    }

    fn double_expression(
        builder: &mut GadgetBuilder<F>,
        expression: &Self::GroupExpression,
    ) -> Self::GroupExpression {
        Self::add_expressions(builder, expression, expression)
    }

    fn double_element(element: &Self::GroupElement) -> Self::GroupElement {
        Self::add_elements(element, element)
    }

    /// Performs scalar multiplication in constraints by first splitting up a scalar into
    /// a binary representation, and then performing the naive double-or-add algorithm. This
    /// implementation is generic for all groups.
    fn mul_scalar_expression(
        builder: &mut GadgetBuilder<F>,
        expression: &Self::GroupExpression,
        scalar: &Expression<F>,
    ) -> Self::GroupExpression {
        let scalar_binary = builder.split_allowing_ambiguity(&scalar);

        let mut sum = Self::identity_expression();
        let mut current = expression.clone();
        for bit in scalar_binary.bits {
            let boolean_product = Self::mul_boolean_expression(builder, &current, &bit);
            sum = Self::add_expressions(builder, &sum, &boolean_product);
            current = Self::double_expression(builder, &current);
        }
        sum
    }

    /// Like `mul_scalart`, but actually evaluates the compression function rather than just adding it
    /// to a `GadgetBuilder`.
    fn mul_scalar_element(
        element: &Self::GroupElement,
        scalar: &Element<F>,
    ) -> Self::GroupElement {
        let mut builder = GadgetBuilder::new();
        let new_point = Self::mul_scalar_expression(
            &mut builder,
            &Self::GroupExpression::from(element),
            &Expression::from(scalar),
        );
        let mut values = WireValues::new();
        builder.build().execute(&mut values);
        new_point.evaluate(&values)
    }

    /// Given a boolean element, return the given element if element is on, otherwise
    /// return the identity.
    fn mul_boolean_expression(
        builder: &mut GadgetBuilder<F>,
        expression: &Self::GroupExpression,
        boolean: &BooleanExpression<F>,
    ) -> Self::GroupExpression {
        let coordinates = expression.to_components();

        let mut r = Vec::new();
        let ic = Self::identity_expression().to_components();

        for (i, x) in coordinates.iter().enumerate() {
            r.push(builder.selection(boolean, &x, &ic[i]));
        }

        Self::GroupExpression::from_component_expression_unsafe(r)
    }
}

/// A trait that defines a generator `g` for a cyclic group in which every element
/// is defined as `g^a` for some scalar `a`.
pub trait CyclicGroup<F: Field>: Group<F> {
    fn generator_element() -> Self::GroupElement;

    fn generator_expression() -> Self::GroupExpression {
        Self::GroupExpression::from(&Self::generator_element())
    }
}

/// Applies a (not necessarily injective) map, defined from a group to the field,
/// to an expression corresponding to an element in the group.
pub trait GroupExpression<F: Field> {
    fn compressed(&self) -> &Expression<F>;
    fn to_components(&self) -> Vec<Expression<F>>;
    fn from_component_expression_unsafe(components: Vec<Expression<F>>) -> Self;
}