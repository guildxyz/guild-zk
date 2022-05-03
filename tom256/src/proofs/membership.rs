use crate::arithmetic::{Point, Scalar};
use crate::pedersen::*;
use crate::{Curve, Cycle};

use std::marker::PhantomData;

pub struct MembershipProof<CC, C, const SEC: usize> {
    cl: [Point<CC>; SEC],
    ca: [Point<CC>; SEC],
    cb: [Point<CC>; SEC],
    cd: [Point<CC>; SEC],
    f: [Scalar<CC>; SEC],
    za: [Scalar<CC>; SEC],
    zb: [Scalar<CC>; SEC],
    zd: Scalar<CC>,
    base_curve: PhantomData<C>,
}

impl<CC: Cycle<C>, C: Curve, const SEC: usize> MembershipProof<CC, C, SEC> {
    const HASH_ID: &'static [u8] = b"membership-proof";

    pub fn construct(
        pedersen_generator: &PedersenGenerator<CC>,
        commitment_to_key: &PedersenCommitment<CC>,
        index: usize,
        // NOTE this is just the public address represented as a scalar (only
        // 160 bit, so it should fit unless C::PRIME_MODULUS is less than
        // 2^160)
        mut ring: Vec<Scalar<CC>>,
    ) -> Result<Self, String> {
        let n = pad_ring_to_2n(&mut ring)?; // log2(ring.len())
                                            //
        let mut a_vec = Vec::<Scalar<CC>>::with_capacity(n);
        let mut l_vec = Vec::<Scalar<CC>>::with_capacity(n);
        let mut r_vec = Vec::<Scalar<CC>>::with_capacity(n);
        let mut s_vec = Vec::<Scalar<CC>>::with_capacity(n);
        let mut t_vec = Vec::<Scalar<CC>>::with_capacity(n);

        let mut tmp_index = index;
        for i in 0..n {
            l_vec.push(tmp_index % 2);
            tmp_index /= 2;
            // TODO fill a, r, s, t with Scalars (?) modulo CC::ORDER (?)
            // CC::ORDER == C::PRIME_MODULUS -> cast Scalar<C> into Scalar<CC> ????
        }
        todo!();
    }
}

fn pad_ring_to_2n<C: Curve>(ring: &mut Vec<Scalar<C>>) -> Result<usize, String> {
    // TODO ensure that the ring is not empty
    if ring.is_empty() {
        Err("empty ring".to_string())
    } else {
        let log_2_ring_len = ring.len().log2();
        let pow_2_ring_len = 2usize.pow(log_2_ring_len);
        // pow_2_ring_len is always less than or equal to keys.len()
        // because log2 always rounds down
        if ring.len() != pow_2_ring_len {
            for _ in 0..pow_2_ring_len * 2 - ring.len() {
                ring.push(ring[0])
            }
            Ok((log_2_ring_len + 1) as usize)
        } else {
            Ok(log_2_ring_len as usize)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Tom256k1;

    #[test]
    fn pad_ring() {
        let mut ring = Vec::<Scalar<Tom256k1>>::new();
        assert!(pad_ring_to_2n(&mut ring).is_err());
        ring.push(Scalar::ONE);
        assert_eq!(pad_ring_to_2n(&mut ring), Ok(0));
        assert_eq!(ring.len(), 1);
        ring.push(Scalar::ZERO);
        assert_eq!(pad_ring_to_2n(&mut ring), Ok(1));
        assert_eq!(ring.len(), 2);
        ring.push(Scalar::ZERO);
        assert_eq!(pad_ring_to_2n(&mut ring), Ok(2));
        assert_eq!(ring.len(), 4);
        assert_eq!(ring[3], Scalar::ONE);
        for _ in 0..5 {
            ring.push(Scalar::ZERO);
        }
        assert_eq!(ring.len(), 9);
        assert_eq!(pad_ring_to_2n(&mut ring), Ok(4));
        assert_eq!(ring.len(), 16);
        assert_eq!(ring[15], Scalar::ONE);
    }
}
