#![deny(warnings)]
#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]

#[cfg(test)]
mod macros;

#[cfg(any(test, feature = "bls-curve"))]
mod bls_curve;
#[cfg(any(test, feature = "k256-curve"))]
mod k256_curve;
mod polynomial;

pub use polynomial::Polynomial;

use subtle::CtOption;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
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

#[cfg(test)]
pub trait GroupElement {
    fn generator() -> Self
    where
        Self: Sized;

    fn identity() -> Self
    where
        Self: Sized;
}
