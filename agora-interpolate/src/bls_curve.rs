use crate::Interpolate;
use bls::Scalar;
use subtle::CtOption;

impl Interpolate for Scalar {
    fn zero() -> Self {
        Self::zero()
    }

    fn one() -> Self {
        Self::one()
    }

    fn from_u64(num: u64) -> Self {
        Self::from(num)
    }

    fn inverse(&self) -> CtOption<Self> {
        self.invert()
    }
}

#[cfg(test)]
impl crate::GroupElement for bls::G2Projective {
    fn generator() -> Self {
        Self::generator()
    }

    fn identity() -> Self {
        Self::identity()
    }
}

#[cfg(test)]
crate::macros::test_polynomial!(bls::Scalar, bls::G2Projective);
