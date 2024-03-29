use crate::Interpolate;
use k256::Scalar;
use subtle::CtOption;

impl Interpolate for Scalar {
    fn zero() -> Self {
        Self::ZERO
    }

    fn one() -> Self {
        Self::ONE
    }

    fn from_u64(num: u64) -> Self {
        Self::from(num)
    }

    fn inverse(&self) -> CtOption<Self> {
        self.invert()
    }
}

#[cfg(test)]
impl crate::GroupElement for k256::ProjectivePoint {
    fn generator() -> Self {
        Self::GENERATOR
    }

    fn identity() -> Self {
        Self::IDENTITY
    }
}

#[cfg(test)]
crate::macros::test_polynomial!(k256::Scalar, k256::ProjectivePoint);
