#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use std::cmp::Ordering;
use std::fmt;
use std::fmt::Formatter;

/// A wire represents a witness element.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Wire {
    pub index: u32,
}

impl Wire {
    /// A special wire whose value is always set to 1. This is used to create `Expression`s with
    /// constant terms.
    pub const ONE: Wire = Wire { index: 0 };
}

impl Ord for Wire {
    fn cmp(&self, other: &Self) -> Ordering {
        // For presentation, we want the 1 wire to be last. Otherwise use ascending index order.
        if *self == Wire::ONE && *other == Wire::ONE {
            Ordering::Equal
        } else if *self == Wire::ONE {
            Ordering::Greater
        } else if *other == Wire::ONE {
            Ordering::Less
        } else {
            self.index.cmp(&other.index)
        }
    }
}

impl PartialOrd for Wire {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Wire {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.index == 0 {
            write!(f, "1")
        } else {
            write!(f, "wire_{}", self.index)
        }
    }
}

/// A `Wire` whose value is constrained to be binary.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct BooleanWire {
    pub wire: Wire,
}

/// A `Wire` which has been constrained in such a way that it can only take on a value of 0 or 1.
impl BooleanWire {
    /// Construct a BooleanWire from an arbitrary wire. This is only safe if you separately
    /// constrain the wire to equal 0 or 1.
    ///
    /// Users should not normally call this method directly; use a method like
    /// `GadgetBuilder::boolean_wire()` instead.
    pub fn new_unsafe(wire: Wire) -> Self {
        BooleanWire { wire }
    }

    pub fn wire(self) -> Wire {
        self.wire
    }
}

/// A "binary wire" which is comprised of several bits, each one being a boolean wire.
#[derive(Clone, Debug)]
pub struct BinaryWire {
    /// The list of bits, ordered from least significant to most significant.
    pub bits: Vec<BooleanWire>,
}

#[allow(clippy::len_without_is_empty)]
impl BinaryWire {
    /// The number of bits.
    pub fn len(&self) -> usize {
        self.bits.len()
    }
}
