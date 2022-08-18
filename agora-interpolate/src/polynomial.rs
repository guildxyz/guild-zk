use crate::{Interpolate, InterpolationError};
use std::ops::{AddAssign, Mul, MulAssign, Neg, SubAssign};

/// An `N`th order polynomial with `N + 1` coefficients.
#[derive(Clone, Debug, PartialEq)]
pub struct Polynomial<const N: usize, T>
where
    [(); N + 1]:,
{
    coeffs: [T; N + 1],
}

impl<const N: usize, T> Polynomial<N, T>
where
    [(); N + 1]:,
{
    pub fn new(coeffs: [T; N + 1]) -> Self {
        Self { coeffs }
    }

    pub fn coeffs(&self) -> &[T; N + 1] {
        &self.coeffs
    }
}

impl<const N: usize, T> Polynomial<N, T>
where
    T: Interpolate + Copy + Mul<Output = T> + Neg<Output = T> + AddAssign + SubAssign + MulAssign,
    [(); N + 1]:,
{
    pub fn interpolate(x: &[T], y: &[T]) -> Result<Self, InterpolationError> {
        if x.len() != y.len() {
            return Err(InterpolationError::InvalidInputLengths(x.len(), y.len()));
        } else if x.len() < N + 1 {
            return Err(InterpolationError::NotEnoughSamples(x.len(), N + 1));
        }

        let n = x.len();
        let mut s = vec![T::zero(); n];
        let mut coeffs_vec = vec![T::zero(); n];

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
                coeffs_vec[j] += aux;
                b *= x[i];
                b += s[j];
            }
        }

        // NOTE if x.len() > N then coeffs_vec[N + 1..]
        // will contain all zeros because we have
        // more samples than coefficients in the polynomial
        let mut coeffs = [T::zero(); N + 1];
        coeffs.copy_from_slice(&coeffs_vec[0..N + 1]);
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
