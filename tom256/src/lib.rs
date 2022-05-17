#![feature(int_log)]
pub mod arithmetic;
pub mod curve;
pub mod pedersen;
pub mod proofs;
mod utils;

use arithmetic::*;
pub use bigint::U256;
use curve::*;
use pedersen::*;
use proofs::*;
use wasm_bindgen::prelude::*;

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

fn address_to_scalar<C: Curve>(address: &str) -> Result<Scalar<C>, String> {
    let stripped = address.trim_start_matches("0x");
    let mut padded = "000000000000000000000000".to_string(); // 24 zeros to pad 20 bit address to 32 bit scalar
    padded.push_str(stripped);
    if padded.len() != 64 {
        return Err("invalid address".to_string());
    }
    Ok(Scalar::new(U256::from_be_hex(&padded)))
}

#[test]
fn address_conversion() {
    let address = "0x0123456789012345678901234567890123456789";
    let scalar = address_to_scalar::<Tom256k1>(address).unwrap();
    assert_eq!(
        scalar,
        Scalar::new(U256::from_be_hex(
            "0000000000000000000000000123456789012345678901234567890123456789"
        ))
    );

    let address = "0000000000000000000000000000000000000000";
    let scalar = address_to_scalar::<Tom256k1>(address).unwrap();
    assert_eq!(scalar, Scalar::<Tom256k1>::ZERO);

    let address = "0x12345";
    assert!(address_to_scalar::<Tom256k1>(address).is_err());

    let address = "3".repeat(42);
    assert!(address_to_scalar::<Tom256k1>(&address).is_err());
}
