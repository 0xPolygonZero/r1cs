use std::collections::HashMap;

use num::BigUint;
use num_traits::One;

use crate::expression::BooleanExpression;
use crate::field::{Element, Field};
use crate::wire::{BinaryWire, BooleanWire, Wire};

/// An assignment of wire values, where each value is an element of the field `F`.
#[derive(Default, Debug)]
pub struct WireValues<F: Field> {
    values: HashMap<Wire, Element<F>>,
}

impl<F: Field> WireValues<F> {
    pub fn new() -> Self {
        let mut values = HashMap::new();
        values.insert(Wire::ONE, Element::one());
        WireValues { values }
    }

    pub fn get(&self, wire: Wire) -> &Element<F> {
        assert!(self.values.contains_key(&wire), "No value for {}", wire);
        &self.values[&wire]
    }

    pub fn get_boolean(&self, wire: BooleanWire) -> bool {
        BooleanExpression::from(wire).evaluate(self)
    }

    pub fn set(&mut self, wire: Wire, value: Element<F>) {
        let old_value = self.values.insert(wire, value);
        assert!(old_value.is_none());
    }

    pub fn set_boolean(&mut self, wire: BooleanWire, value: bool) {
        self.set(wire.wire(), Element::from(value));
    }

    pub fn set_binary_unsigned(&mut self, wire: &BinaryWire, value: &BigUint) {
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

impl<F: Field> Clone for WireValues<F> {
    fn clone(&self) -> Self {
        WireValues { values: self.values.clone() }
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