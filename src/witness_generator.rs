use wire::Wire;
use wire_values::WireValues;

pub struct WitnessGenerator {
    pub inputs: Vec<Wire>,
    pub outputs: Vec<Wire>,
    pub generate: Box<Fn(&mut WireValues)>,
}