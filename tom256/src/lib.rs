#![feature(int_log)]
pub mod arithmetic;
pub mod curve;
mod hasher;
pub mod parse;
pub mod pedersen;
pub mod proofs;

pub use bigint::U256;
/*
use curve::{Secp256k1, Tom256k1};
use parse::{address_to_scalar, ParsedProofInput, ProofInput};
use pedersen::PedersenCycle;
use proofs::ZkAttestProof;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "generateProof")]
pub fn generate_proof(
    input: JsValue,
    commitment_to_address: JsValue,
    pedersen: JsValue,
) -> Result<JsValue, JsValue> {
    let mut rng = rand_core::OsRng;
    let input: ParsedProofInput<Secp256k1, Tom256k1> = input
        .into_serde::<ProofInput>()
        .map_err(|e| e.to_string())?
        .try_into()?;
    let commitment_to_address = commitment_to_address
        .into_serde()
        .map_err(|e| e.to_string())?;
    let pedersen: PedersenCycle<Secp256k1, Tom256k1> =
        pedersen.into_serde().map_err(|e| e.to_string())?;

    let zk_attest_proof =
        ZkAttestProof::construct(&mut rng, pedersen, commitment_to_address, input)?;

    JsValue::from_serde(&zk_attest_proof).map_err(|e| JsValue::from(e.to_string()))
}

#[wasm_bindgen(js_name = "verifyProof")]
pub fn verify_proof(proof: JsValue) -> Result<JsValue, JsValue> {
    let mut rng = rand_core::OsRng;
    let proof: ZkAttestProof<Secp256k1, Tom256k1> =
        proof.into_serde().map_err(|e| e.to_string())?;
    proof.verify(&mut rng)?;
    Ok(JsValue::from(true))
}

#[wasm_bindgen(js_name = "generatePedersenParameters")]
pub fn generate_pedersen_parameters() -> Result<JsValue, JsValue> {
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
*/
