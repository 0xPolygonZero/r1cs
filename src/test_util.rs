//! This module contains test helper functions.

use std::borrow::Borrow;

use crate::expression::BooleanExpression;
use crate::wire_values::WireValues;

pub fn assert_eq_true<T: Borrow<BooleanExpression>>(x: T, values: &WireValues) {
    assert_eq!(true, x.borrow().evaluate(values));
}

pub fn assert_eq_false<T: Borrow<BooleanExpression>>(x: T, values: &WireValues) {
    assert_eq!(false, x.borrow().evaluate(values));
}
