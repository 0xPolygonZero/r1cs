use crate::wire::Wire;
use crate::wire_values::WireValues;

pub struct WitnessGenerator {
    inputs: Vec<Wire>,
    generator: Box<Fn(&mut WireValues)>,
}

impl WitnessGenerator {
    pub fn new<T>(inputs: Vec<Wire>, generate: T) -> Self
        where T: Fn(&mut WireValues) + 'static {
        WitnessGenerator {
            inputs,
            generator: Box::new(generate),
        }
    }

    pub fn inputs(&self) -> &[Wire] {
        &self.inputs
    }

    pub fn generate(&self, values: &mut WireValues) {
        (*self.generator)(values)
    }
}