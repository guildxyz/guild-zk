#![feature(int_log)]
pub mod arithmetic;
pub mod curve;
pub mod pedersen;
pub mod proofs;
mod utils;

pub use bigint::U256;

use wasm_bindgen::prelude::*;
#[wasm_bindgen]
pub fn membership_proof_test(index: u32) -> Result<JsValue, JsValue> {
    use arithmetic::{Modular, Scalar};
    let mut rng = rand_core::OsRng;
    let pedersen_generator = pedersen::PedersenGenerator::<curve::Tom256k1>::new(&mut rng);
    let ring = vec![
        Scalar::<curve::Tom256k1>::new(U256::from_u8(0)),
        Scalar::<curve::Tom256k1>::new(U256::from_u8(1)),
        Scalar::<curve::Tom256k1>::new(U256::from_u8(2)),
        Scalar::<curve::Tom256k1>::new(U256::from_u8(3)),
        Scalar::<curve::Tom256k1>::new(U256::from_u8(4)),
        Scalar::<curve::Tom256k1>::new(U256::from_u8(5)),
        Scalar::<curve::Tom256k1>::new(U256::from_u8(6)),
        Scalar::<curve::Tom256k1>::new(U256::from_u8(7)),
    ];

    let index = index as usize;
    let commitment_to_key = pedersen_generator.commit(&mut rng, ring[index]);

    let proof = proofs::MembershipProof::construct(
        &mut rng,
        &pedersen_generator,
        &commitment_to_key,
        index,
        &ring,
    )
    .map_err(JsValue::from)?;

    proof
        .verify(
            &mut rng,
            &pedersen_generator,
            commitment_to_key.commitment(),
            &ring,
        )
        .map_err(JsValue::from)?;

    Ok(JsValue::from(true))
}
