// TODO: Copy some examples etc. here when the API is more stable.

//! This is a rust library for building R1CS gadgets over prime fields, which are useful in SNARKs
//! and other argument systems.
//!
//! See the [readme](https://github.com/mir-protocol/r1cs) for more information and examples.

#![cfg(feature = "no-std")]
#![no_std]

#[cfg(not(feature = "no-std"))]
#[macro_use]
extern crate std;

#[cfg(feature = "no-std")]
#[macro_use]
extern crate core as std;

#[cfg(feature = "no-std")]
#[macro_use]
extern crate alloc;

pub use num;

pub use constraint::*;
pub use davies_meyer::*;
pub use expression::*;
pub use field::*;
pub use gadget::*;
pub use gadget_builder::*;
pub use gadget_traits::*;
pub use matrices::*;
pub use merkle_damgard::*;
pub use merkle_trees::*;
pub use mimc::*;
pub use permutations::*;
pub use poseidon::*;
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
mod davies_meyer;
mod expression;
mod field;
mod field_arithmetic;
mod gadget;
mod gadget_builder;
mod gadget_traits;
mod matrices;
mod merkle_damgard;
mod merkle_trees;
mod mimc;
mod permutations;
mod poseidon;
mod random_access;
mod sorting;
mod splitting;
mod sponge;
mod verify_permutation;
mod wire;
mod witness_generator;

#[cfg(test)]
mod test_util;

