#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]

#[cfg(test)]
mod macros;

#[cfg(any(test, feature = "bls-scalar"))]
mod bls_scalar;
#[cfg(any(test, feature = "k256-scalar"))]
mod k256_scalar;
mod polynomial;

pub use polynomial::Polynomial;

use subtle::CtOption;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
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
