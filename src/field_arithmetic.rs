//! This module extends GadgetBuilder with native field arithmetic methods.

use crate::expression::{BooleanExpression, Expression};
use crate::field::{Element, Field};
use crate::gadget_builder::GadgetBuilder;
use crate::wire_values::WireValues;

impl<F: Field> GadgetBuilder<F> {
    /// The product of two `Expression`s `x` and `y`, i.e. `x * y`.
    pub fn product(&mut self, x: &Expression<F>, y: &Expression<F>) -> Expression<F> {
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

    /// `x^p` for a constant `p`.
    pub fn exponentiation(&mut self, x: &Expression<F>, p: &Element<F>) -> Expression<F> {
        // This is exponentiation by squaring. For each 1 bit of p, multiply by the associated
        // square power.
        let mut product_exp = Expression::one();
        let mut last_square = Expression::zero();

        for i in 0..p.bits() {
            let square = if i == 0 {
                x.clone()
            } else {
                self.product(&last_square, &last_square)
            };

            if p.bit(i) {
                product_exp = self.product(&product_exp, &square);
            }

            last_square = square;
        }
        product_exp
    }

    /// Returns `1 / x`, assuming `x` is non-zero. If `x` is zero, the gadget will not be
    /// satisfiable.
    pub fn inverse(&mut self, x: &Expression<F>) -> Expression<F> {
        let x_inv = self.wire();
        self.assert_product(x, &Expression::from(x_inv), &Expression::one());

        let x = x.clone();
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

    /// Like `inverse`, except that zero is mapped to itself rather than being prohibited.
    pub fn inverse_or_zero(&mut self, x: &Expression<F>) -> Expression<F> {
        let x_inv_or_zero = self.wire();
        let nonzero = self.nonzero(x);
        self.assert_product(x, &Expression::from(x_inv_or_zero), nonzero.expression());

        let x = x.clone();
        self.generator(
            x.dependencies(),
            move |values: &mut WireValues<F>| {
                let x_value = x.evaluate(values);
                values.set(x_inv_or_zero, x_value.multiplicative_inverse_or_zero());
            },
        );

        x_inv_or_zero.into()
    }

    /// Returns `x / y`, assuming `y` is non-zero. If `y` is zero, the gadget will not be
    /// satisfiable.
    pub fn quotient(&mut self, x: &Expression<F>, y: &Expression<F>) -> Expression<F> {
        let y_inv = self.inverse(y);
        self.product(x, &y_inv)
    }

    /// Returns `x mod y`, assuming `y` is non-zero. If `y` is zero, the gadget will not be
    /// satisfiable.
    pub fn modulus(&mut self, x: &Expression<F>, y: &Expression<F>) -> Expression<F> {
        // We will non-deterministically compute a quotient q and remainder r such that:
        //     y * q = x - r
        //     r < y

        let q = self.wire();
        let r = self.wire();
        self.assert_product(y, &Expression::from(q), &(x - Expression::from(r)));
        self.assert_lt(&Expression::from(r), y);

        {
            let x = x.clone();
            let y = y.clone();
            self.generator(
                [x.dependencies(), y.dependencies()].concat(),
                move |values: &mut WireValues<F>| {
                    let x_value = x.evaluate(values);
                    let y_value = y.evaluate(values);
                    values.set(q, x_value.integer_division(&y_value));
                    values.set(r, x_value.integer_modulus(&y_value));
                },
            );
        }

        r.into()
    }

    /// Returns whether `x` divides `y`, i.e. `x | y`.
    pub fn divides(&mut self, x: &Expression<F>, y: &Expression<F>) -> BooleanExpression<F> {
        let m = self.modulus(y, x);
        self.zero(&m)
    }
}

#[cfg(test)]
mod tests {
    use crate::expression::Expression;
    use crate::field::Element;
    use crate::gadget_builder::GadgetBuilder;
    use crate::test_util::{assert_eq_false, assert_eq_true, F257};

    #[test]
    fn exp() {
        let mut builder = GadgetBuilder::<F257>::new();
        let x = builder.wire();
        let x_exp_0 = builder.exponentiation(&Expression::from(x), &Element::from(0u8));
        let x_exp_1 = builder.exponentiation(&Expression::from(x), &Element::from(1u8));
        let x_exp_2 = builder.exponentiation(&Expression::from(x), &Element::from(2u8));
        let x_exp_3 = builder.exponentiation(&Expression::from(x), &Element::from(3u8));
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
        let mut builder = GadgetBuilder::<F257>::new();
        let x = builder.wire();
        builder.inverse(&Expression::from(x));
        let gadget = builder.build();

        let mut values = values!(x => 0u8.into());
        gadget.execute(&mut values);
    }

    #[test]
    fn divides() {
        let mut builder = GadgetBuilder::<F257>::new();
        let x = builder.wire();
        let y = builder.wire();
        let divides = builder.divides(&Expression::from(x), &Expression::from(y));
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
