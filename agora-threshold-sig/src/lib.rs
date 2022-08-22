#![allow(non_snake_case)]
#[deny(clippy::all)]
#[deny(clippy::dbg_macro)]
mod encrypt;
mod hash;
mod participant;
mod shares;
#[cfg(test)]
mod tests;

const FP_BYTES: usize = 32;
const G2_BYTES: usize = 96; // compressed
