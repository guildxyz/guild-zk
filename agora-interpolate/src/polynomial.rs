use crate::{Interpolate, InterpolationError};
use std::ops::{AddAssign, Mul, MulAssign, Neg, SubAssign};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Polynomial<Y> {
    coeffs: Vec<Y>,
}

impl<Y> Polynomial<Y> {
    pub fn new(coeffs: Vec<Y>) -> Self {
        Self { coeffs }
    }

    pub fn coeffs(&self) -> &[Y] {
        &self.coeffs
    }

    pub fn into_coeffs(self) -> Vec<Y> {
        self.coeffs
    }
}

impl<Y> Polynomial<Y> {
    pub fn interpolate<X>(x: &[X], y: &[Y]) -> Result<Self, InterpolationError>
    where
        Y: Default + Copy + AddAssign + Mul<X, Output = Y>,
        X: Interpolate
            + Copy
            + Mul<Output = X>
            + Neg<Output = X>
            + AddAssign
            + SubAssign
            + MulAssign,
    {
        if x.len() != y.len() {
            return Err(InterpolationError::InvalidInputLengths(x.len(), y.len()));
        }

        let n = x.len();
        let mut s = vec![X::zero(); n];
        let mut coeffs = vec![Y::default(); n];

        s.push(X::one());
        s[n - 1] = -x[0];

        for (i, &x_elem) in x.iter().enumerate().skip(1) {
            for j in n - 1 - i..n - 1 {
                let aux = x_elem * s[j + 1];
                s[j] -= aux;
            }
            s[n - 1] -= x_elem;
        }

        for i in 0..n {
            let mut phi = X::zero();
            for j in (1..=n).rev() {
                phi *= x[i];
                phi += X::from_u64(j as u64) * s[j];
            }
            let maybe_ff: Option<X> = <X as Interpolate>::inverse(&phi).into();
            let ff = maybe_ff.ok_or(InterpolationError::TriedToInvertZero)?;
            let mut b = X::one();
            for j in (0..n).rev() {
                let aux = y[i] * b * ff;
                coeffs[j] += aux;
                b *= x[i];
                b += s[j];
            }
        }

        Ok(Self { coeffs })
    }

    pub fn evaluate<X>(&self, at: X) -> Y
    where
        X: Copy,
        Y: Default + Copy + AddAssign + MulAssign<X>,
    {
        let mut ret = Y::default(); // TODO default is not necessarily always zero?
        for coeff in self.coeffs.iter().rev() {
            ret *= at;
            ret += *coeff;
        }
        ret
    }
}
