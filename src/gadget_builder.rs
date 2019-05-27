use constraint::Constraint;
use field_element::FieldElement;
use gadget::Gadget;
use gadgets::split::split;
use linear_combination::LinearCombination;
use wire::Wire;
use wire_values::WireValues;
use witness_generator::WitnessGenerator;
use std::collections::HashMap;
use itertools::enumerate;

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

    /// Add a wire to the gadget. It will start with no generator and no associated constraints.
    pub fn wire(&mut self) -> Wire {
        let index = self.next_wire_index;
        self.next_wire_index += 1;
        Wire { index }
    }

    /// Add `n` wires to the gadget. They will start with no generator and no associated
    /// constraints.
    pub fn wires(&mut self, n: usize) -> Vec<Wire> {
        (0..n).map(|_i| self.wire()).collect()
    }

    /// Add a generator function for setting certain wire values.
    pub fn generator<T>(&mut self, dependencies: Vec<Wire>, generate: T)
        where T: Fn(&mut WireValues) + 'static {
        self.witness_generators.push(WitnessGenerator::new(dependencies, generate));
    }

    /// The product of two terms.
    pub fn product(&mut self, a: LinearCombination, b: LinearCombination) -> LinearCombination {
        if a == 1.into() {
            return b;
        }
        if b == 1.into() {
            return a;
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
                }
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

    /// The conjunction of two binary values. Assumes both inputs are binary, otherwise the result
    /// is undefined.
    pub fn and(&mut self, x: LinearCombination, y: LinearCombination) -> LinearCombination {
        self.product(x, y)
    }

    /// The disjunction of two binary values. Assumes both inputs are binary, otherwise the result
    /// is undefined.
    pub fn or(&mut self, x: LinearCombination, y: LinearCombination) -> LinearCombination {
        x.clone() + y.clone() - self.and(x, y)
    }

    /// The exclusive disjunction of two binary values. Assumes both inputs are binary, otherwise
    /// the result is undefined.
    pub fn xor(&mut self, x: LinearCombination, y: LinearCombination) -> LinearCombination {
        x.clone() + y.clone() - self.and(x, y) * 2u128
    }

    /// if x == y { 1 } else { 0 }.
    pub fn equal(&mut self, x: LinearCombination, y: LinearCombination) -> LinearCombination {
        self.zero(x - y)
    }

    /// if x == 0 { 1 } else { 0 }.
    pub fn zero(&mut self, x: LinearCombination) -> LinearCombination {
        LinearCombination::one() - self.nonzero(x)
    }

    /// if x != 0 { 1 } else { 0 }.
    pub fn nonzero(&mut self, x: LinearCombination) -> LinearCombination {
        // See the Pinocchio paper for an explanation.
        let (y, m) = (self.wire(), self.wire());
        self.assert_product(x.clone(), m.into(), y.into());
        self.assert_product(LinearCombination::one() - y.into(), x.clone(), 0.into());

        {
            let y = y.clone();
            self.generator(
                x.wires(),
                move |values: &mut WireValues| {
                    let x_value = x.evaluate(values);
                    let y_value: FieldElement = if x_value.is_nonzero() {
                        1.into()
                    } else {
                        0.into()
                    };
                    let m_value: FieldElement = if x_value.is_nonzero() {
                        y_value.clone() / x_value
                    } else {
                        // The value of m doesn't matter if x = 0.
                        42.into()
                    };
                    values.set(m, m_value);
                    values.set(y, y_value);
                },
            );
        }

        y.into()
    }

    /// x <= y
    pub fn le(&mut self, x: LinearCombination, y: LinearCombination) -> LinearCombination {
        // TODO: This is a super naive implementation. Should only need 1 constraint per compared
        // bit, or less using a non-deterministic method like jsnark.
        let bits = FieldElement::max_bits();
        let x_bits = split(self, x, bits);
        let y_bits = split(self, y, bits);

        let mut status = LinearCombination::one();
        for i in 0..bits {
            let x_i: LinearCombination = x_bits[i].into();
            let y_i: LinearCombination = y_bits[i].into();
            let delta_i = x_i - y_i;
            let lt_i = self.equal(delta_i.clone(), LinearCombination::neg_one());
            let eq_i = self.zero(delta_i);
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

    /// Split `x` into `bits` bit wires. Assumes `x < 2^bits`.
    pub fn split(&mut self, x: LinearCombination, bits: usize) -> Vec<Wire> {
        split(self, x, bits)
    }

    /// Join a vector of bit wires into the field element it encodes.
    pub fn join(&mut self, bits: Vec<Wire>) -> LinearCombination {
        let mut coefficients = HashMap::new();
        for (i, bit) in enumerate(bits) {
            coefficients.insert(bit, FieldElement::one() << i);
        }
        LinearCombination::new(coefficients)
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
    fn assert_binary_0_1() {
        let mut builder = GadgetBuilder::new();
        let x = builder.wire();
        builder.assert_binary(x.into());
        let gadget = builder.build();

        // With x = 0, the constraint should be satisfied.
        let mut values0 = wire_values!(x => 0.into());
        assert!(gadget.execute(&mut values0));

        // With x = 1, the constraint should be satisfied.
        let mut values1 = wire_values!(x => 1.into());
        assert!(gadget.execute(&mut values1));
    }

    #[test]
    fn assert_binary_2() {
        let mut builder = GadgetBuilder::new();
        let x = builder.wire();
        builder.assert_binary(x.into());
        let gadget = builder.build();

        // With x = 2, the constraint should NOT be satisfied.
        let mut values2 = wire_values!(x => 2.into());
        assert!(!gadget.execute(&mut values2));
    }

    #[test]
    fn and() {
        let mut builder = GadgetBuilder::new();
        let (x, y) = (builder.wire(), builder.wire());
        let and = builder.and(x.into(), y.into());
        let gadget = builder.build();

        let mut values00 = wire_values!(x => 0.into(), y => 0.into());
        assert!(gadget.execute(&mut values00));
        assert_eq!(FieldElement::zero(), and.evaluate(&values00));

        let mut values01 = wire_values!(x => 0.into(), y => 1.into());
        assert!(gadget.execute(&mut values01));
        assert_eq!(FieldElement::zero(), and.evaluate(&values01));

        let mut values10 = wire_values!(x => 1.into(), y => 0.into());
        assert!(gadget.execute(&mut values10));
        assert_eq!(FieldElement::zero(), and.evaluate(&values10));

        let mut values11 = wire_values!(x => 1.into(), y => 1.into());
        assert!(gadget.execute(&mut values11));
        assert_eq!(FieldElement::one(), and.evaluate(&values11));
    }

    #[test]
    fn or() {
        let mut builder = GadgetBuilder::new();
        let (x, y) = (builder.wire(), builder.wire());
        let or = builder.or(x.into(), y.into());
        let gadget = builder.build();

        let mut values00 = wire_values!(x => 0.into(), y => 0.into());
        assert!(gadget.execute(&mut values00));
        assert_eq!(FieldElement::zero(), or.evaluate(&values00));

        let mut values01 = wire_values!(x => 0.into(), y => 1.into());
        assert!(gadget.execute(&mut values01));
        assert_eq!(FieldElement::one(), or.evaluate(&values01));

        let mut values10 = wire_values!(x => 1.into(), y => 0.into());
        assert!(gadget.execute(&mut values10));
        assert_eq!(FieldElement::one(), or.evaluate(&values10));

        let mut values11 = wire_values!(x => 1.into(), y => 1.into());
        assert!(gadget.execute(&mut values11));
        assert_eq!(FieldElement::one(), or.evaluate(&values11));
    }

    #[test]
    fn xor() {
        let mut builder = GadgetBuilder::new();
        let (x, y) = (builder.wire(), builder.wire());
        let xor = builder.xor(x.into(), y.into());
        let gadget = builder.build();

        let mut values00 = wire_values!(x => 0.into(), y => 0.into());
        assert!(gadget.execute(&mut values00));
        assert_eq!(FieldElement::zero(), xor.evaluate(&values00));

        let mut values01 = wire_values!(x => 0.into(), y => 1.into());
        assert!(gadget.execute(&mut values01));
        assert_eq!(FieldElement::one(), xor.evaluate(&values01));

        let mut values10 = wire_values!(x => 1.into(), y => 0.into());
        assert!(gadget.execute(&mut values10));
        assert_eq!(FieldElement::one(), xor.evaluate(&values10));

        let mut values11 = wire_values!(x => 1.into(), y => 1.into());
        assert!(gadget.execute(&mut values11));
        assert_eq!(FieldElement::zero(), xor.evaluate(&values11));
    }

    #[test]
    fn equal() {
        let mut builder = GadgetBuilder::new();
        let (x, y) = (builder.wire(), builder.wire());
        let equal = builder.equal(x.into(), y.into());
        let gadget = builder.build();

        let mut values_7_7 = wire_values!(x => 7.into(), y => 7.into());
        assert!(gadget.execute(&mut values_7_7));
        assert_eq!(FieldElement::one(), equal.evaluate(&values_7_7));

        let mut values_6_7 = wire_values!(x => 6.into(), y => 7.into());
        assert!(gadget.execute(&mut values_6_7));
        assert_eq!(FieldElement::zero(), equal.evaluate(&values_6_7));

        let mut values_7_13 = wire_values!(x => 7.into(), y => 13.into());
        assert!(gadget.execute(&mut values_7_13));
        assert_eq!(FieldElement::zero(), equal.evaluate(&values_7_13));
    }

    #[test]
    fn assert_le_equal() {
        let mut builder = GadgetBuilder::new();
        let (x, y) = (builder.wire(), builder.wire());
        builder.assert_le(x.into(), y.into());
        let gadget = builder.build();

        let mut values = wire_values!(x => 42.into(), y => 42.into());
        assert!(gadget.execute(&mut values));
    }

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
