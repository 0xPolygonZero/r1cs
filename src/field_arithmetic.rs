//! This module extends GadgetBuilder with native field arithmetic methods.

use core::borrow::Borrow;

use itertools::enumerate;

use crate::expression::{BooleanExpression, Expression};
use crate::field::Field;
use crate::gadget_builder::GadgetBuilder;
use crate::wire_values::WireValues;

impl<F: Field> GadgetBuilder<F> {
    /// x * y
    pub fn product<E1, E2>(&mut self, x: E1, y: E2) -> Expression<F>
        where E1: Borrow<Expression<F>>, E2: Borrow<Expression<F>> {
        let x = x.borrow();
        let y = y.borrow();
        if let Some(c) = x.as_constant() {
            return y * c;
        }
        if let Some(c) = y.as_constant() {
            return x * c;
        }

        let product = self.wire();
        let product_exp = Expression::from(product);
        self.assert_product(x, y, &product_exp);

        {
            let x = x.clone();
            let y = y.clone();
            self.generator(
                [x.dependencies(), y.dependencies()].concat(),
                move |values: &mut WireValues<F>| {
                    let product_value = x.evaluate(values) * y.evaluate(values);
                    values.set(product, product_value);
                },
            );
        }

        product_exp
    }

    /// x^p for a constant p.
    pub fn exp<E: Borrow<Expression<F>>>(&mut self, x: E, p: usize) -> Expression<F> {
        // This is exponentiation by squaring. Generate a list squares where squares[i] = x^(2^i).
        let mut squares = vec![x.borrow().clone()];
        let mut i = 1;
        loop {
            let q = 1 << i;
            if q > p {
                break;
            }
            let last = squares.last().unwrap();
            let next = self.product(last, last);
            squares.push(next);
            i += 1;
        }

        // Now, for each 1 bit of p, multiply by the associated square power.
        let mut product = Expression::one();
        for (i, square) in enumerate(squares) {
            let b = (p >> i) & 1 != 0;
            if b {
                product = self.product(&product, square);
            }
        }
        product
    }

    /// 1 / x. Assumes x is non-zero. If x is zero, the resulting gadget will not be satisfiable.
    pub fn inverse<E: Borrow<Expression<F>>>(&mut self, x: E) -> Expression<F> {
        let x = x.borrow().clone();

        let x_inv = self.wire();
        self.assert_product(&x, Expression::from(x_inv), Expression::one());

        self.generator(
            x.dependencies(),
            move |values: &mut WireValues<F>| {
                let x_value = x.evaluate(values);
                let inverse_value = x_value.multiplicative_inverse();
                values.set(x_inv, inverse_value);
            },
        );

        x_inv.into()
    }

    /// x / y. Assumes y is non-zero. If y is zero, the resulting gadget will not be satisfiable.
    pub fn quotient<E1, E2>(&mut self, x: E1, y: E2) -> Expression<F>
        where E1: Borrow<Expression<F>>, E2: Borrow<Expression<F>> {
        let y_inv = self.inverse(y);
        self.product(x, y_inv)
    }

    /// x mod y.
    pub fn modulus<E1, E2>(&mut self, x: E1, y: E2) -> Expression<F>
        where E1: Borrow<Expression<F>>, E2: Borrow<Expression<F>> {
        // We will non-deterministically compute a quotient q and remainder r such that:
        //     y * q = x - r
        //     r < y

        let x = x.borrow();
        let y = y.borrow();

        let q = self.wire();
        let r = self.wire();
        self.assert_product(y, Expression::from(q), x - Expression::from(r));
        self.assert_lt(Expression::from(r), y);

        {
            let x = x.clone();
            let y = y.clone();
            self.generator(
                [x.dependencies(), y.dependencies()].concat(),
                move |values: &mut WireValues<F>| {
                    let x_value = x.evaluate(values);
                    let y_value = y.evaluate(values);
                    values.set(q, x_value.integer_division(&y_value));
                    values.set(r, x_value.integer_modulus(y_value));
                },
            );
        }

        r.into()
    }

    /// if x | y { 1 } else { 0 }.
    pub fn divides<E1, E2>(&mut self, x: E1, y: E2) -> BooleanExpression<F>
        where E1: Borrow<Expression<F>>, E2: Borrow<Expression<F>> {
        let m = self.modulus(y, x);
        self.zero(m)
    }
}

#[cfg(test)]
mod tests {
    use crate::expression::Expression;
    use crate::field::{Bn128, Element};
    use crate::gadget_builder::GadgetBuilder;
    use crate::test_util::{assert_eq_false, assert_eq_true};

    #[test]
    fn exp() {
        let mut builder = GadgetBuilder::<Bn128>::new();
        let x = builder.wire();
        let x_exp_0 = builder.exp(Expression::from(x), 0);
        let x_exp_1 = builder.exp(Expression::from(x), 1);
        let x_exp_2 = builder.exp(Expression::from(x), 2);
        let x_exp_3 = builder.exp(Expression::from(x), 3);
        let gadget = builder.build();

        let mut values = values!(x => 3u8.into());
        assert!(gadget.execute(&mut values));
        assert_eq!(Element::from(1u8), x_exp_0.evaluate(&values));
        assert_eq!(Element::from(3u8), x_exp_1.evaluate(&values));
        assert_eq!(Element::from(9u8), x_exp_2.evaluate(&values));
        assert_eq!(Element::from(27u8), x_exp_3.evaluate(&values));
    }

    #[test]
    #[should_panic]
    fn invert_zero() {
        let mut builder = GadgetBuilder::<Bn128>::new();
        let x = builder.wire();
        builder.inverse(Expression::from(x));
        let gadget = builder.build();

        let mut values = values!(x => 0u8.into());
        gadget.execute(&mut values);
    }

    #[test]
    fn divides() {
        let mut builder = GadgetBuilder::<Bn128>::new();
        let x = builder.wire();
        let y = builder.wire();
        let divides = builder.divides(Expression::from(x), Expression::from(y));
        let gadget = builder.build();

        let mut values_1_1 = values!(x => 1u8.into(), y => 1u8.into());
        assert!(gadget.execute(&mut values_1_1));
        assert_eq_true(&divides, &values_1_1);

        let mut values_3_6 = values!(x => 3u8.into(), y => 6u8.into());
        assert!(gadget.execute(&mut values_3_6));
        assert_eq_true(&divides, &values_3_6);

        let mut values_3_7 = values!(x => 3u8.into(), y => 7u8.into());
        assert!(gadget.execute(&mut values_3_7));
        assert_eq_false(&divides, &values_3_7);
    }
}
