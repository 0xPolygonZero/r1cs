//! This module contains wrappers for `Wire`s, `Expression`s which designate them as boolean, i.e.,
//! equal to 0 or 1. Similarly, it contains wrappers for `Wire` arrays and `Expression` arrays which
//! designate them as binary, i.e., with each bit equal to 0 or 1.
//!
//! The intention here is to provide a degree of type safety. If you write a function which takes a
//! `BooleanExpression` input, the user could not accidentally pass in an unbound wire; they would
//! need to go through a method like `assert_binary` which would constrain the input to equal 0 or
//! 1.

use std::collections::HashSet;

use num::BigUint;
use num_traits::{One, Zero};

use crate::expression::Expression;
use crate::field_element::FieldElement;
use crate::wire::Wire;
use crate::wire_values::WireValues;

/// A `Wire` whose value is constrained to be binary.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct BooleanWire {
    wire: Wire,
}

impl BooleanWire {
    /// Construct a BooleanWire from an arbitrary wire. This is only safe if you separately
    /// constrain the wire to equal 0 or 1.
    ///
    /// Users should not normally call this method directly; use a method like
    /// `GadgetBuilder::boolean_wire()` instead.
    pub fn new_unsafe(wire: Wire) -> Self {
        BooleanWire { wire }
    }

    pub fn wire(&self) -> Wire {
        self.wire.clone()
    }
}

/// An `Expression` whose value is known to be binary.
#[derive(Clone, Debug)]
pub struct BooleanExpression {
    expression: Expression,
}

impl BooleanExpression {
    /// Create a new bit. This is unsafe in the sense that a "equals 0 or 1" assertion will not be
    /// added automatically. You can use this method if you already know that a quantity will equal
    /// 0 or 1, for example if you computed 1 - b where b is a Bit.
    pub fn new_unsafe(expression: Expression) -> Self {
        BooleanExpression { expression }
    }

    pub fn _false() -> Self {
        BooleanExpression::new_unsafe(Expression::zero())
    }

    pub fn _true() -> Self {
        BooleanExpression::new_unsafe(Expression::one())
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn dependencies(&self) -> Vec<Wire> {
        self.expression.dependencies()
    }

    pub fn evaluate(&self, values: &WireValues) -> bool {
        match self.expression.evaluate(values) {
            ref x if x.is_zero() => false,
            ref x if x.is_one() => true,
            _ => panic!("Boolean expression did not evaluate to [0, 1]")
        }
    }
}

impl From<&BooleanWire> for BooleanExpression {
    fn from(wire: &BooleanWire) -> Self {
        BooleanExpression::new_unsafe(wire.wire.into())
    }
}

impl From<BooleanWire> for BooleanExpression {
    fn from(wire: BooleanWire) -> Self {
        BooleanExpression::from(&wire)
    }
}

/// A "binary wire" which is comprised of several bits, each one being a boolean wire.
#[derive(Clone, Debug)]
pub struct BinaryWire {
    /// The list of bits, ordered from least significant to most significant.
    pub bits: Vec<BooleanWire>,
}

impl BinaryWire {
    /// The number of bits.
    pub fn len(&self) -> usize {
        self.bits.len()
    }
}

/// A "binary expression" which is comprised of several bits, each one being a boolean expression.
#[derive(Clone, Debug)]
pub struct BinaryExpression {
    /// The list of bits, ordered from least significant to most significant.
    pub bits: Vec<BooleanExpression>,
}

impl BinaryExpression {
    /// The number of bits.
    pub fn len(&self) -> usize {
        self.bits.len()
    }

    /// Truncate this bit vector, discarding the more significant bits while keeping the less
    /// significant bits.
    // TODO: Convert to mutable.
    pub fn truncated(&self, l: usize) -> Self {
        assert!(l <= self.len());
        let mut truncated_bits = self.bits.clone();
        truncated_bits.truncate(l);
        BinaryExpression { bits: truncated_bits }
    }

    /// Pad this bit vector, adding 0 bits on the more significant side.
    // TODO: Convert to mutable.
    pub fn padded(&self, l: usize) -> Self {
        assert!(l >= self.len());
        let mut padded_bits = self.bits.clone();
        while padded_bits.len() < l {
            padded_bits.push(BooleanExpression::_false());
        }
        BinaryExpression { bits: padded_bits }
    }

    pub fn add_most_significant(&mut self, bit: BooleanExpression) {
        self.bits.push(bit);
    }

    pub fn chunks(&self, chunk_bits: usize) -> Vec<BinaryExpression> {
        self.bits.chunks(chunk_bits).map(|chunk| BinaryExpression { bits: chunk.to_vec() }).collect()
    }

    /// Join these bits into the field element they encode.
    pub fn join(&self) -> Expression {
        assert!(self.len() < FieldElement::max_bits(), "Cannot fit in a single field element");
        let mut sum = Expression::zero();
        for (i, bit) in self.bits.iter().enumerate() {
            let weight = FieldElement::one() << i;
            sum += &bit.expression * weight;
        }
        sum
    }

    pub fn dependencies(&self) -> Vec<Wire> {
        let mut all = HashSet::new();
        for bool_expression in self.bits.iter() {
            all.extend(bool_expression.dependencies());
        }
        all.into_iter().collect()
    }

    pub fn evaluate(&self, values: &WireValues) -> BigUint {
        let mut sum = BigUint::zero();
        for (i, bit) in self.bits.iter().enumerate() {
            if bit.evaluate(values) {
                sum += BigUint::one() << i;
            }
        }
        sum
    }
}

impl From<&BinaryWire> for BinaryExpression {
    fn from(wire: &BinaryWire) -> Self {
        BinaryExpression {
            bits: wire.bits.iter()
                .map(|bool_wire| BooleanExpression::from(bool_wire))
                .collect()
        }
    }
}

impl From<BinaryWire> for BinaryExpression {
    fn from(wire: BinaryWire) -> Self {
        BinaryExpression::from(&wire)
    }
}