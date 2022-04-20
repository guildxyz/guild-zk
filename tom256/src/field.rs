use elliptic_curve::bigint::{nlimbs, Limb, LimbUInt, U256, Encoding};
use elliptic_curve::subtle::{Choice, ConditionallySelectable};

use crate::int_ops::*;

use std::convert::From;

/*
const MODULUS: U256 =
    U256::from_be_hex("3fffffffc000000040000000000000002ae382c7957cc4ff9713c3d82bc47d3af");
*/

pub trait ModularNumber {
    const MODULUS: U256;
}

const MOD_SMALL: U256 = U256::from_be_hex("00000000000000000000000000000000000000000000000000000000000003e8"); // 1000
const MOD_BIG: U256 =   U256::from_be_hex("ffffffff00000001000000000000000000000000ffffffffffffffffffffffff"); // original


#[derive(Debug)]
pub struct FieldElement(U256);

#[derive(Debug)]
pub struct Scalar(U256);

impl FieldElement {
    #[allow(unused)]
    pub fn new(number: u32) -> Self {
        let some_int = U256::from(number);

        if some_int > Self::MODULUS {
            Self(some_int.reduce(&Self::MODULUS).unwrap())
        }
        else {
            Self(some_int)
        }
    }
}

impl From<[u8; 32]> for FieldElement {
    fn from(bytes: [u8; 32]) -> FieldElement {
        Self(U256::from_be_bytes(bytes))
    }
}

impl From<U256> for FieldElement {
    // NOTE: Not constant time conversion
    fn from(uint: U256) -> FieldElement {
        if uint > Self::MODULUS {
            Self(uint.reduce(&Self::MODULUS).unwrap())
        }
        else {
            Self(uint)
        }
    }
}

pub trait Modular: From<U256> {
    const MODULUS: U256;
    const ORDER: U256;

    fn add(&self, other: &Self) -> Self;
    fn neg(&self) -> Self;
    fn sub(&self, other: &Self) -> Self;
    fn mul(&self, other: &Self) -> Self;

}

impl Modular for FieldElement {
    const MODULUS: U256 = MOD_SMALL;
    const ORDER: U256 = MOD_SMALL;

    fn add(&self, other: &Self) -> Self {
        Self(self.0.add_mod(&other.0, &Self::MODULUS))
    }

    fn neg(&self) -> Self {
        Self(self.0.neg_mod(&Self::MODULUS))
    }

    fn sub(&self, other: &Self) -> Self {
        Self(self.0.sub_mod(&other.0, &Self::MODULUS))
    }

    fn mul(&self, other: &Self) -> Self {
        let (lo, hi) = self.0.mul_wide(&other.0);
        reduce_impl::<Self>(lo, hi, false)
    }
}

