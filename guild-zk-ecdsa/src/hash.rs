use ark_ec::models::short_weierstrass::{Affine, SWCurveConfig};
use ark_ff::PrimeField;
use ark_serialize::CanonicalSerialize;
use sha3::{Digest, Sha3_256};

pub fn hash_points<C: SWCurveConfig>(hash_id: &[u8], points: &[&Affine<C>]) -> C::ScalarField {
    let mut input = hash_id.to_vec();
    for &point in points {
        point
            .serialize_compressed(&mut input)
            .expect("this operation never fails; qed");
    }

    let mut hasher = Sha3_256::new();
    hasher.update(&input);

    let mut digest = [0u8; 32];
    hasher.finalize_into((&mut digest).into());

    C::ScalarField::from_le_bytes_mod_order(&digest)
}
