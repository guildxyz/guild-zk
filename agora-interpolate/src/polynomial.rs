use crate::{Interpolate, InterpolationError};
use std::ops::{AddAssign, Mul, MulAssign, Neg, SubAssign};

#[derive(Clone, Debug, PartialEq)]
pub struct Polynomial<T> {
    coeffs: Vec<T>,
}

impl<T> Polynomial<T> {
    pub fn new(coeffs: Vec<T>) -> Self {
        Self { coeffs }
    }

    pub fn coeffs(&self) -> &[T] {
        &self.coeffs
    }

    pub fn into_coeffs(self) -> Vec<T> {
        self.coeffs
    }
}

impl<T> Polynomial<T>
where
    T: Interpolate + Copy + Mul<Output = T> + Neg<Output = T> + AddAssign + SubAssign + MulAssign,
{
    pub fn interpolate(x: &[T], y: &[T]) -> Result<Self, InterpolationError> {
        if x.len() != y.len() {
            return Err(InterpolationError::InvalidInputLengths(x.len(), y.len()));
        }

        let n = x.len();
        let mut s = vec![T::zero(); n];
        let mut coeffs = vec![T::zero(); n];

        s.push(T::one());
        s[n - 1] = -x[0];

        for (i, &x_elem) in x.iter().enumerate().skip(1) {
            for j in n - 1 - i..n - 1 {
                let aux = x_elem * s[j + 1];
                s[j] -= aux;
            }
            s[n - 1] -= x_elem;
        }

        for i in 0..n {
            let mut phi = T::zero();
            for j in (1..=n).rev() {
                phi *= x[i];
                phi += T::from_u64(j as u64) * s[j];
            }
            let maybe_ff: Option<T> = <T as Interpolate>::inverse(&phi).into();
            let ff = maybe_ff.ok_or(InterpolationError::TriedToInvertZero)?;
            let mut b = T::one();
            for j in (0..n).rev() {
                let aux = b * ff * y[i];
                coeffs[j] += aux;
                b *= x[i];
                b += s[j];
            }
        }

        Ok(Self { coeffs })
    }

    pub fn evaluate(&self, at: T) -> T {
        let mut ret = T::zero();
        for coeff in self.coeffs.iter().rev() {
            ret *= at;
            ret += *coeff;
        }
        ret
    }
}
