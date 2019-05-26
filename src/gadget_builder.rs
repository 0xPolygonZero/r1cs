use constraint::Constraint;
use field_element::FieldElement;
use gadget::Gadget;
use gadgets::split::split;
use linear_combination::LinearCombination;
use wire::Wire;
use wire_values::WireValues;
use witness_generator::WitnessGenerator;

pub struct GadgetBuilder {
    next_wire_index: u32,
    constraints: Vec<Constraint>,
    witness_generators: Vec<WitnessGenerator>,
}

impl GadgetBuilder {
    pub fn new() -> Self {
        GadgetBuilder {
            next_wire_index: 1,
            constraints: Vec::new(),
            witness_generators: Vec::new(),
        }
    }

    pub fn wire(&mut self) -> Wire {
        let index = self.next_wire_index;
        self.next_wire_index += 1;
        Wire { index: index }
    }

    pub fn wires(&mut self, n: usize) -> Vec<Wire> {
        (0..n).map(|_i| self.wire()).collect()
    }

    // TODO: Take the input list and generate function directly.
    pub fn generator<T>(&mut self, inputs: Vec<Wire>, generate: T)
        where T: Fn(&mut WireValues) + 'static {
        self.witness_generators.push(WitnessGenerator::new(inputs, generate));
    }

    /// The product of two terms.
    pub fn product(&mut self, a: LinearCombination, b: LinearCombination) -> LinearCombination {
        if a == 1.into() {
            return b;
        }
        if b == 1.into() {
            return a;
        }

        let product: LinearCombination = self.wire().into();
        self.assert_product(a, b, product.clone());
        product
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

    pub fn and(&mut self, a: LinearCombination, b: LinearCombination) -> LinearCombination {
        self.product(a, b)
    }

    pub fn or(&mut self, a: LinearCombination, b: LinearCombination) -> LinearCombination {
        a.clone() + b.clone() - self.and(a, b)
    }

    /// if x == 0 { 1 } else { 0 }.
    pub fn equal(&mut self, a: LinearCombination, b: LinearCombination) -> LinearCombination {
        self.equals_zero(a - b)
    }

    /// if x == 0 { 1 } else { 0 }.
    pub fn equals_zero(&mut self, x: LinearCombination) -> LinearCombination {
        // We will non-deterministically compute three wires:
        // - z := if x == 0 { 1 } else { 0 }
        // - y := if x == 0 { 42 } else { 1 / x } (42 being an arbitrary non-zero element)
        // - y_inv := 1 / y
        // And then add three constraints:
        // - z must be binary
        // - y must be non-zero (in other words, it must have an inverse)
        // - x * y = 1 - z
        // If x == 0, then the third constraint requires that z == 1.
        // If x != 0, then the first constraint implies that z is in [0, 1]. If z == 1, then the
        // third constraint would require that y == 0, which the second constraint prohibits.
        // Ergo, z must always equal (x == 0) in order for the constraints to be satisfied.
        let (y, z) = (self.wire(), self.wire());

        {
            let x = x.clone();
            self.generator(
                x.wires(),
                move |values: &mut WireValues| {
                    let x_value = x.evaluate(values);
                    let z_value = if x_value == 0.into() { 1.into() } else { 0.into() };
                    let y_value = if x_value == 0.into() { 42.into() } else { x_value.multiplicative_inverse() };
                    values.set(z, z_value);
                    values.set(y, y_value);
                },
            );
        }

        self.assert_binary(z.into());
        self.assert_nonzero(y.into());
        self.assert_product(x.into(), y.into(), LinearCombination::one() - z.into());

        z.into()
    }

    pub fn le(&mut self, a: LinearCombination, b: LinearCombination) -> LinearCombination {
        // TODO: This is a super naive implementation. Should only need 1 constraint per compared
        // bit, or less using a non-deterministic method like jsnark.
        let bits = FieldElement::bits();
        let a_bits = split(self, a, bits);
        let b_bits = split(self, b, bits);
        let mut status = LinearCombination::one();
        for i in 0..bits {
            let a_i: LinearCombination = a_bits[i].into();
            let b_i: LinearCombination = b_bits[i].into();
            let delta_i = a_i - b_i;
            let lt_i = self.equal(delta_i.clone(), LinearCombination::neg_one());
            let eq_i = self.equals_zero(delta_i);
            let carry = self.product(eq_i, status);
            status = self.or(lt_i, carry);
        }
        status
    }

    /// if c { x } else { y }. Assumes c is binary.
    pub fn _if(&mut self, c: LinearCombination,
               x: LinearCombination, y: LinearCombination) -> LinearCombination {
        let not_c = LinearCombination::one() - c.clone();
        self.product(c, x) + self.product(not_c, y)
    }

    pub fn assert_product(&mut self, a: LinearCombination, b: LinearCombination,
                          c: LinearCombination) {
        self.constraints.push(Constraint { a, b, c });
    }

    /// Assert that the given quantity is in [0, 1].
    pub fn assert_binary(&mut self, a: LinearCombination) {
        self.assert_product(a.clone(), a - 1.into(), 0.into());
    }

    pub fn assert_equal(&mut self, x: LinearCombination, y: LinearCombination) {
        self.constraints.push(Constraint { a: x, b: 1.into(), c: y });
    }

    pub fn assert_nonequal(&mut self, x: LinearCombination, y: LinearCombination) {
        let difference = x - y;
        self.assert_nonzero(difference);
    }

    pub fn assert_zero(&mut self, x: LinearCombination) {
        self.assert_equal(x, 0.into());
    }

    pub fn assert_nonzero(&mut self, x: LinearCombination) {
        // A field element is non-zero iff it has a multiplicative inverse.
        // We don't care what the inverse is, but calling inverse(x) will require that it exists.
        self.inverse(x);
    }

    /// Assert that x == 1.
    pub fn assert_true(&mut self, x: LinearCombination) {
        self.assert_equal(x, 1.into());
    }

    pub fn assert_le(&mut self, x: LinearCombination, y: LinearCombination) {
        let le = self.le(x, y);
        self.assert_true(le);
    }

    pub fn build(self) -> Gadget {
        Gadget {
            constraints: self.constraints,
            witness_generators: self.witness_generators,
        }
    }
}

#[cfg(test)]
mod tests {
    use field_element::FieldElement;
    use gadget_builder::GadgetBuilder;
    use wire_values::WireValues;

    #[test]
    fn equal() {
        let mut builder = GadgetBuilder::new();
        let (x, y) = (builder.wire(), builder.wire());
        let equal = builder.equal(x.into(), y.into());
        let gadget = builder.build();

        let mut values = WireValues::new();
        values.set(x, 42.into());
        values.set(y, 42.into());

        let constraints_satisfied = gadget.execute(&mut values);
        assert!(constraints_satisfied);
        assert_eq!(FieldElement::one(), equal.evaluate(&values));
    }

    #[test]
    fn assert_le_equal() {
        let mut builder = GadgetBuilder::new();
        let (x, y) = (builder.wire(), builder.wire());
        builder.assert_le(x.into(), y.into());
        let gadget = builder.build();

        let mut values = WireValues::new();
        values.set(x, 42.into());
        values.set(y, 42.into());

        let constraints_satisfied = gadget.execute(&mut values);
        assert!(constraints_satisfied);
    }

    #[test]
    #[should_panic]
    fn invert_zero() {
        let mut builder = GadgetBuilder::new();
        let x = builder.wire();
        builder.inverse(x.into());
        let gadget = builder.build();

        let mut values = WireValues::new();
        values.set(x, 0.into());
        gadget.execute(&mut values);
    }
}
