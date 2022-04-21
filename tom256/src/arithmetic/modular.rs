use bigint::{NonZero, Split, U256, U512};

pub trait Modular: Sized {
    const MODULUS: U256;

    fn new(number: U256) -> Self;

    fn inner(&self) -> &U256;

    fn add(&self, other: &Self) -> Self {
        Self::new(self.inner().add_mod(&other.inner(), &Self::MODULUS))
    }

    fn neg(&self) -> Self {
        Self::new(self.inner().neg_mod(&Self::MODULUS))
    }

    fn sub(&self, other: &Self) -> Self {
        Self::new(self.inner().sub_mod(&other.inner(), &Self::MODULUS))
    }

    fn mul(&self, other: &Self) -> Self {
        Self::new(mul_mod_u256(self.inner(), other.inner(), &Self::MODULUS))
    }
}

pub fn mul_mod_u256(lhs: &U256, rhs: &U256, modulus: &U256) -> U256 {
    // NOTE modulus is never zero, so unwrap is fine here
    let mod512 = NonZero::new(U512::from((*modulus, U256::ZERO))).unwrap();
    // U512::from((lo, hi))
    let product = U512::from(lhs.mul_wide(rhs));
    // split the remainder result of a % b into a (lo, hi) U256 pair
    // 'hi' should always be zero because the modulus is an U256 number
    let (rem, _) = (product % mod512).split();
    rem
}
