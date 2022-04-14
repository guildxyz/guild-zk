use elliptic_curve::bigint::U256;

pub struct FieldElement(U256);

impl FieldElement {
    const MODULUS: U256 =
        U256::from_be_hex("3fffffffc000000040000000000000002ae382c7957cc4ff9713c3d82bc47d3af");
    //const MODULUS_INV: U256 = U256::from_be_bytes("10")

    pub fn add(&self, other: &Self) -> Self {
        Self(self.0.add_mod(&other.0, &Self::MODULUS))
    }

    pub fn neg(&self) -> Self {
        Self(self.0.neg_mod(&Self::MODULUS))
    }

    pub fn sub(&self, other: &Self) -> Self {
        Self(self.0.sub_mod(&other.0, &Self::MODULUS))
    }

    pub fn mul(&self, other: &Self) -> Self {
        todo!();
        //Self(self.0.mul_mod(&other.0, &Self::MODULUS))
    }
}
