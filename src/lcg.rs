/// This module provides a linear congruential generator for (not cryptographically secure) random
/// data.

use num::BigUint;
use num_traits::One;

use crate::field::{Element, Field};

/// A simple linear congruential generator, with parameters taken from Numerical Recipes.
pub struct LCG {
    state: u32
}

impl LCG {
    pub fn new() -> Self {
        LCG { state: 0 }
    }

    pub fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        self.state
    }

    pub fn next_element<F: Field>(&mut self) -> Element<F> {
        Element::from(self.next_biguint(F::order()))
    }

    pub fn next_biguint(&mut self, limit_exclusive: BigUint) -> BigUint {
        let bits = (&limit_exclusive - BigUint::one()).bits();
        loop {
            let n = self.next_biguint_bits(bits);
            if n < limit_exclusive {
                return n;
            }
        }
    }

    fn next_biguint_bits(&mut self, bits: usize) -> BigUint {
        let full_chunks = bits / 32;
        let remaining_bits = bits % 32;
        let partial_chunk = remaining_bits > 0;

        let mut chunk_data = Vec::new();
        for _i in 0..full_chunks {
            chunk_data.push(self.next_u32());
        }
        if partial_chunk {
            chunk_data.push(self.next_u32() % (1 << remaining_bits))
        }
        BigUint::new(chunk_data)
    }
}

#[cfg(test)]
mod tests {
    use crate::lcg::LCG;

    #[test]
    fn next_u32() {
        let mut lcg = LCG::new();
        assert_eq!(lcg.next_u32(), 1013904223);
        assert_eq!(lcg.next_u32(), 1196435762);
        assert_eq!(lcg.next_u32(), 3519870697);
        assert_eq!(lcg.next_u32(), 2868466484);
    }
}