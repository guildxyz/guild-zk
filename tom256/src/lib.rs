#![feature(int_log)]
pub mod arithmetic;
pub mod curve;
mod hasher;
mod parse;
pub mod pedersen;
pub mod proofs;

use arithmetic::*;
pub use bigint::U256;
use curve::{Secp256k1, Tom256k1};
use parse::address_to_scalar;
use pedersen::PedersenCycle;
use proofs::MembershipProof;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "generateProof")]
pub fn generate_proof(
    input: JsValue,
    commitment_to_address: JsValue,
    pedersen: JsValue,
) -> Result<JsValue, JsValue> {
    todo!();
}

#[wasm_bindgen(js_name = "verifyProof")]
pub fn verify_proof(proof: JsValue) -> Result<JsValue, JsValue> {
    todo!();
}

#[wasm_bindgen(js_name = "membershipProofTest")]
pub fn membership_proof_test(
    address: String,
    commitment: JsValue,
    pedersen: JsValue,
) -> Result<JsValue, JsValue> {
    let mut rng = rand_core::OsRng;

    let pedersen: PedersenCycle<Secp256k1, Tom256k1> =
        pedersen.into_serde().map_err(|e| e.to_string())?;
    let commitment_to_key = commitment.into_serde().map_err(|e| e.to_string())?;
    let address_scalar = address_to_scalar(&address)?;

    let ring = vec![
        Scalar::<curve::Tom256k1>::new(U256::from_u8(0)),
        Scalar::<curve::Tom256k1>::new(U256::from_u8(1)),
        Scalar::<curve::Tom256k1>::new(U256::from_u8(2)),
        Scalar::<curve::Tom256k1>::new(U256::from_u8(3)),
        Scalar::<curve::Tom256k1>::new(U256::from_u8(4)),
        Scalar::<curve::Tom256k1>::new(U256::from_u8(5)),
        Scalar::<curve::Tom256k1>::new(U256::from_u8(6)),
        address_scalar,
    ];

    let index = 7; // seventh element is our address

    let proof =
        MembershipProof::construct(&mut rng, pedersen.cycle(), &commitment_to_key, index, &ring)
            .map_err(JsValue::from)?;

    proof
        .verify(
            &mut rng,
            pedersen.cycle(),
            commitment_to_key.commitment(),
            &ring,
        )
        .map_err(JsValue::from)?;

    Ok(JsValue::from(true))
}

#[wasm_bindgen(js_name = "generatePedersenParams")]
pub fn generate_pedersen_params() -> Result<JsValue, JsValue> {
    let mut rng = rand_core::OsRng;
    let pedersen = PedersenCycle::<Secp256k1, Tom256k1>::new(&mut rng);
    JsValue::from_serde(&pedersen).map_err(|e| JsValue::from(e.to_string()))
}

#[wasm_bindgen(js_name = "commitAddress")]
pub fn commit_address(address: String, pedersen: JsValue) -> Result<JsValue, JsValue> {
    let mut rng = rand_core::OsRng;

    let pedersen: PedersenCycle<Secp256k1, Tom256k1> =
        pedersen.into_serde().map_err(|e| e.to_string())?;

    let secret = address_to_scalar(&address)?;
    let commitment = pedersen.cycle().commit(&mut rng, secret);
    JsValue::from_serde(&commitment).map_err(|e| JsValue::from(e.to_string()))
}
