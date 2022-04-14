use sha2::Sha512;

use curve25519_dalek::edwards::{CompressedEdwardsY, EdwardsPoint};
use curve25519_dalek::ristretto::RistrettoPoint;
use curve25519_dalek::scalar::Scalar;

use ed25519_dalek::{PublicKey, SecretKey};

use rand_core::{CryptoRng, OsRng, RngCore};

#[allow(unused)]
pub fn pubkey_to_edwards(pubkey: &PublicKey) -> EdwardsPoint {
    CompressedEdwardsY::from_slice(pubkey.as_bytes())
        .decompress()
        .unwrap()
}

#[allow(unused)]
pub fn secret_key_to_scalar(secret: &SecretKey) -> Scalar {
    Scalar::from_bytes_mod_order(secret.to_bytes())
}

#[allow(unused)]
pub fn random_ristretto_point<T: RngCore + CryptoRng>(rng: &mut T) -> RistrettoPoint {
    RistrettoPoint::random(rng)
}

#[allow(unused)]
pub fn hash_to_ristretto_point(point: RistrettoPoint) -> RistrettoPoint {
    RistrettoPoint::hash_from_bytes::<Sha512>(point.compress().as_bytes())
}

pub fn generate_ring_containing_address(
    signer_pubkey: &RistrettoPoint,
    ring_len: usize,
) -> (Vec<RistrettoPoint>, usize) {
    let mut ring = Vec::<RistrettoPoint>::new();
    for _ in 0..ring_len {
        ring.push(random_ristretto_point(&mut OsRng));
    }

    let pi = OsRng.next_u64() as usize % ring_len;
    ring[pi] = *signer_pubkey;

    (ring, pi)
}
