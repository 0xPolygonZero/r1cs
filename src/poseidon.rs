#[cfg(feature = "no-std")]
use alloc::vec::Vec;
#[cfg(feature = "no-std")]
use alloc::boxed::Box;

use crate::{Element, Expression, Field, GadgetBuilder, InversePermutation, MdsMatrix, MonomialPermutation, MultiPermutation, Permutation};

const DEFAULT_SECURITY_LEVEL: usize = 128;

/// An S-Box that can be used with Poseidon.
#[derive(Copy, Clone, Debug)]
pub enum PoseidonSbox {
    Exponentiation3,
    Exponentiation5,
    Inverse,
}

/// The Poseidon permutation.
pub struct Poseidon<F: Field> {
    width: usize,
    num_rounds: NumberOfRounds,
    sbox: PoseidonSbox,
    mds_matrix: MdsMatrix<F>,
}

#[derive(Copy, Clone, Debug)]
pub struct NumberOfRounds {
    full: usize,
    partial: usize,
}

impl<F: Field> Poseidon<F> {
    /// TODO: Generate an MDS matrix instead of making the caller supply one.
    pub fn new(width: usize, mds_matrix: MdsMatrix<F>) -> Self {
        // Determine the optimal S-box, based on the guidance in the paper.
        let sbox = match Element::<F>::largest_element() {
            ref x if x.gcd(&3u8.into()).is_one() => PoseidonSbox::Exponentiation3,
            ref x if x.gcd(&5u8.into()).is_one() => PoseidonSbox::Exponentiation5,
            _ => PoseidonSbox::Inverse,
        };

        // Determine the optimal numbers of full and partial rounds.
        let num_rounds = secure_num_rounds_padded::<F>(sbox, width);

        Poseidon {
            width,
            num_rounds,
            sbox,
            mds_matrix,
        }
    }

    fn sbox_permute(&self, builder: &mut GadgetBuilder<F>, x: &Expression<F>) -> Expression<F> {
        self.sbox_to_permutation().permute(builder, x)
    }

    fn sbox_inverse(&self, builder: &mut GadgetBuilder<F>, x: &Expression<F>) -> Expression<F> {
        self.sbox_to_permutation().inverse(builder, x)
    }

    fn sbox_to_permutation(&self) -> Box<dyn Permutation<F>> {
        match &self.sbox {
            PoseidonSbox::Inverse => Box::new(InversePermutation),
            PoseidonSbox::Exponentiation3 => Box::new(MonomialPermutation::new(Element::from(3u8))),
            PoseidonSbox::Exponentiation5 => Box::new(MonomialPermutation::new(Element::from(5u8))),
        }
    }
}

impl<F: Field> MultiPermutation<F> for Poseidon<F> {
    fn width(&self) -> usize {
        self.width
    }

    fn permute(&self, builder: &mut GadgetBuilder<F>, inputs: &[Expression<F>])
               -> Vec<Expression<F>> {
        assert_eq!(inputs.len(), self.width);

        let rounds = self.num_rounds.full + self.num_rounds.partial;
        assert!(self.num_rounds.full % 2 == 0, "asymmetric permutation configuration");
        let full_rounds_per_side = self.num_rounds.full / 2;

        let mut current = inputs.to_vec();
        for round in 0..rounds {
            // Sub words layer.
            let full = round < full_rounds_per_side || round >= rounds - full_rounds_per_side;
            if full {
                current = current.into_iter()
                    .map(|exp| self.sbox_permute(builder, &exp))
                    .collect();
            } else {
                current[0] = self.sbox_permute(builder, &current[0]);
            }

            // Mix layer.
            current = &self.mds_matrix * current.as_slice();
        }

        current
    }

