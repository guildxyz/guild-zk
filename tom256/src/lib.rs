mod field;
mod point;

use elliptic_curve::bigint::U256;

const ORDER: U256 =
    U256::from_be_hex("ffffffff00000001000000000000000000000000ffffffffffffffffffffffff");

const MODULUS: U256 = U256::from_be_hex();
const EQUATION_COEFF_B: U256 = U256::from_be_hex("7");
const GEN_X: U256 = U256::from_be_hex();
const GEN_Y: U256 = U256::from_be_hex();
