use constraint::Constraint;
use std::collections::HashMap;
use witness_generator::WitnessGenerator;
use wire_values::WireValues;
use core::borrow::Borrow;

pub struct Gadget {
    constraints: Vec<Constraint>,
    generators: Vec<WitnessGenerator>,
    witness_generators: Vec<WitnessGenerator>,
}

impl Gadget {
    fn execute(&self, mut wire_values: WireValues) -> ExecutionResult {
        let mut pending_generators: Vec<&WitnessGenerator> = self.generators.iter().collect();

        loop {
            let mut made_progress = false;
            pending_generators.retain(|generator| {
                if wire_values.contains_all(&mut generator.inputs()) {
                    generator.generate(&mut wire_values);
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

        let constraints_satisfied = self.constraints.iter().all(
            |constraint| constraint.evaluate(&wire_values));

        ExecutionResult { constraints_satisfied, wire_values }
    }
}

struct ExecutionResult {
    constraints_satisfied: bool,
    wire_values: WireValues,
}