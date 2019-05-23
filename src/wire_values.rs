use std::collections::HashMap;
use wire::Wire;
use field_element::FieldElement;
use std::ops::{Index, IndexMut};

pub struct WireValues {
    values: HashMap<Wire, FieldElement>,
}

impl WireValues {
    // TODO: remove in favor of []?
    pub fn get(&self, wire: &Wire) -> FieldElement {
        self.values[wire].clone()
    }

    pub fn get_all<'a, I>(&self, wires: I) -> Vec<FieldElement>
        where I: Iterator<Item=&'a Wire> {
        wires.map(|w| self.get(&w)).collect()
    }

    pub fn set(&mut self, wire: Wire, value: FieldElement) {
        self.values.insert(wire, value);
    }

    pub fn set_all<'a, W, F>(&mut self, wires: W, values: F)
        where W: Iterator<Item=&'a Wire>,
              F: Iterator<Item=&'a FieldElement> {
        let mut wires_iter = wires;
        let mut values_iter = values;
        loop {
            match (wires_iter.next(), values_iter.next()) {
                (Some(wire), Some(value)) => self.set(*wire, value.clone()),
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