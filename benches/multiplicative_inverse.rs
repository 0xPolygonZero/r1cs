use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;

use r1cs::{Bls12_381, Element, LCG};

fn criterion_benchmark(c: &mut Criterion) {
    type F = Bls12_381;

    let mut lcg = LCG::new();

    c.bench_function("1/x", move |b| b.iter(|| {
        let x = lcg.next_element::<F>();
        let x_inv = x.multiplicative_inverse();
        assert_eq!(x * x_inv, Element::one());
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
