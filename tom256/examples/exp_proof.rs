use tom256::arithmetic::{Point, Scalar};
use tom256::curve::{Secp256k1, Tom256k1};
use tom256::pedersen::PedersenCycle;
use tom256::proofs::{ExpProof, PointExpSecrets};

use rand::rngs::StdRng;
use rand_core::SeedableRng;

fn main() {
    let mut rng = StdRng::from_seed([14; 32]);
    let base_gen = Point::<Secp256k1>::GENERATOR;
    let pedersen_cycle = PedersenCycle::<Secp256k1, Tom256k1>::new(&mut rng);

    let exponent = Scalar::<Secp256k1>::random(&mut rng);
    let result = base_gen.scalar_mul(&exponent);

    let secrets = PointExpSecrets::new(exponent, result);
    let commitments = secrets.commit(&mut rng, &pedersen_cycle, None);

    let security_param = 60;
    let proof = ExpProof::construct(
        &mut rng,
        &base_gen,
        &pedersen_cycle,
        &secrets,
        &commitments,
        security_param,
    )
    .unwrap();

    assert!(proof
        .verify(
            &mut rng,
            &base_gen,
            &pedersen_cycle,
            &commitments,
            security_param,
        )
        .is_ok());
}
