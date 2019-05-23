use wire::Wire;
use wire_values::WireValues;

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

    pub fn inputs(&self) -> impl Iterator<Item = &Wire> {
        self.inputs.iter()
    }

    pub fn generate(&self, values: &mut WireValues) {
        (*self.generator)(values)
    }
}