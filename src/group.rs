use crate::{Field, GadgetBuilder, Evaluable, WireValues};
use std::marker::PhantomData;
use num::BigUint;

pub trait Group<F: Field> where Self::GroupExpression: for<'a> From<&'a Self::GroupElement>,
                                Self::GroupExpression: Evaluable<F, Self::GroupElement> {
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
}

pub trait CyclicGroup<F: Field>: Group<F> {
    fn generator() -> Self::GroupElement;
}

pub trait KnownOrderGroup<F: Field>: Group<F> {
    fn order() -> BigUint;
}

pub trait PrimeOrderGroup<F: Field>: CyclicGroup<F> {}

pub struct CyclicSubgroup<F: Field, G: Group<F>, Gen: GroupGenerator<G::GroupElement>> {
    phantom_f: PhantomData<*const F>,
    phantom_g: PhantomData<*const G>,
    phantom_gen: PhantomData<*const Gen>,
}

pub trait GroupGenerator<E> {
    fn generator() -> E;
}

impl<F: Field, G: Group<F>, Gen: GroupGenerator<G::GroupElement>> Group<F> for CyclicSubgroup<F, G, Gen> {
    type GroupElement = G::GroupElement;
    type GroupExpression = G::GroupExpression;

    fn identity_element() -> Self::GroupElement {
        G::identity_element()
    }

    fn add_expressions(
        builder: &mut GadgetBuilder<F>,
        lhs: &Self::GroupExpression,
        rhs: &Self::GroupExpression,
    ) -> Self::GroupExpression {
        G::add_expressions(builder, lhs, rhs)
    }
}

impl<F: Field, G: Group<F>, Gen: GroupGenerator<G::GroupElement>>
CyclicGroup<F> for CyclicSubgroup<F, G, Gen> {
    fn generator() -> Self::GroupElement {
        Gen::generator()
    }
}