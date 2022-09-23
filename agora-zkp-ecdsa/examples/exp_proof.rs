use agora_zkp_ecdsa::arithmetic::{Point, Scalar};
use agora_zkp_ecdsa::curve::{Secp256k1, Tom256k1};
use agora_zkp_ecdsa::pedersen::PedersenCycle;
use agora_zkp_ecdsa::proofs::{ExpProof, ExpSecrets};

use rand_core::OsRng;

use std::time::Instant;

fn main() {
    let mut rng = OsRng;
    let base_gen = Point::<Secp256k1>::GENERATOR;
    let pedersen_cycle = PedersenCycle::<Secp256k1, Tom256k1>::new(&mut rng);

    let loops = 10;
    let mut total_prove_elapsed = 0u128;
    let mut total_verify_elapsed = 0u128;
    for i in 1..=loops {
        let exponent = Scalar::<Secp256k1>::random(&mut rng);
        let result = base_gen.scalar_mul(&exponent);
        let secrets = ExpSecrets::new(exponent, result.into());
        let commitments = secrets.commit(&mut rng, &pedersen_cycle);
        println!("RUNNING LOOP {}/{}", i, loops);
        let mut start = Instant::now();
        let proof = ExpProof::construct(
            //&mut rng,
            &base_gen,
            &pedersen_cycle,
            &secrets,
            &commitments,
            None,
        )
        .unwrap();
        let prove_elapsed = start.elapsed().as_millis();
        start = Instant::now();
        assert!(proof
            .verify(
                //&mut rng,
                &base_gen,
                &pedersen_cycle,
                &commitments.into_commitments(),
                None,
            )
            .is_ok());
        let verify_elapsed = start.elapsed().as_millis();
        total_prove_elapsed += prove_elapsed;
        total_verify_elapsed += verify_elapsed;
    }
    println!("AVG PROVE  {} [ms]", total_prove_elapsed / loops as u128);
    println!("AVG VERIFY {} [ms]", total_verify_elapsed / loops as u128);
}
