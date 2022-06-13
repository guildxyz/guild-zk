use tom256::arithmetic::{Point, Scalar};
use tom256::curve::{Secp256k1, Tom256k1};
use tom256::pedersen::PedersenCycle;
use tom256::proofs::{ExpProof, ExpSecrets};

use rand::rngs::OsRng;
use std::time::Instant;

#[tokio::main]
async fn main() {
    let mut rng = OsRng;
    let base_gen = Point::<Secp256k1>::GENERATOR;
    let pedersen_cycle = PedersenCycle::<Secp256k1, Tom256k1>::new(&mut rng);

    let exponent = Scalar::<Secp256k1>::random(&mut rng);

    let security_param = 60;
    for i in 0..10 {
        let result = base_gen.scalar_mul(&exponent);
        let secrets = ExpSecrets::new(exponent, result.into());
        let commitments = secrets.commit(&mut rng, &pedersen_cycle);
        let loop_start = Instant::now();
        println!("RUNNING LOOP {}", i);
        let proof = ExpProof::construct(
            rng,
            &base_gen,
            &pedersen_cycle,
            &secrets,
            &commitments,
            security_param,
            None,
        )
        .await
        .unwrap();

        assert!(proof
            .verify(
                &mut rng,
                &base_gen,
                &pedersen_cycle,
                &commitments.into_commitments(),
                security_param,
                None,
            )
            .is_ok());
        println!("ELAPSED {}", loop_start.elapsed().as_millis());
    }
}
