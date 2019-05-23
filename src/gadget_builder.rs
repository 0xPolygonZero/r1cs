use constraint::Constraint;
use wire::Wire;
use linear_combination::LinearCombination;
use witness_generator::WitnessGenerator;
use gadget::Gadget;

pub struct GadgetBuilder {
    next_wire_index: u32,
    nonzero_element: LinearCombination,
    constraints: Vec<Constraint>,
    generators: Vec<WitnessGenerator>,
}

impl GadgetBuilder {
    fn new() -> Self {
        GadgetBuilder {
            next_wire_index: 1,
            nonzero_element: Wire::ONE.into(),
            constraints: Vec::new(),
            generators: Vec::new(),
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
        self.generators.push(generator);
    }

    /// Return the product of zero or more terms.
    pub fn product(&mut self, terms: &[LinearCombination]) -> LinearCombination {
        // As an optimization, filter out any 1 terms.
        let filtered_terms: Vec<&LinearCombination> = terms.iter()
            .filter(|t| **t != LinearCombination::one())
            .collect();

        if filtered_terms.is_empty() {
            LinearCombination::one()
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

    pub fn assert_nonequal(&mut self, x: LinearCombination, y: LinearCombination) {
        let difference = x - y;
        self.assert_nonzero(difference)
    }

    pub fn assert_nonzero(&mut self, x: LinearCombination) {
        let terms = [self.nonzero_element.clone(), x];
        self.nonzero_element = self.product(&terms);
    }

    pub fn assert_le(&mut self, a: LinearCombination, b: LinearCombination) {
        unimplemented!("TODO")
    }

    fn build(&self) -> Gadget {
        unimplemented!("TODO")
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
