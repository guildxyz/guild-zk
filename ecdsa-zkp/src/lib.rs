use curv::arithmetic::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn hello(bytes: &[u8]) -> String {
    let bigint = BigInt::from_bytes(bytes);
    bigint.to_string()
}
