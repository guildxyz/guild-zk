use elliptic_curve::bigint::U256;

const MODULUS: U256 =
    U256::from_be_hex("3fffffffc000000040000000000000002ae382c7957cc4ff9713c3d82bc47d3af");

pub struct FieldElement(U256);
pub struct Scalar(U256);

pub trait Modular {
    const MODULUS: U256;
    const NEG_MODULUS: U256;
    fn add(&self, other: &Self) -> Self;
    fn neg(&self, other: &Self) -> Self;
    fn sub(&self, other: &Self) -> Self;
    fn mul(&self, other: &Self) -> Self;

}

impl Modular for FieldElement {
    fn add(&self, other: &Self) -> Self {
        Self(self.0.add_mod(&other.0, &MODULUS))
    }

    fn neg(&self) -> Self {
        Self(self.0.neg_mod(&MODULUS))
    }

    fn sub(&self, other: &Self) -> Self {
        Self(self.0.sub_mod(&other.0, &MODULUS))
    }

    pub fn mul(&self, other: &Self) -> Self {
        let (lo, hi) = self.mul_wide(other);
        Self::reduce(lo, hi)
        todo!();
    }
}

