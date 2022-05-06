use tom256::arithmetic::{Point, Scalar};
use tom256::pedersen::PedersenGenerator;
use tom256::proofs::{PointAddProof, PointAddSecrets};
use tom256::{Secp256k1, Tom256k1};

use rand::rngs::StdRng;
use rand_core::SeedableRng;

fn main() {
    let mut rng = StdRng::from_seed([14; 32]);
    let pedersen_generator = PedersenGenerator::<Tom256k1>::new(&mut rng);

    let p = &Point::<Secp256k1>::GENERATOR * Scalar::<Secp256k1>::random(&mut rng);
    let q = &Point::<Secp256k1>::GENERATOR * Scalar::<Secp256k1>::random(&mut rng);
    let r = &p + &q;
    let secret = PointAddSecrets::new(p, q, r);
    let commitments = secret.commit(&mut rng, &pedersen_generator);

    let proof = PointAddProof::construct(&mut rng, &pedersen_generator, &commitments, &secret);
    assert!(proof.verify(
        &mut rng,
        &pedersen_generator,
        &commitments.into_commitments()
    ));
}
