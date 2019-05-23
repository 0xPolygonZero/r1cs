use constraint::Constraint;
use witness_generator::WitnessGenerator;
use wire_values::WireValues;

pub struct Gadget {
    pub constraints: Vec<Constraint>,
    pub witness_generators: Vec<WitnessGenerator>,
}

impl Gadget {
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
