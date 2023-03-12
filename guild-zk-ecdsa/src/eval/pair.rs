use ark_ec::models::short_weierstrass::{Affine, SWCurveConfig};

use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};

#[derive(Clone, Copy)]
pub struct Known<C: SWCurveConfig> {
    pub point: Affine<C>,
    pub index: usize,
}

#[derive(Clone, Copy)]
pub struct Pair<C: SWCurveConfig> {
    pub scalar: C::ScalarField,
    pub point: Affine<C>,
}

impl<C: SWCurveConfig> Pair<C> {
    pub fn multiply(&self) -> Affine<C> {
        (self.point * self.scalar).into()
    }
}

impl<C: SWCurveConfig> std::ops::Mul<C::ScalarField> for Pair<C> {
    type Output = Self;
    fn mul(self, rhs: C::ScalarField) -> Self::Output {
        Self {
            scalar: self.scalar * rhs,
            point: self.point,
        }
    }
}

impl<C: SWCurveConfig> PartialEq for Pair<C> {
    fn eq(&self, other: &Self) -> bool {
        self.scalar.eq(&other.scalar)
    }
}

impl<C: SWCurveConfig> Eq for Pair<C> {}

impl<C: SWCurveConfig> PartialOrd for Pair<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.scalar.partial_cmp(&other.scalar)
    }
}

impl<C: SWCurveConfig> Ord for Pair<C> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.scalar.cmp(&other.scalar)
    }
}
