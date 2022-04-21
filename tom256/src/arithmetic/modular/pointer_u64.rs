use bigint::subtle::{Choice, ConditionallySelectable};
use bigint::{Limb, U256};

pub trait Modular: Sized {
    const MODULUS: U256;
    const NEG_MOD: [u64; 4] = [
        !Self::MODULUS.limbs()[0].0 + 1,
        !Self::MODULUS.limbs()[1].0,
        !Self::MODULUS.limbs()[2].0,
        !Self::MODULUS.limbs()[3].0,
    ];

    fn new(number: U256) -> Self;

    fn inner(&self) -> &U256;

    fn add(&self, other: &Self) -> Self {
        Self::new(self.inner().add_mod(&other.inner(), &Self::MODULUS))
    }

    fn neg(&self) -> Self {
        Self::new(self.inner().neg_mod(&Self::MODULUS))
    }

    fn sub(&self, other: &Self) -> Self {
        Self::new(self.inner().sub_mod(&other.inner(), &Self::MODULUS))
    }

    fn mul(&self, other: &Self) -> Self {
        let (lo, hi) = self.inner().mul_wide(&other.inner());
        Self::reduce(lo, hi)
    }

    fn reduce(lo: U256, hi: U256) -> Self {
        let hi_limbs: [u64; 4] = hi.to_uint_array();
        let lo_limbs: [u64; 4] = lo.to_uint_array();

        let n0 = hi_limbs[0];
        let n1 = hi_limbs[1];
        let n2 = hi_limbs[2];
        let n3 = hi_limbs[3];
        // Reduce 512 bits into 385.
        // m[0..6] = self[0..3] + n[0..3] * neg_modulus.
        let c0 = lo_limbs[0];
        let c1 = 0;
        let c2 = 0;
        let (c0, c1) = muladd_fast(n0, Self::NEG_MOD[0], c0, c1);
        let (m0, c0, c1) = (c0, c1, 0);
        let (c0, c1) = sumadd_fast(lo_limbs[1], c0, c1);
        let (c0, c1, c2) = muladd(n1, Self::NEG_MOD[0], c0, c1, c2);
        let (c0, c1, c2) = muladd(n0, Self::NEG_MOD[1], c0, c1, c2);
        let (m1, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = sumadd(lo_limbs[2], c0, c1, c2);
        let (c0, c1, c2) = muladd(n2, Self::NEG_MOD[0], c0, c1, c2);
        let (c0, c1, c2) = muladd(n1, Self::NEG_MOD[1], c0, c1, c2);
        let (c0, c1, c2) = sumadd(n0, c0, c1, c2);
        let (m2, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = sumadd(lo_limbs[3], c0, c1, c2);
        let (c0, c1, c2) = muladd(n3, Self::NEG_MOD[0], c0, c1, c2);
        let (c0, c1, c2) = muladd(n2, Self::NEG_MOD[1], c0, c1, c2);
        let (c0, c1, c2) = sumadd(n1, c0, c1, c2);
        let (m3, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = muladd(n3, Self::NEG_MOD[1], c0, c1, c2);
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
        let (c0, c1) = muladd_fast(m4, Self::NEG_MOD[0], c0, c1);
        let (p0, c0, c1) = (c0, c1, 0);
        let (c0, c1) = sumadd_fast(m1, c0, c1);
        let (c0, c1, c2) = muladd(m5, Self::NEG_MOD[0], c0, c1, c2);
        let (c0, c1, c2) = muladd(m4, Self::NEG_MOD[1], c0, c1, c2);
        let (p1, c0, c1) = (c0, c1, 0);
        let (c0, c1, c2) = sumadd(m2, c0, c1, c2);
        let (c0, c1, c2) = muladd(m6, Self::NEG_MOD[0], c0, c1, c2);
        let (c0, c1, c2) = muladd(m5, Self::NEG_MOD[1], c0, c1, c2);
        let (c0, c1, c2) = sumadd(m4, c0, c1, c2);
        let (p2, c0, c1, _c2) = (c0, c1, c2, 0);
        let (c0, c1) = sumadd_fast(m3, c0, c1);
        let (c0, c1) = muladd_fast(m6, Self::NEG_MOD[1], c0, c1);
        let (c0, c1) = sumadd_fast(m5, c0, c1);
        let (p3, c0, _c1) = (c0, c1, 0);
        let p4 = c0 + m6;
        debug_assert!(p4 <= 2);

        // Reduce 258 bits into 256.
        // r[0..3] = p[0..3] + p[4] * neg_modulus.
        let mut c = (p0 as u128) + (Self::NEG_MOD[0] as u128) * (p4 as u128);
        let r0 = (c & 0xFFFFFFFFFFFFFFFFu128) as u64;
        c >>= 64;
        c += (p1 as u128) + (Self::NEG_MOD[1] as u128) * (p4 as u128);
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
        let (r2, underflow) = r.sbb(&Self::MODULUS, Limb::ZERO);
        let high_bit = Choice::from(c as u8);
        let underflow = Choice::from((underflow.0 >> 63) as u8);

        Self::new(U256::conditional_select(&r, &r2, !underflow | high_bit))
    }
}

/// Constant-time comparison.
#[inline(always)]
fn ct_less(a: u64, b: u64) -> u64 {
    // Do not convert to Choice since it is only used internally,
    // and we don't want loss of performance.
    (a < b) as u64
}

/// Add a to the number defined by (c0,c1,c2). c2 must never overflow.
fn sumadd(a: u64, c0: u64, c1: u64, c2: u64) -> (u64, u64, u64) {
    let new_c0 = c0.wrapping_add(a); // overflow is handled on the next line
    let over = ct_less(new_c0, a);
    let new_c1 = c1.wrapping_add(over); // overflow is handled on the next line
    let new_c2 = c2 + ct_less(new_c1, over); // never overflows by contract
    (new_c0, new_c1, new_c2)
}

/// Add a to the number defined by (c0,c1). c1 must never overflow, c2 must be zero.
fn sumadd_fast(a: u64, c0: u64, c1: u64) -> (u64, u64) {
    let new_c0 = c0.wrapping_add(a); // overflow is handled on the next line
    let new_c1 = c1 + ct_less(new_c0, a); // never overflows by contract (verified the next line)
    debug_assert!((new_c1 != 0) | (new_c0 >= a));
    (new_c0, new_c1)
}

/// Add a*b to the number defined by (c0,c1,c2). c2 must never overflow.
fn muladd(a: u64, b: u64, c0: u64, c1: u64, c2: u64) -> (u64, u64, u64) {
    let t = (a as u128) * (b as u128);
    let th = (t >> 64) as u64; // at most 0xFFFFFFFFFFFFFFFE
    let tl = t as u64;

    let new_c0 = c0.wrapping_add(tl); // overflow is handled on the next line
    let new_th = th + if new_c0 < tl { 1 } else { 0 }; // at most 0xFFFFFFFFFFFFFFFF
    let new_c1 = c1.wrapping_add(new_th); // overflow is handled on the next line
    let new_c2 = c2 + ct_less(new_c1, new_th); // never overflows by contract (verified in the next line)
    debug_assert!((new_c1 >= new_th) || (new_c2 != 0));
    (new_c0, new_c1, new_c2)
}

/// Add a*b to the number defined by (c0,c1). c1 must never overflow.
fn muladd_fast(a: u64, b: u64, c0: u64, c1: u64) -> (u64, u64) {
    let t = (a as u128) * (b as u128);
    let th = (t >> 64) as u64; // at most 0xFFFFFFFFFFFFFFFE
    let tl = t as u64;

    let new_c0 = c0.wrapping_add(tl); // overflow is handled on the next line
    let new_th = th + ct_less(new_c0, tl); // at most 0xFFFFFFFFFFFFFFFF
    let new_c1 = c1 + new_th; // never overflows by contract (verified in the next line)
    debug_assert!(new_c1 >= new_th);
    (new_c0, new_c1)
}
