use std::borrow::Borrow;
use std::collections::HashMap;

use num::BigUint;
use num_traits::One;

use crate::expression::BooleanExpression;
use crate::field_element::FieldElement;
use crate::wire::{BinaryWire, BooleanWire, Wire};

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct WireValues {
    values: HashMap<Wire, FieldElement>,
}

impl WireValues {
    pub fn new() -> Self {
        let mut values = HashMap::new();
        values.insert(Wire::ONE, FieldElement::one());
        WireValues { values }
    }

    pub fn get(&self, wire: Wire) -> &FieldElement {
        assert!(self.values.contains_key(&wire), "No value for {}", wire);
        &self.values[&wire]
    }

    pub fn get_boolean(&self, wire: BooleanWire) -> bool {
        BooleanExpression::from(wire).evaluate(self)
    }

    pub fn set(&mut self, wire: Wire, value: FieldElement) {
        let old_value = self.values.insert(wire, value);
        assert!(old_value.is_none());
    }

    pub fn set_boolean(&mut self, wire: BooleanWire, value: bool) {
        self.set(wire.wire(), value.into());
    }

    pub fn set_binary_unsigned<BW, BU>(&mut self, wire: BW, value: BU)
        where BW: Borrow<BinaryWire>, BU: Borrow<BigUint> {
        let wire = wire.borrow();
        let value = value.borrow();

        let l = wire.len();
        assert!(value.bits() <= l, "Value does not fit");

        for i in 0..l {
            let value = ((value >> i) & BigUint::one()).is_one();
            self.set_boolean(wire.bits[i], value);
        }
    }

    pub fn contains(&self, wire: Wire) -> bool {
        self.values.contains_key(&wire)
    }

    pub fn contains_boolean(&self, wire: BooleanWire) -> bool {
        self.contains(wire.wire)
    }

    pub fn contains_all(&self, wires: &[Wire]) -> bool {
        wires.iter().all(|&wire| self.contains(wire))
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