use constraint::Constraint;
use wire_values::WireValues;
use witness_generator::WitnessGenerator;

pub struct Gadget {
    pub constraints: Vec<Constraint>,
    pub witness_generators: Vec<WitnessGenerator>,
}

impl Gadget {
    /// The number of constraints in this gadget.
    pub fn size(&self) -> usize {
        self.constraints.len()
    }

    /// Execute the gadget, and return whether all constraints were satisfied.
    pub fn execute(&self, wire_values: &mut WireValues) -> bool {
        let mut pending_generators: Vec<&WitnessGenerator> = self.witness_generators.iter().collect();

        loop {
            let mut made_progress = false;
            pending_generators.retain(|generator| {
                if wire_values.contains_all(&mut generator.inputs()) {
                    generator.generate(wire_values);
                    made_progress = true;
                    false
                } else {
                    true
                }
            });

            if !made_progress {
                break;
            }
        }

        self.constraints.iter().all(|constraint| constraint.evaluate(wire_values))
    }
}

#[cfg(test)]
mod tests {
    use gadget_builder::GadgetBuilder;
    use wire_values::WireValues;

    #[test]
    fn constraint_not_satisfied() {
        let mut builder = GadgetBuilder::new();
        let (x, y) = (builder.wire(), builder.wire());
        builder.assert_equal(x.into(), y.into());
        let gadget = builder.build();

        let mut values = wire_values!(x => 42.into(), y => 43.into());
        let constraints_satisfied = gadget.execute(&mut values);
        assert!(!constraints_satisfied);
    }

    #[test]
    #[should_panic]
    fn missing_generator() {
        let mut builder = GadgetBuilder::new();
        let (x, y, z) = (builder.wire(), builder.wire(), builder.wire());
        builder.assert_product(x.into(), y.into(), z.into());
        let gadget = builder.build();

        let mut values = wire_values!(x => 2.into(), y => 3.into());
        gadget.execute(&mut values);
    }

    #[test]
    #[should_panic]
    fn missing_input() {
        let mut builder = GadgetBuilder::new();
        let x = builder.wire();
        builder.inverse(x.into());
        let gadget = builder.build();

        let mut values = WireValues::new();
        gadget.execute(&mut values);
    }
}
