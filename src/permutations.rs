use num::{BigUint, Integer};
use num_traits::One;

use crate::{Element, Expression, Field, GadgetBuilder, Permutation, WireValues};

/// The permutation `1 / x`, with zero being mapped to itself.
pub struct InversePermutation {}

impl<F: Field> Permutation<F> for InversePermutation {
    fn permute(&self, builder: &mut GadgetBuilder<F>, x: &Expression<F>) -> Expression<F> {
        builder.inverse_or_zero(x)
    }

    fn inverse(&self, builder: &mut GadgetBuilder<F>, x: &Expression<F>) -> Expression<F> {
        builder.inverse_or_zero(x)
    }
}

pub struct MonomialPermutation<F: Field> {
    exponent: Element<F>,
}

impl<F: Field> MonomialPermutation<F> {
    pub fn new(exponent: Element<F>) -> Self {
        // It is well-known that x^n is a permutation of F_q iff gcd(n, q - 1) = 1. See, for
        // example, Theorem 1.14 in "Permutation Polynomials of Finite Fields" [Shallue 12].
        assert!(Element::largest_element().gcd(&exponent).is_one(),
                "x^{} is not a permutation of F", exponent);
        MonomialPermutation { exponent }
    }
}

impl<F: Field> Permutation<F> for MonomialPermutation<F> {
    fn permute(&self, builder: &mut GadgetBuilder<F>, x: &Expression<F>) -> Expression<F> {
        builder.exp(x, &self.exponent)
    }

    fn inverse(&self, builder: &mut GadgetBuilder<F>, x: &Expression<F>) -> Expression<F> {
        let inv_wire = builder.wire();
        let inv = Expression::from(inv_wire);
        let exponentiation = builder.exp(&inv, &self.exponent);
        builder.assert_equal(&exponentiation, x);

        // By Fermat's little theorem, x^p = x, so if n divides e, then x^(p / n)^n = x.
        // Further, since x^(p - 1) = 1, x^((p + (p - 1)*k) / n)^n = x for any positive k,
        // provided that n divides p + (p - 1)*k. Thus we start with p, and repeatedly add
        // p - 1 until we find an exponent divisible by n.
        let mut exponent_times_n = F::order();
        let exponent = loop {
            exponent_times_n += F::order() - BigUint::one();
            if exponent_times_n.is_multiple_of(self.exponent.to_biguint()) {
                break Element::from(exponent_times_n / self.exponent.to_biguint());
            }
        };

        let x = x.clone();
        builder.generator(
            x.dependencies(),
            move |values: &mut WireValues<F>| {
                let inv_value = x.evaluate(values).exp(&exponent);
                values.set(inv_wire, inv_value);
            });

        inv
    }
}

#[cfg(tests)]
mod tests {
    use crate::{Element, MonomialPermutation, GadgetBuilder, Permutation};
    use crate::test_util::{F7, F11};

    #[test]
    fn cube_and_cube_root() {
        let mut builder = GadgetBuilder::<F11>::new();
        let permutation = MonomialPermutation::new(3);
        let x_wire = builder.wire();
        let x = Expression::from(x_wire);
        let x_cubed = permutation.permute(&mut builder, &x);
        let cube_root = permutation.inverse(&mut builder, &x_cubed);
        let gadget = builder.build();

        for i in 0u8..11 {
            let mut values = values!(x_wire => i.into());
            assert!(gadget.execute(&mut values));
            assert_eq!(Element::from(i), cube_root.evaluate(&values));
        }
    }

    #[test]
    #[should_panic]
    fn not_a_permutation() {
        // x^3 is not a permutation in F_7, since gcd(3, 7-1) = 3 != 1.
        MonomialPermutation::<F7>::new(Element::from(3u8));
    }
}