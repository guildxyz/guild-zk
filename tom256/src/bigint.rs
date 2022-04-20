//! Wide scalar (64-bit limbs)

// super::MODULUS = ORDER ?? so why use both
use super::{Scalar, MODULUS};
use crate::ORDER;
use crate::int_ops::*;
use elliptic_curve::{
    bigint::{Limb, U256, U512},
    subtle::{Choice, ConditionallySelectable},
};

/// Limbs of 2^256 minus the secp256k1 order.
const NEG_MODULUS: [u64; 4] = [!MODULUS[0] + 1, !MODULUS[1], !MODULUS[2], !MODULUS[3]];

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct WideScalar(pub(super) U512);

impl WideScalar {
    pub const fn from_bytes(bytes: &[u8; 64]) -> Self {
        Self(U512::from_be_slice(bytes))
    }

    /// Multiplies two scalars without modulo reduction, producing up to a 512-bit scalar.
    #[inline(always)] // only used in Scalar::mul(), so won't cause binary bloat
    pub fn mul_wide(a: &Scalar, b: &Scalar) -> Self {
        let a = a.0.to_uint_array();
        let b = b.0.to_uint_array();

        // 160 bit accumulator.
        let c0 = 0;
        let c1 = 0;
        let c2 = 0;

        // l[0..7] = a[0..3] * b[0..3].
        let (c0, c1) = muladd_fast(a[0], b[0], c0, c1);
        let (l0, c0, c1) = (c0, c1, 0);
        let (c0, c1, c2) = muladd(a[0], b[1], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[1], b[0], c0, c1, c2);
        let (l1, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = muladd(a[0], b[2], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[1], b[1], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[2], b[0], c0, c1, c2);
        let (l2, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = muladd(a[0], b[3], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[1], b[2], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[2], b[1], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[3], b[0], c0, c1, c2);
        let (l3, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = muladd(a[1], b[3], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[2], b[2], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[3], b[1], c0, c1, c2);
        let (l4, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = muladd(a[2], b[3], c0, c1, c2);
        let (c0, c1, c2) = muladd(a[3], b[2], c0, c1, c2);
        let (l5, c0, c1, _c2) = (c0, c1, c2, 0);
        let (c0, c1) = muladd_fast(a[3], b[3], c0, c1);
        let (l6, c0, _c1) = (c0, c1, 0);
        let l7 = c0;

        Self(U512::from_uint_array([l0, l1, l2, l3, l4, l5, l6, l7]))
    }

    fn reduce(&self) -> Scalar {
        let neg_modulus0 = NEG_MODULUS[0];
        let modulus = ORDER;

        let w = self.0.to_uint_array();
        let n0 = w[4];
        let n1 = w[5];
        let n2 = w[6];
        let n3 = w[7];

        // Reduce 512 bits into 385.
        // m[0..6] = self[0..3] + n[0..3] * neg_modulus.
        let c0 = w[0];
        let c1 = 0;
        let c2 = 0;
        let (c0, c1) = muladd_fast(n0, neg_modulus0, c0, c1);
        let (m0, c0, c1) = (c0, c1, 0);
        let (c0, c1) = sumadd_fast(w[1], c0, c1);
        let (c0, c1, c2) = muladd(n1, neg_modulus0, c0, c1, c2);
        let (c0, c1, c2) = muladd(n0, NEG_MODULUS[1], c0, c1, c2);
        let (m1, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = sumadd(w[2], c0, c1, c2);
        let (c0, c1, c2) = muladd(n2, neg_modulus0, c0, c1, c2);
        let (c0, c1, c2) = muladd(n1, NEG_MODULUS[1], c0, c1, c2);
        let (c0, c1, c2) = sumadd(n0, c0, c1, c2);
        let (m2, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = sumadd(w[3], c0, c1, c2);
        let (c0, c1, c2) = muladd(n3, neg_modulus0, c0, c1, c2);
        let (c0, c1, c2) = muladd(n2, NEG_MODULUS[1], c0, c1, c2);
        let (c0, c1, c2) = sumadd(n1, c0, c1, c2);
        let (m3, c0, c1, c2) = (c0, c1, c2, 0);
        let (c0, c1, c2) = muladd(n3, NEG_MODULUS[1], c0, c1, c2);
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
        let (c0, c1, c2) = muladd(m4, NEG_MODULUS[1], c0, c1, c2);
        let (p1, c0, c1) = (c0, c1, 0);
        let (c0, c1, c2) = sumadd(m2, c0, c1, c2);
        let (c0, c1, c2) = muladd(m6, neg_modulus0, c0, c1, c2);
        let (c0, c1, c2) = muladd(m5, NEG_MODULUS[1], c0, c1, c2);
        let (c0, c1, c2) = sumadd(m4, c0, c1, c2);
        let (p2, c0, c1, _c2) = (c0, c1, c2, 0);
        let (c0, c1) = sumadd_fast(m3, c0, c1);
        let (c0, c1) = muladd_fast(m6, NEG_MODULUS[1], c0, c1);
        let (c0, c1) = sumadd_fast(m5, c0, c1);
        let (p3, c0, _c1) = (c0, c1, 0);
        let p4 = c0 + m6;
        debug_assert!(p4 <= 2);

        // Reduce 258 bits into 256.
        // r[0..3] = p[0..3] + p[4] * neg_modulus.
        let mut c = (p0 as u128) + (neg_modulus0 as u128) * (p4 as u128);
        let r0 = (c & 0xFFFFFFFFFFFFFFFFu128) as u64;
        c >>= 64;
        c += (p1 as u128) + (NEG_MODULUS[1] as u128) * (p4 as u128);
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
        Scalar(U256::conditional_select(&r, &r2, !underflow | high_bit))
    }
}
