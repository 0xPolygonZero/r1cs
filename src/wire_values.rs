use std::collections::HashMap;

use num::BigUint;
use num_traits::One;

use crate::bits::{BinaryWire, BooleanWire, BooleanExpression};
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

    pub fn get_boolean(&self, wire: BooleanWire) -> bool {
        BooleanExpression::from(wire).evaluate(self)
    }

    pub fn get_all<'a, I>(&self, wires: I) -> Vec<FieldElement>
        where I: Iterator<Item=&'a Wire> {
        wires.map(|w| self.get(&w)).collect()
    }

    pub fn set(&mut self, wire: Wire, value: FieldElement) {
        let old_value = self.values.insert(wire, value);
        assert!(old_value.is_none());
    }

    pub fn set_boolean(&mut self, wire: BooleanWire, value: bool) {
        self.set(wire.wire(), value.into());
    }

    pub fn set_binary_unsigned(&mut self, wire: BinaryWire, value: BigUint) {
        let l = wire.len();
        assert!(value.bits() <= l, "Value does not fit");
        for i in 0..l {
            let value = ((value.clone() >> i) & BigUint::one()).is_one();
            self.set_boolean(wire.bits[i].clone(), value);
        }
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

#[macro_export]
macro_rules! boolean_values {
    ( $( $wire:expr => $value:expr ),* ) => {
        {
            let mut values = $crate::wire_values::WireValues::new();
            $(
                values.set_boolean($wire, $value);
            )*
            values
        }
    }
}

#[macro_export]
macro_rules! binary_unsigned_values {
    ( $( $wire:expr => $value:expr ),* ) => {
        {
            let mut values = $crate::wire_values::WireValues::new();
            $(
                values.set_binary_unsigned($wire, $value);
            )*
            values
        }
    }
}