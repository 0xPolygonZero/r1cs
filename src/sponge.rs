//! This module extends GadgetBuilder with an implementation of the Merkle-Damgard construction.

#[cfg(feature = "no-std")]
use alloc::vec::Vec;

use core::iter;
use std::marker::PhantomData;

use itertools::{enumerate, Itertools};

use crate::{GadgetBuilder, MultiPermutation};
use crate::Expression;
use crate::Field;

/// A sponge function.
///
/// In a SNARK setting, efficiency demands that the two sections of sponge state memory (R and C) be
/// stored in separate field elements, so that inputs can be efficiently added to R without
/// affecting C.
pub struct Sponge<F: Field, MP: MultiPermutation<F>> {
    permutation: MP,
    bitrate: usize,
    capacity: usize,
    phantom: PhantomData<*const F>,
}

impl<F: Field, MP: MultiPermutation<F>> Sponge<F, MP> {
    /// Create a new sponge function.
    ///
    /// # Parameters
    /// - `permutation` - the permutation with which to transform state memory
    /// - `bitrate` - the size of the input section, in field elements
    /// - `capacity` - the size of the capacity section, in field elements
    pub fn new(permutation: MP, bitrate: usize, capacity: usize) -> Self {
        assert_eq!(bitrate + capacity, permutation.width(),
                   "Sponge state memory size must match permutation size");
        Sponge { permutation, bitrate, capacity, phantom: PhantomData }
    }

    pub fn evaluate(
        &self, builder: &mut GadgetBuilder<F>, inputs: &[Expression<F>], output_len: usize,
    ) -> Vec<Expression<F>> {
        let mut input_section = iter::repeat(Expression::zero())
            .take(self.bitrate).collect_vec();
        let mut capacity_section = iter::repeat(Expression::zero())
            .take(self.capacity).collect_vec();

        let chunks = inputs.chunks(self.bitrate);
        for chunk in chunks {
            // Add this chunk to the input section.
            for (i, element) in enumerate(chunk) {
                input_section[i] += element;
            }

            // Apply the permutation.
            let old_state = [input_section, capacity_section].concat();
            let new_state = self.permutation.permute(builder, &old_state);
            assert_eq!(old_state.len(), new_state.len());
            let (new_input, new_capacity) = new_state.split_at(self.bitrate);
            input_section = new_input.to_vec();
            capacity_section = new_capacity.to_vec();
        }

        let mut outputs = input_section.clone();
        while outputs.len() < output_len {
            // Apply the permutation.
            let old_state = [input_section, capacity_section].concat();
            let new_state = self.permutation.permute(builder, &old_state);
            assert_eq!(old_state.len(), new_state.len());
            let (new_input, new_capacity) = new_state.split_at(self.bitrate);
            input_section = new_input.to_vec();
            capacity_section = new_capacity.to_vec();

            outputs.extend(input_section.clone())
        }

        // If output_len is not a multiple of the bitrate, then the code above would have added more
        // output elements than we actually want to return.
        outputs.truncate(output_len);

        outputs
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "no-std")]
    use alloc::vec::Vec;
    use crate::{Element, Expression, Field, GadgetBuilder, MultiPermutation, Sponge};
    use crate::test_util::F7;

    #[test]
    fn sponge_1_1_1_f7() {
        // We will use a trivial compression function to keep the test simple.
        // It transforms (x, y) into (2y, 3x).
        struct TestPermutation;

        impl<F: Field> MultiPermutation<F> for TestPermutation {
            fn width(&self) -> usize {
                2
            }

            fn permute(
                &self, _builder: &mut GadgetBuilder<F>, inputs: &[Expression<F>],
            ) -> Vec<Expression<F>> {
                assert_eq!(inputs.len(), 2);
                let x = &inputs[0];
                let y = &inputs[1];
                vec![y * Element::from(2u8), x * Element::from(3u8)]
            }

            fn inverse(
                &self, _builder: &mut GadgetBuilder<F>, outputs: &[Expression<F>],
            ) -> Vec<Expression<F>> {
                assert_eq!(outputs.len(), 2);
                let x = &outputs[0];
                let y = &outputs[1];
                vec![y / Element::from(3u8), x / Element::from(2u8)]
            }
        }

        let mut builder = GadgetBuilder::<F7>::new();
        let x_wire = builder.wire();
        let y_wire = builder.wire();
        let x = Expression::from(x_wire);
        let y = Expression::from(y_wire);
        let blocks = &[x, y];
        let sponge = Sponge::new(TestPermutation, 1, 1);
        let hash = sponge.evaluate(&mut builder, blocks, 1);
        assert_eq!(hash.len(), 1);
        let hash = &hash[0];
        let gadget = builder.build();

        let mut values = values!(x_wire => 3u8.into(), y_wire => 4u8.into());
        assert!(gadget.execute(&mut values));
        // It transforms (x, y) into (2y, 3x).
        // Initial state: (0, 0)
        // After adding 3: (3, 0)
        // After permuting: (0, 2)
        // After adding 4: (4, 2)
        // After permuting: (4, 5)
        // Output: 4
        assert_eq!(Element::from(4u8), hash.evaluate(&values));
    }
}