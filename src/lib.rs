// TODO: Copy some examples etc. here when the API is more stable.

//! This is a rust library for building R1CS gadgets over prime fields, which are useful in SNARKs
//! and other argument systems.
//!
//! See the [readme](https://github.com/mir-protocol/r1cs) for more information and examples.

pub use num;

pub use constraint::*;
pub use curve::*;
pub use davies_meyer::*;
pub use embedded_curve::*;
pub use expression::*;
pub use field::*;
pub use gadget::*;
pub use gadget_builder::*;
pub use gadget_traits::*;
pub use merkle_damgard::*;
pub use merkle_trees::*;
pub use mimc::*;
pub use sponge::*;
pub use wire::*;
pub use wire_values::*;
pub use witness_generator::*;

#[macro_use]
mod wire_values;

mod bimap_util;
mod binary_arithmetic;
mod bitwise_operations;
mod boolean_algebra;
mod comparisons;
mod constraint;
mod curve;
mod davies_meyer;
mod embedded_curve;
mod expression;
mod field;
mod field_arithmetic;
mod gadget;
mod gadget_builder;
mod gadget_traits;
mod merkle_damgard;
mod merkle_trees;
mod mimc;
mod permutations;
mod random_access;
mod sorting;
mod splitting;
mod sponge;
mod wire;
mod witness_generator;

#[cfg(test)]
mod test_util;
