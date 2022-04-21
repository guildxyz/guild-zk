pub mod arithmetic;
mod point;

use arithmetic::field::FieldElement;
use arithmetic::modular::Modular;

use bigint::U256;

pub trait Curve {
    const PRIME_MODULUS: U256;
    const ORDER: U256;
    const GENERATOR_X: U256;
    const GENERATOR_Y: U256;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TestWasmCurve;

impl Curve for TestWasmCurve {
    const PRIME_MODULUS: U256 = U256::from_u32(1783242237);
    const ORDER: U256 = U256::ONE;
    const GENERATOR_X: U256 = U256::ZERO;
    const GENERATOR_Y: U256 = U256::ZERO;
}

use wasm_bindgen::prelude::*;
#[wasm_bindgen]
pub fn wasm_build_test(bignum: String) -> String {
    let a = FieldElement::<TestWasmCurve>::new(U256::from_be_hex(&bignum));
    let b = FieldElement::<TestWasmCurve>::new(U256::from_u32(134));

    format!("{:?}", a * b)
}
