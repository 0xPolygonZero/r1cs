use gadget_builder::GadgetBuilder;
use linear_combination::LinearCombination;
use wire_values::WireValues;
use itertools::enumerate;

impl GadgetBuilder {
    /// x * y
    pub fn product(&mut self, x: LinearCombination, y: LinearCombination) -> LinearCombination {
        if let Some(c) = x.as_constant() {
            return y * c;
        }
        if let Some(c) = y.as_constant() {
            return x * c;
        }

        let product = self.wire();
        self.assert_product(x.clone(), y.clone(), product.into());

        {
            let product = product.clone();
            self.generator(
                [x.wires(), y.wires()].concat(),
                move |values: &mut WireValues| {
                    let product_value = x.evaluate(values) * y.evaluate(values);
                    values.set(product, product_value);
                },
            );
        }

        product.into()
    }

    /// x^p for a constant p.
    pub fn exp(&mut self, x: LinearCombination, p: usize) -> LinearCombination {
        // This is exponentiation by squaring. Generate a list squares where squares[i] = x^(2^i).
        let mut squares = vec![x];
        let mut i = 1;
        loop {
            let q = 1 << i;
            if q > p {
                break;
            }
            let square = squares[i - 1].clone();
            squares.push(self.product(square.clone(), square));
            i += 1;
        }

        // Now, for each 1 bit of p, multiply by the associated square power.
        let mut product = LinearCombination::one();
        for (i, square) in enumerate(squares) {
            let b = (p >> i) & 1 != 0;
            if b {
                product = self.product(product.clone(), square);
            }
        }
        product
    }

    /// 1 / x. Assumes x is non-zero. If x is zero, the resulting gadget will not be satisfiable.
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

    /// x / y. Assumes y is non-zero. If y is zero, the resulting gadget will not be satisfiable.
    pub fn quotient(&mut self, x: LinearCombination, y: LinearCombination) -> LinearCombination {
        let y_inv = self.inverse(y);
        self.product(x, y_inv)
    }

    /// x mod y.
    pub fn modulus(&mut self, x: LinearCombination, y: LinearCombination) -> LinearCombination {
        // We will non-deterministically compute a quotient q and remainder r such that:
        //     y * q = x - r
        //     r < y

        let q = self.wire();
        let r = self.wire();
        self.assert_product(y.clone(), q.into(), x.clone() - LinearCombination::from(r));
        self.assert_lt(r.into(), y.clone());

        self.generator(
            [x.wires(), y.wires()].concat(),
            move |values: &mut WireValues| {
                let x_value = x.evaluate(values);
                let y_value = y.evaluate(values);
                values.set(q, x_value.integer_division(y_value.clone()));
                values.set(r, x_value.integer_modulus(y_value));
            }
        );

        r.into()
    }

    /// if x | y { 1 } else { 0 }.
    pub fn divides(&mut self, x: LinearCombination, y: LinearCombination) -> LinearCombination {
        let m = self.modulus(y, x);
        self.zero(m)
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