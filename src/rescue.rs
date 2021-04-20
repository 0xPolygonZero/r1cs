#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::{Element, Expression, Field, GadgetBuilder, MdsMatrix, MonomialPermutation, MultiPermutation, Permutation};

const DEFAULT_SECURITY_BITS: usize = 128;
const SECURITY_MARGIN: usize = 2;
const MINIMUM_ROUNDS: usize = 10;

/// The Rescue permutation.
pub struct Rescue<F: Field> {
    /// The size of the permutation, in field elements.
    width: usize,
    /// The degree of the permutation monomial.
    alpha: Element<F>,
    /// The number of rounds to use.
    num_rounds: usize,
    /// The MDS matrix to apply after each permutation layer.
    mds_matrix: MdsMatrix<F>,
}

impl<F: Field> Rescue<F> {
    fn pi_1(&self, builder: &mut GadgetBuilder<F>, x: &Expression<F>) -> Expression<F> {
        MonomialPermutation::new(self.alpha.clone()).permute(builder, x)
    }

    fn pi_2(&self, builder: &mut GadgetBuilder<F>, x: &Expression<F>) -> Expression<F> {
        MonomialPermutation::new(self.alpha.clone()).inverse(builder, x)
    }
}

impl<F: Field> MultiPermutation<F> for Rescue<F> {
    fn width(&self) -> usize {
        self.width
    }

    fn permute(&self, builder: &mut GadgetBuilder<F>, inputs: &[Expression<F>])
               -> Vec<Expression<F>> {
        let mut current = inputs.to_vec();
        for _round in 0..self.num_rounds {
            current = current.iter().map(|exp| self.pi_1(builder, exp)).collect();
            current = &self.mds_matrix * current.as_slice();
            current = current.iter().map(|exp| self.pi_2(builder, exp)).collect();
            current = &self.mds_matrix * current.as_slice();
        }
        current
    }

    fn inverse(&self, _builder: &mut GadgetBuilder<F>, _outputs: &[Expression<F>])
               -> Vec<Expression<F>> {
        unimplemented!("TODO: implement inverse Rescue")
    }
}

/// Builds a `Rescue` instance.
pub struct RescueBuilder<F: Field> {
    /// The size of the permutation, in field elements.
    width: usize,
    /// The degree of the permutation monomial.
    alpha: Option<Element<F>>,
    /// The number of rounds to use.
    num_rounds: Option<usize>,
    /// The desired (classical) security level, in bits.
    security_bits: Option<usize>,
    /// The MDS matrix to apply after each permutation layer.
    mds_matrix: Option<MdsMatrix<F>>,
}

impl<F: Field> RescueBuilder<F> {
    pub fn new(width: usize) -> Self {
        assert!(width > 0, "Permutation width must be non-zero");
        RescueBuilder {
            width,
            alpha: None,
            num_rounds: None,
            security_bits: None,
            mds_matrix: None,
        }
    }

    pub fn alpha(&mut self, alpha: Element<F>) -> &mut Self {
        self.alpha = Some(alpha);
        self
    }

    pub fn num_rounds(&mut self, num_rounds: usize) -> &mut Self {
        self.num_rounds = Some(num_rounds);
        self
    }

    pub fn security_bits(&mut self, security_bits: usize) -> &mut Self {
        self.security_bits = Some(security_bits);
        self
    }

    pub fn mds_matrix(&mut self, mds_matrix: MdsMatrix<F>) -> &mut Self {
        self.mds_matrix = Some(mds_matrix);
        self
    }

    pub fn build(&self) -> Rescue<F> {
        let width = self.width;
        let alpha = self.alpha.clone().unwrap_or_else(Self::smallest_alpha);

        // TODO: Generate a default MDS matrix instead of making the caller supply one.
        let mds_matrix = self.mds_matrix.clone().expect("MDS matrix required for now");

        if self.num_rounds.is_some() && self.security_bits.is_some() {
            panic!("Cannot specify both the number of rounds and the desired security level");
        }
        let num_rounds = self.num_rounds.unwrap_or_else(
            || Self::secure_num_rounds(
                self.security_bits.unwrap_or(DEFAULT_SECURITY_BITS),
                width));

        Rescue { width, alpha, num_rounds, mds_matrix }
    }

    /// Find the smallest prime `a` such that `x^a` is a permutation in `F`, or equivalently,
    /// `gcd(|F| - 1, a) = 1`.
    fn smallest_alpha() -> Element<F> {
        let largest_element = Element::<F>::largest_element();
        let mut alpha = Element::<F>::from(3u8);
        while !largest_element.gcd(&alpha).is_one() {
            // Incremenet alpha to the next prime.
            alpha += Element::one();
            while !alpha.is_prime() {
                alpha += Element::one();
            }
        }
        alpha
    }

    fn secure_num_rounds(security_bits: usize, width: usize) -> usize {
        // As per the paper, a Gröbner basis attack is lower bounded by 2^{4 * width * rounds}.
        // Thus, attackable_rounds = security_bits / (4 * width)
        let attackable_rounds = integer_division_ceil(security_bits, 4 * width);
        (attackable_rounds * SECURITY_MARGIN).max(MINIMUM_ROUNDS)
    }
}

fn integer_division_ceil(n: usize, m: usize) -> usize {
    (n + m - 1) / m
}

#[cfg(test)]
mod tests {
    use crate::MdsMatrix;
    use crate::rescue::RescueBuilder;
    use crate::test_util::F11;

    #[test]
    fn rescue_permutation_f11() {
        let mds_matrix = MdsMatrix::<F11>::new(vec![
            vec![2u8.into(), 3u8.into(), 1u8.into(), 1u8.into()],
            vec![1u8.into(), 2u8.into(), 3u8.into(), 1u8.into()],
            vec![1u8.into(), 1u8.into(), 2u8.into(), 3u8.into()],
            vec![3u8.into(), 1u8.into(), 1u8.into(), 2u8.into()],
        ]);

        let _rescue = RescueBuilder::new(2).security_bits(128).mds_matrix(mds_matrix).build();

        // TODO: Verify execution.
    }
}