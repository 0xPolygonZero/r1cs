// TODO: Copy some examples etc. here when the API is more stable.

//! This is a rust library for building R1CS gadgets over prime fields, which are useful in SNARKs
//! and other argument systems.
//!
//! See the [readme](https://github.com/mir-protocol/r1cs) for more information and examples.

#[macro_use]
pub mod wire_values;

mod binary_arithmetic;
mod bitwise_operations;
mod boolean_algebra;
mod comparisons;
pub mod constraint;
pub mod davies_meyer;
pub mod expression;
pub mod field;
mod field_arithmetic;
pub mod gadget;
pub mod gadget_builder;
pub mod gadget_traits;
pub mod merkle_damgard;
pub mod merkle_trees;
pub mod mimc;
mod permutations;
mod random_access;
mod sorting;
mod splitting;
pub mod wire;
pub mod witness_generator;

mod bimap_util;

#[cfg(test)]
mod test_util;