fn reduce_impl<T: Modular>(lo: U256, hi: U256, modulus_minus_one: bool) -> T {
    let mod_limbs: [LimbUInt; nlimbs!(256)] = T::MODULUS.to_uint_array();
    let neg_mod: [u64; 4] = [!mod_limbs[0] + 1, !mod_limbs[1], !mod_limbs[2], !mod_limbs[3]];

    let neg_modulus0 = if modulus_minus_one {
        neg_mod[0] + 1
    } else {
        neg_mod[0]
    };
    let modulus = if modulus_minus_one {
        T::ORDER.wrapping_sub(&U256::ONE)
    } else {
        T::ORDER
    };

    let hi_limbs: [LimbUInt; nlimbs!(256)] = hi.to_uint_array();

    let lo_limbs: [LimbUInt; nlimbs!(256)] = lo.to_uint_array();
    let n0 = hi_limbs[0];
    let n1 = hi_limbs[1];
    let n2 = hi_limbs[2];
    let n3 = hi_limbs[3];

    // Reduce 512 bits into 385.
    // m[0..6] = self[0..3] + n[0..3] * neg_modulus.
    let c0 = lo_limbs[0];
    let c1 = 0;
    let c2 = 0;
    let (c0, c1) = muladd_fast(n0, neg_modulus0, c0, c1);
    let (m0, c0, c1) = (c0, c1, 0);
    let (c0, c1) = sumadd_fast(lo_limbs[1], c0, c1);
    let (c0, c1, c2) = muladd(n1, neg_modulus0, c0, c1, c2);
    let (c0, c1, c2) = muladd(n0, neg_mod[1], c0, c1, c2);
    let (m1, c0, c1, c2) = (c0, c1, c2, 0);
    let (c0, c1, c2) = sumadd(lo_limbs[2], c0, c1, c2);
    let (c0, c1, c2) = muladd(n2, neg_modulus0, c0, c1, c2);
    let (c0, c1, c2) = muladd(n1, neg_mod[1], c0, c1, c2);
    let (c0, c1, c2) = sumadd(n0, c0, c1, c2);
    let (m2, c0, c1, c2) = (c0, c1, c2, 0);
    let (c0, c1, c2) = sumadd(lo_limbs[3], c0, c1, c2);
    let (c0, c1, c2) = muladd(n3, neg_modulus0, c0, c1, c2);
    let (c0, c1, c2) = muladd(n2, neg_mod[1], c0, c1, c2);
    let (c0, c1, c2) = sumadd(n1, c0, c1, c2);
    let (m3, c0, c1, c2) = (c0, c1, c2, 0);
    let (c0, c1, c2) = muladd(n3, neg_mod[1], c0, c1, c2);
    let (c0, c1, c2) = sumadd(n2, c0, c1, c2);
    let (m4, c0, c1, _c2) = (c0, c1, c2, 0);
    let (c0, c1) = sumadd_fast(n3, c0, c1);
    let (m5, c0, _c1) = (c0, c1, 0);
    debug_assert!(c0 <= 1);
    let m6 = c0;

    // Reduce 385 bits into 258.
    // p[0..4] = m[0..3] + m[4..6] * neg_modulus.
    let c0 = m0;
    let c1 = 0;
    let c2 = 0;
    let (c0, c1) = muladd_fast(m4, neg_modulus0, c0, c1);
    let (p0, c0, c1) = (c0, c1, 0);
    let (c0, c1) = sumadd_fast(m1, c0, c1);
    let (c0, c1, c2) = muladd(m5, neg_modulus0, c0, c1, c2);
    let (c0, c1, c2) = muladd(m4, neg_mod[1], c0, c1, c2);
    let (p1, c0, c1) = (c0, c1, 0);
    let (c0, c1, c2) = sumadd(m2, c0, c1, c2);
    let (c0, c1, c2) = muladd(m6, neg_modulus0, c0, c1, c2);
    let (c0, c1, c2) = muladd(m5, neg_mod[1], c0, c1, c2);
    let (c0, c1, c2) = sumadd(m4, c0, c1, c2);
    let (p2, c0, c1, _c2) = (c0, c1, c2, 0);
    let (c0, c1) = sumadd_fast(m3, c0, c1);
    let (c0, c1) = muladd_fast(m6, neg_mod[1], c0, c1);
    let (c0, c1) = sumadd_fast(m5, c0, c1);
    let (p3, c0, _c1) = (c0, c1, 0);
    let p4 = c0 + m6;
    debug_assert!(p4 <= 2);

    // Reduce 258 bits into 256.
    // r[0..3] = p[0..3] + p[4] * neg_modulus.
    let mut c = (p0 as u128) + (neg_modulus0 as u128) * (p4 as u128);
    let r0 = (c & 0xFFFFFFFFFFFFFFFFu128) as u64;
    c >>= 64;
    c += (p1 as u128) + (neg_mod[1] as u128) * (p4 as u128);
    let r1 = (c & 0xFFFFFFFFFFFFFFFFu128) as u64;
    c >>= 64;
    c += (p2 as u128) + (p4 as u128);
    let r2 = (c & 0xFFFFFFFFFFFFFFFFu128) as u64;
    c >>= 64;
    c += p3 as u128;
    let r3 = (c & 0xFFFFFFFFFFFFFFFFu128) as u64;
    c >>= 64;

    // Final reduction of r.
    let r = U256::from([r0, r1, r2, r3]);
    let (r2, underflow) = r.sbb(&modulus, Limb::ZERO);
    let high_bit = Choice::from(c as u8);
    let underflow = Choice::from((underflow.0 >> 63) as u8);

    T::from(U256::conditional_select(&r, &r2, !underflow | high_bit))
}

