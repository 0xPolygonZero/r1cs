use gadget_builder::GadgetBuilder;
use linear_combination::LinearCombination;
use wire_values::WireValues;

impl GadgetBuilder {
    /// The product of two terms.
    pub fn product(&mut self, a: LinearCombination, b: LinearCombination) -> LinearCombination {
        if let Some(c) = a.as_constant() {
            return b * c;
        }
        if let Some(c) = b.as_constant() {
            return a * c;
        }

        let product = self.wire();
        self.assert_product(a.clone(), b.clone(), product.into());

        {
            let product = product.clone();
            self.generator(
                [a.wires(), b.wires()].concat(),
                move |values: &mut WireValues| {
                    let product_value = a.evaluate(values) * b.evaluate(values);
                    values.set(product, product_value);
                },
            );
        }

        product.into()
    }

    /// 1 / x. Assumes x is non-zero. If x is zero, the gadget will not be satisfiable.
    pub fn inverse(&mut self, x: LinearCombination) -> LinearCombination {
        let x_inv = self.wire();

        {
            let x = x.clone();
            self.generator(
                x.wires(),
                move |values: &mut WireValues| {
                    let x_value = x.evaluate(values);
                    let inverse_value = x_value.multiplicative_inverse();
                    values.set(x_inv, inverse_value);
                },
            );
        }

        self.assert_product(x, x_inv.into(), 1.into());
        x_inv.into()
    }

    pub fn quotient(&mut self, x: LinearCombination, y: LinearCombination) -> LinearCombination {
        let y_inv = self.inverse(y);
        self.product(x, y_inv)
    }
}

#[cfg(test)]
mod tests {
    use gadget_builder::GadgetBuilder;

    #[test]
    #[should_panic]
    fn invert_zero() {
        let mut builder = GadgetBuilder::new();
        let x = builder.wire();
        builder.inverse(x.into());
        let gadget = builder.build();

        let mut values = wire_values!(x => 0.into());
        gadget.execute(&mut values);
    }
}