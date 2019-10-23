use crate::{Field, GadgetBuilder, Evaluable, WireValues, BooleanExpression, Element, Expression};
use std::marker::PhantomData;
use num::BigUint;

pub trait Group<F: Field> where Self::GroupExpression: for<'a> From<&'a Self::GroupElement> + for<'b> From<Vec<&'b Expression<F>>>,
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

    fn scalar_mult_expression(
        builder: &mut GadgetBuilder<F>,
        expression: &Self::GroupExpression,
        scalar: &Expression<F>,
    ) -> Self::GroupExpression {
        let scalar_binary = builder.split_allowing_ambiguity(&scalar);

        let mut sum = Self::identity_expression();
        let mut current = expression.clone();
        for bit in scalar_binary.bits {
            let boolean_product = Self::boolean_mult_expression(builder, &current, &bit);
            sum = Self::add_expressions(builder, &sum, &boolean_product);
            current = Self::double_expression(builder, &current);
        }
        sum
    }

    fn boolean_mult_expression(
        builder: &mut GadgetBuilder<F>,
        expression: &Self::GroupExpression,
        boolean: &BooleanExpression<F>,
    ) -> Self::GroupExpression {
        let coordinates = expression.to_coordinate_expression();

        let x = builder.selection(boolean, &coordinates[0], &Expression::zero());
        let y = builder.selection(boolean, &coordinates[1], &Expression::one());

        Self::GroupExpression::from(vec![&x, &y])
    }

    fn scalar_mult_element(
        element: &Self::GroupElement,
        scalar: &Element<F>
    ) -> Self::GroupElement;
}

pub trait CyclicGroup<F: Field>: Group<F> {

    fn generator_element() -> Self::GroupElement;

    fn generator_expression() -> Self::GroupExpression {
        Self::GroupExpression::from(&Self::generator_element())
    }
}

/// Applies a (not necessarily injective) map, defined from a group to the field,
/// to an expression corresponding to an element in the group.
pub trait GroupExpression<F: Field> where Self: for<'b> From<Vec<&'b Expression<F>>> {
    fn compressed_expression(&self) -> &Expression<F>;
    fn to_coordinate_expression(&self) -> Vec<&Expression<F>>;
}