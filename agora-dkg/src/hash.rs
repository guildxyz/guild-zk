// NOTE check domain separation tags
const G1_DST: &[u8] = b"ThIs2Is A8rAnDoM DoMaIn SePaRaTiOn+TaG fOr G1";
const G2_DST: &[u8] = b"ThIs#Is_A rAnDoM!DoMaIn SePaRaTiOn9TaG fOr G2";

use bls::hash_to_curve::{ExpandMsgXmd, HashToCurve, HashToField};
use bls::{G1Affine, G1Projective, G2Affine, G2Projective, Scalar};
use generic_array::GenericArray;
use sha3::digest::FixedOutput;
use sha3::{Digest, Sha3_256, Sha3_384};

pub fn hash_to_g1(msg: &[u8]) -> G1Affine {
    let g1 = <G1Projective as HashToCurve<ExpandMsgXmd<Sha3_256>>>::hash_to_curve(msg, G1_DST);
    G1Affine::from(g1)
}

pub fn hash_to_g2(msg: &[u8]) -> G2Affine {
    let g2 = <G2Projective as HashToCurve<ExpandMsgXmd<Sha3_256>>>::hash_to_curve(msg, G2_DST);
    G2Affine::from(g2)
}

pub fn hash_to_fp(msg: &[u8]) -> Scalar {
    let mut hasher = Sha3_384::new();
    hasher.update(msg);
    let mut hash = GenericArray::from_exact_iter([0u8; 48].into_iter()).unwrap();
    hasher.finalize_into(&mut hash);
    Scalar::from_okm(&hash)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn h2g1() {
        let msg = b"hello world";
        let g1 = hash_to_g1(msg);
        assert!(bool::from(g1.is_on_curve()));

        let msg = b"000fa0fdaffdaaeeeee0012342aaaaaaaaaa098756224235635242342325";
        let g1 = hash_to_g1(msg);
        assert!(bool::from(g1.is_on_curve()));
    }

    #[test]
    fn h2g2() {
        let msg = b"hello world";
        let g2 = hash_to_g2(msg);
        assert!(bool::from(g2.is_on_curve()));

        let msg = b"000fa0fdaffdaaeeeee0012342aaaaaaaaaa098756224235635242342325";
        let g2 = hash_to_g2(msg);
        assert!(bool::from(g2.is_on_curve()));
    }

    #[test]
    fn h2fp() {
        let msg = b"hello world";
        let _scalar = hash_to_fp(msg); // no error, a scalar was successfully created
        let msg = b"another test and three pugs";
        let _scalar = hash_to_fp(msg); // no error, a scalar was successfully created
        let msg = b"000fa0fdaffdaaeeeee0012342aaaaaaaaaa098756224235635242342325";
        let _scalar = hash_to_fp(msg); // no error, a scalar was successfully created
    }
}
