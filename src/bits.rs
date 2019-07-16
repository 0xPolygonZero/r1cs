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
