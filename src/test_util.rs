//! This module contains test helper functions.

use std::borrow::Borrow;

use num::BigUint;

use crate::expression::BooleanExpression;
use crate::field::Field;
use crate::wire_values::WireValues;

pub fn assert_eq_true<F, T>(x: T, values: &WireValues<F>)
    where F: Field, T: Borrow<BooleanExpression<F>> {
    assert_eq!(true, x.borrow().evaluate(values));
}

pub fn assert_eq_false<F, T>(x: T, values: &WireValues<F>)
    where F: Field, T: Borrow<BooleanExpression<F>> {
    assert_eq!(false, x.borrow().evaluate(values));
}

#[derive(Debug)]
pub struct F7 {}

impl Field for F7 {
    fn order() -> BigUint {
        BigUint::from(7u8)
    }
}

#[derive(Debug)]
pub struct F11 {}

impl Field for F11 {
    fn order() -> BigUint {
        BigUint::from(11u8)
    }
}

#[derive(Debug)]
pub struct F257 {}

impl Field for F257 {
    fn order() -> BigUint {
        BigUint::from(257u16)
    }
}
