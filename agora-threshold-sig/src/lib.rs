#![allow(non_snake_case)]
#[deny(clippy::all)]
#[deny(clippy::dbg_macro)]
// this is just a test documentation
// This is the public version $A(x)$ of the privately generated
// polynomial $a(x)$ with degree $t-1$, where $t$ is the threshold.
//
// The private polynomial is generated over a finite field $\mathbb{F}_p$
//
// $$a(x) = a_0 + a_1x +\ldots + a_{t - 1}x^{t - 1}$$
//
// with $x, a_i\in\mathbb{F_p}\ \forall i$. The public polynomial is defined as
//
// $$A(x) = A_0 + A_1x +\ldots + A_{t - 1}x^{t - 1}$$
//
// with $x\in\mathbb{F_p}$ and $A_i = g_2^{a_i}\in\mathbb{G_2}\ \forall i$.
mod address;
pub mod hash;
mod keypair;
mod node;
mod share;

pub use node::{Discovery, Finalized, Node, Parameters, ShareCollection};

pub fn sig_verify(msg: &[u8], vk: &bls::G2Affine, sig: &bls::G1Affine) -> bool {
    let msg_hash_g1 = hash::hash_to_g1(msg);
    bls::pairing(&msg_hash_g1, vk) == bls::pairing(sig, &bls::G2Affine::generator())
}
