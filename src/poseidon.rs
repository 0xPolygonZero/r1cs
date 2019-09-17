#[cfg(feature = "no-std")]
use alloc::vec::Vec;

use crate::{Expression, Field, GadgetBuilder, MdsMatrix, MultiPermutation, Permutation};

/// The Poseidon permutation.
pub struct Poseidon<F: Field, SB: Permutation<F>> {
    pub width: usize,
    pub full_rounds: usize,
    pub partial_rounds: usize,
    pub sbox: SB,
    pub mds_matrix: MdsMatrix<F>,
}

impl<F: Field, SB: Permutation<F>> MultiPermutation<F> for Poseidon<F, SB> {
    fn width(&self) -> usize {
        self.width
    }

    fn permute(&self, builder: &mut GadgetBuilder<F>, inputs: &[Expression<F>])
               -> Vec<Expression<F>> {
        assert_eq!(inputs.len(), self.width);

        let rounds = self.full_rounds + self.partial_rounds;
        assert!(self.full_rounds % 2 == 0, "asymmetric permutation configuration");
        let full_rounds_per_side = self.full_rounds / 2;

        let mut current = inputs.to_vec();//.to_owned();
        for round in 0..rounds {
            // Sub words layer.
            let full = round < full_rounds_per_side || round >= rounds - full_rounds_per_side;
            if full {
                current = current.into_iter()
                    .map(|exp| self.sbox.permute(builder, &exp))
                    .collect();
            } else {
                current[0] = self.sbox.permute(builder, &current[0]);
            }

            // Mix layer.
            current = &self.mds_matrix * current.as_slice();
        }

        current.to_vec()
    }

    fn inverse(&self, builder: &mut GadgetBuilder<F>, outputs: &[Expression<F>])
               -> Vec<Expression<F>> {
        assert_eq!(outputs.len(), self.width);

        let rounds = self.full_rounds + self.partial_rounds;
        assert!(self.full_rounds % 2 == 0, "asymmetric permutation configuration");
        let full_rounds_per_side = self.full_rounds / 2;

        let mut current = outputs.to_vec();//.to_owned();
        for round in 0..rounds {
            // Mix layer.
            current = &self.mds_matrix * current.as_slice();

            // Sub words layer.
            let full = round < full_rounds_per_side || round >= rounds - full_rounds_per_side;
            if full {
                current = current.into_iter()
                    .map(|exp| self.sbox.inverse(builder, &exp))
                    .collect();
            } else {
                current[0] = self.sbox.inverse(builder, &current[0]);
            }
        }

        current
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::{Element, Expression, GadgetBuilder, MdsMatrix, MonomialPermutation, MultiPermutation, Poseidon};
    use crate::test_util::F11;

    #[test]
    fn poseidon_x3_f11() {
        let sbox = MonomialPermutation::new(Element::from(3u8));

        let mds_matrix = MdsMatrix::<F11>::new(vec![
            vec![2u8.into(), 3u8.into(), 1u8.into(), 1u8.into()],
            vec![1u8.into(), 2u8.into(), 3u8.into(), 1u8.into()],
            vec![1u8.into(), 1u8.into(), 2u8.into(), 3u8.into()],
            vec![3u8.into(), 1u8.into(), 1u8.into(), 2u8.into()],
        ]);

        let poseidon = Poseidon {
            width: 4,
            full_rounds: 4,
            partial_rounds: 6,
            sbox,
            mds_matrix,
        };

        let mut builder = GadgetBuilder::new();
        let input_wires = builder.wires(4);
        let input_exps = input_wires.iter().map(Expression::from).collect_vec();
        let outputs = poseidon.permute(&mut builder, &input_exps);
        let gadget = builder.build();

        let mut values = values!(
            input_wires[0] => 0u8.into(), input_wires[1] => 1u8.into(),
            input_wires[2] => 2u8.into(), input_wires[3] => 3u8.into());
        assert!(gadget.execute(&mut values));
    }
}