    fn inverse(&self, builder: &mut GadgetBuilder<F>, outputs: &[Expression<F>])
               -> Vec<Expression<F>> {
        assert_eq!(outputs.len(), self.width);

        let rounds = self.num_rounds.full + self.num_rounds.partial;
        assert!(self.num_rounds.full % 2 == 0, "asymmetric permutation configuration");
        let full_rounds_per_side = self.num_rounds.full / 2;

        let mut current = outputs.to_vec();//.to_owned();
        for round in 0..rounds {
            // Mix layer.
            current = &self.mds_matrix * current.as_slice();

            // Sub words layer.
            let full = round < full_rounds_per_side || round >= rounds - full_rounds_per_side;
            if full {
                current = current.into_iter()
                    .map(|exp| self.sbox_inverse(builder, &exp))
                    .collect();
            } else {
                current[0] = self.sbox_inverse(builder, &current[0]);
            }
        }

        current
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::{Expression, GadgetBuilder, MdsMatrix, MultiPermutation, Poseidon};
    use crate::poseidon::NumberOfRounds;
    use crate::PoseidonSbox::Exponentiation3;
    use crate::test_util::F11;

    #[test]
    fn poseidon_x3_f11() {
        let mds_matrix = MdsMatrix::<F11>::new(vec![
            vec![2u8.into(), 3u8.into(), 1u8.into(), 1u8.into()],
            vec![1u8.into(), 2u8.into(), 3u8.into(), 1u8.into()],
            vec![1u8.into(), 1u8.into(), 2u8.into(), 3u8.into()],
            vec![3u8.into(), 1u8.into(), 1u8.into(), 2u8.into()],
        ]);

        let poseidon = Poseidon {
            width: 4,
            num_rounds: NumberOfRounds { full: 4, partial: 6 },
            sbox: Exponentiation3,
            mds_matrix,
        };

        let mut builder = GadgetBuilder::new();
        let input_wires = builder.wires(4);
        let input_exps = input_wires.iter().map(Expression::from).collect_vec();
        let _outputs = poseidon.permute(&mut builder, &input_exps);
        let gadget = builder.build();

        let mut values = values!(
            input_wires[0] => 0u8.into(), input_wires[1] => 1u8.into(),
            input_wires[2] => 2u8.into(), input_wires[3] => 3u8.into());
        assert!(gadget.execute(&mut values));
    }
}

/// Selects a number of full and partial rounds so as to provide plausible security, including a
/// reasonable security margin as suggested by the Poseidon authors.
fn secure_num_rounds_padded<F: Field>(sbox: PoseidonSbox, width: usize) -> NumberOfRounds {
    let unpadded = secure_num_rounds_unpadded::<F>(sbox, width);
    NumberOfRounds {
        full: unpadded.full + 2,
        partial: (unpadded.partial as f64 * 1.075).round() as usize,
    }
}

fn secure_num_rounds_unpadded<F: Field>(sbox: PoseidonSbox, width: usize) -> NumberOfRounds {
    let mut full = 6;
    let mut best_rounds = NumberOfRounds {
        full,
        partial: secure_partial_rounds_unpadded::<F>(sbox, width, full),
    };
    let mut best_sboxes = num_sboxes(width, best_rounds);

    loop {
        // We increment by 2 to maintain symmetry.
        full += 2;

        let rounds = NumberOfRounds {
            full,
            partial: secure_partial_rounds_unpadded::<F>(sbox, width, full),
        };
        let sboxes = num_sboxes(width, rounds);

        if sboxes > best_sboxes {
            // The cost is starting to increase. Terminate with the best configuration we found.
            break best_rounds;
        }

        best_rounds = rounds;
        best_sboxes = sboxes;
    }
}

fn secure_partial_rounds_unpadded<F: Field>(
    sbox: PoseidonSbox, width: usize, full_rounds: usize
) -> usize {
    // We could do an exponential search here, but brute force seems fast enough.
    let mut partial = 0;
    loop {
        if !is_attackable::<F>(sbox, width, NumberOfRounds { full: full_rounds, partial }) {
            break partial
        }
        partial += 1;
    }
}

fn is_attackable<F: Field>(sbox: PoseidonSbox, width: usize, num_rounds: NumberOfRounds) -> bool {
    match sbox {
        PoseidonSbox::Exponentiation3 => is_attackable_exponentiation_3::<F>(width, num_rounds),
        PoseidonSbox::Exponentiation5 => is_attackable_exponentiation_5::<F>(width, num_rounds),
        PoseidonSbox::Inverse => is_attackable_inverse::<F>(width, num_rounds),
    }
}

fn is_attackable_exponentiation_3<F: Field>(width: usize, num_rounds: NumberOfRounds) -> bool {
    let inequality_1 = (num_rounds.full + num_rounds.partial) as f64
        <= 2f64.log(3f64) * min_n_m::<F>() + (width as f64).log2();
    let inequality_2a = (num_rounds.full + num_rounds.partial) as f64
        <= 0.32 * min_n_m::<F>();
    let inequality_2b = ((width - 1) * num_rounds.full + num_rounds.partial) as f64
        <= 0.18 * min_n_m::<F>() - 1.0;
    inequality_1 || inequality_2a || inequality_2b
}

fn is_attackable_exponentiation_5<F: Field>(width: usize, num_rounds: NumberOfRounds) -> bool {
    let inequality_1 = (num_rounds.full + num_rounds.partial) as f64
        <= 2f64.log(5f64) * min_n_m::<F>() + (width as f64).log2();
    let inequality_2a = (num_rounds.full + num_rounds.partial) as f64
        <= 0.21 * min_n_m::<F>();
    let inequality_2b = ((width - 1) * num_rounds.full + num_rounds.partial) as f64
        <= 0.14 * min_n_m::<F>() - 1.0;
    inequality_1 || inequality_2a || inequality_2b
}

fn is_attackable_inverse<F: Field>(width: usize, num_rounds: NumberOfRounds) -> bool {
    let inequality_1 = num_rounds.full as f64 * (width as f64).log2() + num_rounds.partial as f64
        <= (width as f64).log2() + 0.5 + min_n_m::<F>();
    // In the paper, inequality (2a) is identical to (1) for the case of 1/x, so we omit it.
    let inequality_2 = ((width - 1) * num_rounds.full + num_rounds.partial) as f64
        <= 0.25 * min_n_m::<F>() - 1.0;
    inequality_1 || inequality_2
}

/// The minimum of the field size (in bits) and the security level, which the paper calls
/// `min{n, M}`.
fn min_n_m<F: Field>() -> f64 {
    DEFAULT_SECURITY_LEVEL.min(Element::<F>::max_bits()) as f64
}

fn num_sboxes(width: usize, num_rounds: NumberOfRounds) -> usize {
    num_rounds.full * width + num_rounds.partial
}