use std::collections::HashMap;
use wire::Wire;
use field_element::FieldElement;

pub struct WireValues {
    values: HashMap<Wire, FieldElement>,
}

impl WireValues {
    pub fn get(&self, wire: &Wire) -> FieldElement {
        return self.values[wire].clone();
    }

    pub fn contains(&self, wire: &Wire) -> bool {
        self.values.contains_key(wire)
    }

    pub fn contains_all<'a>(&self, wires: &mut impl Iterator<Item=&'a Wire>) -> bool {
        wires.all(|wire| self.contains(wire))
    }
}