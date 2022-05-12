use tom256::arithmetic::{Point, Scalar};
use tom256::pedersen::PedersenGenerator;
use tom256::proofs::{ExpProof, PointExpSecrets};
use tom256::{Secp256k1, Tom256k1};

use rand::rngs::StdRng;
use rand_core::SeedableRng;

fn main() {
    let mut rng = StdRng::from_seed([14; 32]);
    let base_pedersen_generator = PedersenGenerator::<Secp256k1>::new(&mut rng);
    let tom_pedersen_generator = PedersenGenerator::<Tom256k1>::new(&mut rng);

    let exponent = Scalar::<Secp256k1>::random(&mut rng);
    let result = Point::<Secp256k1>::GENERATOR.scalar_mul(&exponent);

    let secrets = PointExpSecrets::new(exponent, result);
    let commitments = secrets.commit(
        &mut rng,
        &base_pedersen_generator,
        &tom_pedersen_generator,
        None,
    );

    let security_param = 80;
    let proof = ExpProof::construct(
        &mut rng,
        &base_pedersen_generator,
        &tom_pedersen_generator,
        &secrets,
        &commitments,
        security_param,
    )
    .unwrap();

    assert!(proof
        .verify(
            &mut rng,
            &base_pedersen_generator,
            &tom_pedersen_generator,
            &commitments,
            security_param,
        )
        .is_ok());
}
