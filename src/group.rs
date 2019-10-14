use std::str::FromStr;

use num::BigUint;

use crate::{BooleanExpression, Element, Evaluable, Expression, Field, GadgetBuilder, WireValues};

/// An embedded group. Note that we use additive notation for group operations.
trait Group<F: Field> where Self::GroupExpression: for<'a> From<&'a Self::GroupElement>,
                            Self::GroupExpression: Evaluable<F, Self::GroupElement> {
    type GroupElement;
    type GroupExpression;

    fn add_expressions(
        &self,
        builder: &mut GadgetBuilder<F>,
        a: &Self::GroupExpression,
        b: &Self::GroupExpression,
    ) -> Self::GroupExpression;

    fn add_elements(&self, a: &Self::GroupElement, b: &Self::GroupElement) -> Self::GroupElement {
        let mut builder = GadgetBuilder::new();
        let sum = Self::add_expressions(
            self, &mut builder, &Self::GroupExpression::from(a), &Self::GroupExpression::from(b));
        let mut values = WireValues::new();
        builder.build().execute(&mut values);
        sum.evaluate(&values)
    }

    fn double_expression(
        &self,
        builder: &mut GadgetBuilder<F>,
        element: &Self::GroupExpression,
    ) -> Self::GroupExpression {
        self.add_expressions(element, element)
    }

    fn double_element(&self, expression: &Self::GroupElement) -> Self::GroupElement {
        self.add_elements(expression, expression)
    }

    // TODO: implement Daira's algorithm from https://github.com/zcash/zcash/issues/3924
    fn expression_times_expression(
        &self,
        builder: &mut GadgetBuilder<F>,
        expression: &Self::GroupExpression,
        scalar: &Expression<F>,
    ) -> Self::GroupExpression {
        let scalar_binary = builder.split_allowing_ambiguity(&scalar);

        let mut sum = Self::identity();
        let mut current = point.clone();
        for bit in scalar_binary.bits {
            let boolean_product = &Self::boolean_mult(builder, &current, &bit);
            sum = Self::add(builder, &sum, boolean_product);
            current = Self::double(builder, &current);
        }
        sum
    }

    /// Given a boolean element, return the given element if element is on, otherwise
    /// return the identity.
    fn expression_times_bool(
        builder: &mut GadgetBuilder<F>,
        point: &Self::GroupExpression,
        boolean: &BooleanExpression<F>,
    ) -> EdwardsPointExpression<F, C> {
        let x = builder.selection(boolean, &point.x, &Expression::zero());
        let y = builder.selection(boolean, &point.y, &Expression::one());
        EdwardsPointExpression::from_expressions_unsafe(x, y)
    }
}

trait CyclicGroup<F: Field>: Group<F> {
    fn generator() -> Self::GroupElement;
}

/// A marker trait indicating that a cycling group has prime order.
trait PrimeOrderGroup<F: Field>: CyclicGroup<F> {}

trait KnownOrderGroup<F: Field>: Group<F> {
    fn order() -> BigUint;
}

struct Subgroup<F: Field, G: Group<F>> {
    group: G,
    generator: G::GroupElement,
}

impl<F: Field, G: Group<F>> Group<F> for Subgroup<F, G> {
    type GroupElement = G::GroupElement;
    type GroupExpression = G::GroupExpression;

    fn add_expressions(
        &self,
        builder: &mut GadgetBuilder<F>,
        a: &Self::GroupExpression,
        b: &Self::GroupExpression,
    ) -> Self::GroupExpression {
        self.group.add_expressions(builder, a, b)
    }

    fn add_elements(&self, a: &Self::GroupElement, b: &Self::GroupElement) -> Self::GroupElement {
        self.group.add_elements(a, b)
    }
}
