use crate::{Field, GadgetBuilder, Evaluable, WireValues};
use std::marker::PhantomData;
use num::BigUint;

pub trait Group<F: Field> where Self::GroupExpression: for<'a> From<&'a Self::GroupElement>,
                                Self::GroupExpression: Evaluable<F, Self::GroupElement> {
    type GroupElement;
    type GroupExpression;

    fn identity() -> Self::GroupElement;

    fn identity_expression() -> Self::GroupExpression {
        Self::GroupExpression::from(&Self::identity())
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
    ) -> Self::GroupExpression;

    fn double_element(element: &Self::GroupElement) -> Self::GroupElement {
        let exp = Self::GroupExpression::from(element);

        let mut builder = GadgetBuilder::new();

        let doubled = Self::double_expression(&mut builder, exp);
        let mut values = WireValues::new();
        builder.build().execute(&mut values);
        doubled.evaluate(&values)
    }
}

pub trait CyclicGroup<F: Field>: Group<F> {
    fn generator() -> Self::GroupElement;
}