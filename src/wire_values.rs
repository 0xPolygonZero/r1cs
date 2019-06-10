use std::collections::HashMap;

use crate::field_element::FieldElement;
use crate::wire::Wire;

pub struct WireValues {
    values: HashMap<Wire, FieldElement>,
}

impl WireValues {
    pub fn new() -> Self {
        let mut values = HashMap::new();
        values.insert(Wire::ONE, FieldElement::one());
        WireValues { values }
    }

    pub fn get(&self, wire: &Wire) -> FieldElement {
        assert!(self.values.contains_key(wire), "No value for {}", wire);
        self.values[wire].clone()
    }

    pub fn get_all<'a, I>(&self, wires: I) -> Vec<FieldElement>
        where I: Iterator<Item=&'a Wire> {
        wires.map(|w| self.get(&w)).collect()
    }

    pub fn set(&mut self, wire: Wire, value: FieldElement) {
        let old_value = self.values.insert(wire, value);
        assert!(old_value.is_none());
    }

    pub fn set_all<W, F>(&mut self, mut wires: W, mut values: F)
        where W: Iterator<Item=Wire>,
              F: Iterator<Item=FieldElement> {
        loop {
            match (wires.next(), values.next()) {
                (Some(wire), Some(value)) => self.set(wire, value),
                (None, None) => break,
                _ => panic!("different numbers of wires and values"),
            }
        }
    }

    pub fn contains(&self, wire: &Wire) -> bool {
        self.values.contains_key(wire)
    }

    pub fn contains_all<'a>(&self, wires: &mut impl Iterator<Item=&'a Wire>) -> bool {
        wires.all(|wire| self.contains(wire))
    }
}

#[macro_export]
macro_rules! values {
    ( $( $wire:expr => $value:expr ),* ) => {
        {
            let mut values = $crate::wire_values::WireValues::new();
            $(
                values.set($wire, $value);
            )*
            values
        }
    }
}