use constraint::Constraint;
use wire::Wire;
use linear_combination::LinearCombination;
use witness_generator::WitnessGenerator;
use gadget::Gadget;
use gadgets::split::split;
use field_element::FieldElement;

pub struct GadgetBuilder {
    next_wire_index: u32,
    nonzero_element: LinearCombination,
    constraints: Vec<Constraint>,
    witness_generators: Vec<WitnessGenerator>,
}

impl GadgetBuilder {
    pub fn new() -> Self {
        GadgetBuilder {
            next_wire_index: 1,
            nonzero_element: Wire::ONE.into(),
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

    pub fn generator(&mut self, generator: WitnessGenerator) {
        self.witness_generators.push(generator);
    }

    /// Return the product of zero or more terms.
    pub fn product(&mut self, terms: &[LinearCombination]) -> LinearCombination {
        // As an optimization, filter out any 1 terms.
        let filtered_terms: Vec<&LinearCombination> = terms.iter()
            .filter(|t| **t != 1.into())
            .collect();

        if filtered_terms.is_empty() {
            1.into()
        } else if filtered_terms.len() == 1 {
            filtered_terms[0].clone()
        } else {
            unimplemented!("TODO")
        }
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
        self.constraints.push(Constraint { a: x, b: 1.into(), c: y })
    }

    pub fn assert_nonequal(&mut self, x: LinearCombination, y: LinearCombination) {
        let difference = x - y;
        self.assert_nonzero(difference)
    }

    pub fn assert_nonzero(&mut self, x: LinearCombination) {
        let terms = [self.nonzero_element.clone(), x];
        self.nonzero_element = self.product(&terms);
    }

    pub fn assert_le(&mut self, a: LinearCombination, b: LinearCombination) {
        split(self, a, FieldElement::bits());
        unimplemented!("TODO")
    }

    pub fn build(self) -> Gadget {
        let mut generated_constraints = self.constraints;

        // Constrain nonzero_element to be non-zero.
        // TODO: implementation pending

        Gadget { constraints: generated_constraints, witness_generators: self.witness_generators }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn constraint_not_satisfied() {
        // TODO
    }

    #[test]
    fn missing_generator() {
        // TODO
    }
}
