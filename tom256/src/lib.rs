#![feature(int_log)]
pub mod arithmetic;
pub mod curve;
mod hasher;
pub mod parse;
pub mod pedersen;
pub mod proofs;

pub use bigint::U256;
use curve::{Secp256k1, Tom256k1};
use parse::{ParsedProofInput, ProofInput};
use pedersen::PedersenCycle;
use proofs::ZkAttestProof;
use wasm_bindgen::prelude::*;

//#[wasm_bindgen(js_name = "generateProof")]
//pub fn generate_proof(input: JsValue) -> Result<JsValue, JsValue> {
//    let mut rng = rand_core::OsRng;
//    let pedersen = PedersenCycle::<Secp256k1, Tom256k1>::new(&mut rng);
//
//    let input: ParsedProofInput<Secp256k1, Tom256k1> = input
//        .into_serde::<ProofInput>()
//        .map_err(|e| e.to_string())?
//        .try_into()?;
//
//    let zk_attest_proof = ZkAttestProof::construct(&mut rng, pedersen, input)?;
//
//    JsValue::from_serde(&zk_attest_proof).map_err(|e| JsValue::from(e.to_string()))
//}
//
//// This function is only for wasm test purposes as the
//// verification is done on the backend in pure rust.
//// TODO: put this behind a wasm-test feature flag?
//#[wasm_bindgen(js_name = "verifyProof")]
//pub fn verify_proof(proof: JsValue, input: JsValue) -> Result<JsValue, JsValue> {
//    let mut rng = rand_core::OsRng;
//
//    let input: ParsedProofInput<Secp256k1, Tom256k1> = input
//        .into_serde::<ProofInput>()
//        .map_err(|e| e.to_string())?
//        .try_into()?;
//
//    let proof: ZkAttestProof<Secp256k1, Tom256k1> =
//        proof.into_serde().map_err(|e| e.to_string())?;
//
//    proof.verify(&mut rng, &input.ring)?;
//    Ok(JsValue::from(true))
//}
