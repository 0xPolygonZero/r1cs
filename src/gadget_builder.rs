use constraint::Constraint;
use field_element::FieldElement;
use gadget::Gadget;
use gadgets::split::split;
use linear_combination::LinearCombination;
use wire::Wire;
use wire_values::WireValues;
use witness_generator::WitnessGenerator;
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

    /// x < y
    pub fn lt(&mut self, x: LinearCombination, y: LinearCombination) -> LinearCombination {
        self.cmp(x, y, true, true)
    }

    /// x <= y
    pub fn le(&mut self, x: LinearCombination, y: LinearCombination) -> LinearCombination {
        self.cmp(x, y, true, false)
    }

    /// x > y
    pub fn gt(&mut self, x: LinearCombination, y: LinearCombination) -> LinearCombination {
        self.cmp(x, y, false, true)
    }

    /// x >= y
    pub fn ge(&mut self, x: LinearCombination, y: LinearCombination) -> LinearCombination {
        self.cmp(x, y, false, false)
    }

    fn cmp(&mut self, x: LinearCombination, y: LinearCombination,
           less: bool, strict: bool) -> LinearCombination {
        let bits = FieldElement::max_bits();
        let x_bits = self.split(x.into(), bits);
        let y_bits = self.split(y.into(), bits);
        self.cmp_binary(x_bits, y_bits, less, strict)
    }

    fn cmp_binary(&mut self, x_bits: Vec<Wire>, y_bits: Vec<Wire>,
                  less: bool, strict: bool) -> LinearCombination {
        assert_eq!(x_bits.len(), y_bits.len());

        // We will chunk both bit vectors, then have the prover supply a mask which identifies the
        // first pair of chunks to differ. Credit to Ahmed Kosba who described this technique.
        let chunk_bits = GadgetBuilder::cmp_chunk_bits();
        let x_chunks: Vec<LinearCombination> = x_bits.chunks(chunk_bits)
            .map(LinearCombination::join_bits)
            .collect();
        let y_chunks: Vec<LinearCombination> = y_bits.chunks(chunk_bits)
            .map(LinearCombination::join_bits)
            .collect();
        let chunks = x_chunks.len();

        // Create a mask bit for each chunk index. masks[i] must equal 1 iff i is the first index
        // where the chunks differ, otherwise 0. If no chunks differ, all masks must equal 0.
        let mask = self.wires(chunks);
        // Each mask must equal 0 or 1.
        for &m in &mask {
            self.assert_binary(m.into());
        }
        // The sum of all masks must equal 0 or 1, so that at most one mask can equal 1.
        let diff_exists = LinearCombination::sum(&mask);
        self.assert_binary(diff_exists.clone());

        {
            let x_chunks = x_chunks.clone();
            let y_chunks = y_chunks.clone();
            let mask = mask.clone();
            self.generator(
                [x_bits, y_bits].concat(),
                move |values: &mut WireValues| {
                    let mut seen_diff: bool = false;
                    for (i, &mask_bit) in enumerate(&mask).rev() {
                        let x_chunk_value = x_chunks[i].evaluate(values);
                        let y_chunk_value = y_chunks[i].evaluate(values);
                        let diff = x_chunk_value != y_chunk_value;
                        let mask_bit_value = diff && !seen_diff;
                        seen_diff |= diff;
                        values.set(mask_bit, mask_bit_value.into());
                    }
                }
            );
        }

        // Get the chunks that differ (or zero if none) by computing the dot product of the mask
        // vector with x_chunks and y_chunks, respectively.
        let mut x_diff_chunk = LinearCombination::zero();
        let mut y_diff_chunk = LinearCombination::zero();
        for i in 0..chunks {
            x_diff_chunk += self.product(mask[i].into(), x_chunks[i].clone());
            y_diff_chunk += self.product(mask[i].into(), y_chunks[i].clone());
        }

        // Verify that any more significant pairs of chunks are equal.
        // diff_seen tracks whether a mask bit of 1 has been observed for a less significant bit.
        let mut diff_seen: LinearCombination = mask[0].into();
        for i in 1..chunks {
            // If diff_seen = 1, we require that x_chunk = y_chunk.
            // Equivalently, we require that diff_seen * x_chunk = diff_seen * y_chunk.
            let x_if_diff_seen = self.product(diff_seen.clone(), x_chunks[i].clone());
            self.assert_product(diff_seen.clone(), y_chunks[i].clone(), x_if_diff_seen);

            diff_seen += mask[i].into();
        }

        // If the mask has a 1 bit, then the corresponding pair of chunks must differ. In other
        // words, their difference must be non-zero.
        let nonzero = self._if(diff_exists,
                               x_diff_chunk.clone() - y_diff_chunk.clone(),
                               // The mask is 0, so just assert that 42 (arbitrary) is non-zero.
                               42.into());
        self.assert_nonzero(nonzero);

        // Finally, apply a different comparison algorithm to the (small) differing chunks.
        self.cmp_subtractive(x_diff_chunk, y_diff_chunk, less, strict, chunk_bits)
    }

    fn cmp_subtractive(&mut self, x: LinearCombination, y: LinearCombination,
                   less: bool, strict: bool, bits: usize) -> LinearCombination {
        // An as example, assume less=false and strict=false. In that case, we compute
        //     2^bits + x - y
        // And check the most significant bit, i.e., the one with index `bits`.
        // x >= y iff that bit is set. The other cases are similar.
        let base = LinearCombination::from(
            (FieldElement::one() << bits) - FieldElement::from(strict));
        let z = base + if less { y - x } else { x - y };
        self.split(z, bits + 1)[bits].into()
    }

    /// The number of constraints used by `cmp_binary`, given a certain chunk size.
    fn cmp_constraints(chunk_bits: usize) -> usize {
        let chunks = (FieldElement::max_bits() + chunk_bits - 1) / chunk_bits;
        5 * chunks + 6 + chunk_bits
    }

    /// The optimal number of bits per chunk for the comparison algorithm used in `cmp_binary`.
    fn cmp_chunk_bits() -> usize {
        let mut best_chunk_bits = 1;
        let mut best_constraints = GadgetBuilder::cmp_constraints(1);
        for chunk_bits in 2..FieldElement::max_bits() {
            let constraints = GadgetBuilder::cmp_constraints(chunk_bits);
            if constraints < best_constraints {
                best_chunk_bits = chunk_bits;
                best_constraints = constraints;
            }
        }
        best_chunk_bits
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

    /// Assert that x == 0.
    pub fn assert_false(&mut self, x: LinearCombination) {
        self.assert_equal(x, 0.into());
    }

    /// Split `x` into `bits` bit wires. Assumes `x < 2^bits`.
    pub fn split(&mut self, x: LinearCombination, bits: usize) -> Vec<Wire> {
        split(self, x, bits)
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
    use linear_combination::LinearCombination;
    use std::borrow::Borrow;

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
    fn comparisons() {
        let mut builder = GadgetBuilder::new();
        let (x, y) = (builder.wire(), builder.wire());
        let lt = builder.lt(x.into(), y.into());
        let le = builder.le(x.into(), y.into());
        let gt = builder.gt(x.into(), y.into());
        let ge = builder.ge(x.into(), y.into());
        let gadget = builder.build();

        let mut values_42_63 = wire_values!(x => 42.into(), y => 63.into());
        assert!(gadget.execute(&mut values_42_63));
        assert_1(&lt, &values_42_63);
        assert_1(&le, &values_42_63);
        assert_0(&gt, &values_42_63);
        assert_0(&ge, &values_42_63);

        let mut values_42_42 = wire_values!(x => 42.into(), y => 42.into());
        assert!(gadget.execute(&mut values_42_42));
        assert_0(&lt, &values_42_42);
        assert_1(&le, &values_42_42);
        assert_0(&gt, &values_42_42);
        assert_1(&ge, &values_42_42);

        let mut values_42_41 = wire_values!(x => 42.into(), y => 41.into());
        assert!(gadget.execute(&mut values_42_41));
        assert_0(&lt, &values_42_41);
        assert_0(&le, &values_42_41);
        assert_1(&gt, &values_42_41);
        assert_1(&ge, &values_42_41);

        // This is a white box sort of test. Since the implementation is based on chunks of roughly
        // 32 bits each, all the numbers in the preceding tests will fit into the least significant
        // chunk. So let's try some larger numbers. In particular, let's have x < y but have the
        // least significant chunk of y exceed that of x, to make sure the more significant chunk
        // takes precedence.
        let mut values_large_lt = wire_values!(
            x => FieldElement::from(1u128 << 80 | 1u128),
            y => FieldElement::from(1u128 << 81));
        assert!(gadget.execute(&mut values_large_lt));
        assert_1(&lt, &values_large_lt);
        assert_1(&le, &values_large_lt);
        assert_0(&gt, &values_large_lt);
        assert_0(&ge, &values_large_lt);
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

    fn assert_1<T: Borrow<LinearCombination>>(x: T, values: &WireValues) {
        assert_eq!(FieldElement::one(), x.borrow().evaluate(values));
    }

    fn assert_0<T: Borrow<LinearCombination>>(x: T, values: &WireValues) {
        assert_eq!(FieldElement::zero(), x.borrow().evaluate(values));
    }
}
