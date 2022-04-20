mod field;
mod int_ops;
mod point;

use elliptic_curve::bigint::U256;
use field::{FieldElement, Modular};


const ORDER: U256 =
    U256::from_be_hex("ffffffff00000001000000000000000000000000ffffffffffffffffffffffff");

/*
const MODULUS: U256 = U256::from_be_hex();
const EQUATION_COEFF_B: U256 = U256::from_be_hex("7");
const GEN_X: U256 = U256::from_be_hex();
const GEN_Y: U256 = U256::from_be_hex();
*/

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sanity_check() {
        let fe1 = FieldElement::new(11);
        let fe2 = FieldElement::new(97);

        println!("fe1: {:?}", fe1);
        println!("fe2: {:?}", fe2);
        println!("");

        println!("!fe1: {:?}", fe1.neg());
        println!("fe1 + fe2: {:?}", fe1.add(&fe2));
        println!("fe1 - fe2: {:?}", fe1.sub(&fe2));
        println!("fe1 * fe2: {:?}", fe1.mul(&fe2));

        

    }
}