#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::string::String;

use core::borrow::Borrow;

/// Like SliceConcatExt::concat, but works in stable with no_std.
/// See https://github.com/rust-lang/rust/issues/27747
pub fn concat<T: Clone, V: Borrow<[T]>>(vecs: &[V]) -> Vec<T> {
    let size = vecs.iter().map(|slice| slice.borrow().len()).sum();
    let mut result = Vec::with_capacity(size);
    for v in vecs {
        result.extend_from_slice(v.borrow())
    }
    result
}

/// Like SliceConcatExt::join for strings, but works in stable with no_std.
/// See https://github.com/rust-lang/rust/issues/27747
pub fn join<S: Borrow<str>>(sep: &str, strings: &[S]) -> String {
    let mut builder = String::new();
    for s in strings {
        if !builder.is_empty() {
            builder += sep;
        }
        builder += s.borrow();
    }
    builder
}