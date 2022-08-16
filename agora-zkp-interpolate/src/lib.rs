#![deny(warnings)]
#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]

use subtle::CtOption;
use thiserror::Error;

use std::ops::{AddAssign, Mul, MulAssign, Neg, SubAssign};

#[derive(Error, Debug)]
pub enum InterpolationError {
    #[error("unequal slice lengths: {0} and {1}")]
    InvalidInputLengths(usize, usize),
    #[error("attempted to invert a zero scalar")]
    TriedToInvertZero,
}

pub trait Interpolate {
    fn zero() -> Self
    where
        Self: Sized;
    fn one() -> Self
    where
        Self: Sized;
    fn from_u64(num: u64) -> Self
    where
        Self: Sized;
    fn inverse(&self) -> CtOption<Self>
    where
        Self: Sized;
}

pub fn interpolate<T>(x: &[T], y: &[T]) -> Result<Vec<T>, InterpolationError>
where
    T: Interpolate + Copy + Mul<Output = T> + Neg<Output = T> + AddAssign + SubAssign + MulAssign,
{
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
        // NOTE unwrap is always fine?
        let ff = <T as Interpolate>::inverse(&phi).unwrap();
        let mut b = T::one();
        for j in (0..n).rev() {
            let aux = b * ff * y[i];
            coeffs[j] += aux;
            b *= x[i];
            b += s[j];
        }
    }

    Ok(coeffs)
}
