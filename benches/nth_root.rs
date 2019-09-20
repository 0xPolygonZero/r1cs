use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;

use r1cs::{Bls12_381, Element, GadgetBuilder, LCG, MonomialPermutation, Permutation, values};

fn criterion_benchmark(c: &mut Criterion) {
    type F = Bls12_381;
    let n = Element::from(5u8);
    let x_n = MonomialPermutation::<F>::new(n.clone());

    let mut builder = GadgetBuilder::<F>::new();
    let residue = builder.wire();
    let root = x_n.inverse(&mut builder, &residue.into());
    let gadget = builder.build();
    let mut lcg = LCG::new();

    c.bench_function("x^{1/5}", move |b| b.iter(|| {
        let residue_value = lcg.next_element();
        let mut values = values!(residue => residue_value.clone());
        gadget.execute(&mut values);

        assert_eq!(root.evaluate(&values).exponentiation(&n), residue_value);
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
