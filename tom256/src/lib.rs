mod arithmetic;
mod point;

use bigint::U256;

pub trait Curve {
    const PRIME_MODULUS: U256;
    const ORDER: U256;
    const GENERATOR_X: U256;
    const GENERATOR_Y: U256;
}

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
