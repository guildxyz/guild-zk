use tom256::arithmetic::{Point, Scalar};
use tom256::curve::{Secp256k1, Tom256k1};
use tom256::pedersen::PedersenGenerator;
use tom256::proofs::{ExpPedersenParameters, ExpProof, PointExpSecrets};

use rand::rngs::StdRng;
use rand_core::SeedableRng;

fn main() {
    let mut rng = StdRng::from_seed([14; 32]);
    let pedersen_params = ExpPedersenParameters {
        base_g: Point::<Secp256k1>::GENERATOR,
        base: PedersenGenerator::<Secp256k1>::new(&mut rng),
        cycle: PedersenGenerator::<Tom256k1>::new(&mut rng),
    };

    let exponent = Scalar::<Secp256k1>::random(&mut rng);
    let result = Point::<Secp256k1>::GENERATOR.scalar_mul(&exponent);

    let secrets = PointExpSecrets::new(exponent, result);
    let commitments = secrets.commit(&mut rng, &pedersen_params, None);

    let security_param = 80;
    let proof = ExpProof::construct(
        &mut rng,
        &pedersen_params,
        &secrets,
        &commitments,
        security_param,
    )
    .unwrap();

    assert!(proof
        .verify(&mut rng, &pedersen_params, &commitments, security_param,)
        .is_ok());
}
