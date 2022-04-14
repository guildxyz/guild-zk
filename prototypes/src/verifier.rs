use curve25519_dalek::constants::{BASEPOINT_ORDER, RISTRETTO_BASEPOINT_POINT};
use curve25519_dalek::scalar::Scalar;

use sha2::{Digest, Sha512};

use crate::ring_signature::{RingSignature, RingSignatureLinked};
use crate::utils::hash_to_ristretto_point;

pub struct Verifier;

impl Verifier {
    #[allow(unused)]
    pub fn verify(ring_signature: RingSignature) -> bool {
        let mut ring_vec = ring_signature
            .ring
            .iter()
            .flat_map(|e| e.compress().to_bytes())
            .collect::<Vec<u8>>();

        let prefix_bytes = [ring_vec.as_slice(), ring_signature.message.as_slice()].concat();

        let mut c_curr = ring_signature.c1;
        for (i, ring_element) in ring_signature.ring.iter().enumerate() {
            let hasher = Sha512::new().chain(&prefix_bytes).chain(
                (ring_signature.response_vec[i] * RISTRETTO_BASEPOINT_POINT
                    + c_curr * ring_element)
                    .compress()
                    .as_bytes(),
            );

            c_curr = Scalar::from_hash(hasher);
        }

        c_curr == ring_signature.c1
    }

    #[allow(unused)]
    pub fn verify_linked(ring_signature: RingSignatureLinked) -> bool {
        if BASEPOINT_ORDER * ring_signature.key_image != Scalar::zero() * RISTRETTO_BASEPOINT_POINT
        {
            return false;
        }

        let mut ring_vec = ring_signature
            .ring
            .iter()
            .flat_map(|e| e.compress().to_bytes())
            .collect::<Vec<u8>>();

        let prefix_bytes = [ring_vec.as_slice(), ring_signature.message.as_slice()].concat();

        let mut c_curr = ring_signature.c1;
        for (i, ring_element) in ring_signature.ring.iter().enumerate() {
            let hashed_pubkey = hash_to_ristretto_point(*ring_element);

            let hasher = Sha512::new()
                .chain(&prefix_bytes)
                .chain(
                    (ring_signature.response_vec[i] * RISTRETTO_BASEPOINT_POINT
                        + c_curr * ring_element)
                        .compress()
                        .as_bytes(),
                )
                .chain(
                    (ring_signature.response_vec[i] * hashed_pubkey
                        + c_curr * ring_signature.key_image)
                        .compress()
                        .as_bytes(),
                );

            c_curr = Scalar::from_hash(hasher);
        }

        c_curr == ring_signature.c1
    }
}
