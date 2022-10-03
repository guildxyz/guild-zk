use agora_interpolate::Polynomial;
use bls::{G2Affine, G2Projective, Scalar};
use criterion::{criterion_group, criterion_main, Criterion};
use ff::Field;
use rand_core::OsRng;

fn bench_interpolate(c: &mut Criterion) {
    let mut group = c.benchmark_group("interpolate");

    let n = 20_usize;
    let mut rng = OsRng;
    let random_scalars_x = (0..n)
        .map(|_| Scalar::random(&mut rng))
        .collect::<Vec<Scalar>>();
    let random_scalars_y = (0..n)
        .map(|_| Scalar::random(&mut rng))
        .collect::<Vec<Scalar>>();
    let random_points_y = random_scalars_y
        .iter()
        .map(|y| G2Affine::generator() * y)
        .collect::<Vec<G2Projective>>();

    group.bench_function("scalar-scalar", |b| {
    	b.iter(|| Polynomial::interpolate(&random_scalars_x, &random_scalars_y).unwrap())
    });

    group.bench_function("scalar-point", |b| {
    	b.iter(|| Polynomial::interpolate(&random_scalars_x, &random_points_y).unwrap())
    });
}

criterion_group!(benches, bench_interpolate);
criterion_main!(benches);
