use crate::field::Field;
use crate::wire::Wire;
use crate::wire_values::WireValues;

pub struct WitnessGenerator<F: Field> {
    inputs: Vec<Wire>,
    generator: Box<dyn Fn(&mut WireValues<F>)>,
}

impl<F: Field> WitnessGenerator<F> {
    pub fn new<T>(inputs: Vec<Wire>, generate: T) -> Self
        where T: Fn(&mut WireValues<F>) + 'static {
        WitnessGenerator {
            inputs,
            generator: Box::new(generate),
        }
    }

    pub fn inputs(&self) -> &[Wire] {
        &self.inputs
    }

    pub fn generate(&self, values: &mut WireValues<F>) {
        (*self.generator)(values)
    }
}