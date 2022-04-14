use curve25519_dalek::scalar::Scalar;

use rand_core::OsRng;
use sha2::{Digest, Sha512};

use crate::ring_signature::{RingSignature, RingSignatureLinked};
use crate::utils::hash_to_ristretto_point;

use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;
use curve25519_dalek::ristretto::RistrettoPoint;

#[derive(Debug)]
pub struct Signer {
    pub secret: Scalar,
}

impl Signer {
    #[allow(unused)]
    pub fn sign_message(
        &self,
        message: Vec<u8>,
        ring: &mut Vec<RistrettoPoint>,
        secret_idx: usize,
    ) -> RingSignature {
        let ring_len = ring.len() as usize;

        // Select alpha and pi randomly
        let alpha = Scalar::random(&mut OsRng);
        let pi = secret_idx;

        // Convert the ring and message to bytes (to be included in the hashes)
        let ring_vec = ring
            .iter()
            .flat_map(|e| e.compress().to_bytes())
            .collect::<Vec<u8>>();
        let prefix_bytes = [ring_vec.as_slice(), message.as_slice()].concat();

        // Initialize the responses vec to random values
        // i == pi will be replaced later with the correct response
        let mut responses = Vec::with_capacity(ring_len);
        for _ in 0..ring_len {
            responses.push(Scalar::random(&mut OsRng));
        }

        // Initialize c vector with zeros
        let mut c_vec: Vec<Scalar> = (0..ring_len).map(|_| Scalar::zero()).collect();

        // Calculate the first element (with index (pi + 1) mod ring_len) in the ring
        let hasher = Sha512::new()
            .chain(&prefix_bytes)
            .chain((alpha * RISTRETTO_BASEPOINT_POINT).compress().as_bytes());
        c_vec[(pi + 1) % ring_len] = Scalar::from_hash(hasher);

        // Iterate (ring_len - 1 times) to calculate remaining c values
        let mut current_idx = (pi + 1) % ring_len;
        for _ in 1..ring_len {
            let hasher = Sha512::new().chain(&prefix_bytes).chain(
                (responses[current_idx] * RISTRETTO_BASEPOINT_POINT
                    + c_vec[current_idx] * ring[current_idx])
                    .compress()
                    .to_bytes(),
            );

            current_idx = (current_idx + 1) % ring_len;
            c_vec[current_idx] = Scalar::from_hash(hasher);
        }

        // Calculate last response to close the ring of signatures
        responses[pi] = alpha - c_vec[pi] * self.secret;

        RingSignature {
            response_vec: responses,
            c1: c_vec[0],
            message: message,
            ring: ring.clone(),
        }
    }

    #[allow(unused)]
    pub fn sign_message_linked(
        &self,
        message: Vec<u8>,
        ring: &mut Vec<RistrettoPoint>,
        secret_idx: usize,
    ) -> RingSignatureLinked {
        let ring_len = ring.len() as usize;

        // Select alpha and pi randomly
        let alpha = Scalar::random(&mut OsRng);
        let pi = secret_idx;

        // Calculate key image
        let key_image = self.secret * hash_to_ristretto_point(ring[secret_idx]);

        // Convert the ring and message to bytes (to be included in the hashes)
        let ring_vec = ring
            .iter()
            .flat_map(|e| e.compress().to_bytes())
            .collect::<Vec<u8>>();
        let prefix_bytes = [ring_vec.as_slice(), message.as_slice()].concat();

        let hashed_signer_pubkey = hash_to_ristretto_point(ring[pi]);

        // Initialize the responses vec to random values
        // i == pi will be replaced later with the correct response
        let mut responses = Vec::with_capacity(ring_len);
        for _ in 0..ring_len {
            responses.push(Scalar::random(&mut OsRng));
        }

        // Initialize c vector with zeros
        let mut c_vec: Vec<Scalar> = (0..ring_len).map(|_| Scalar::zero()).collect();

        // Calculate the first element (with index (pi + 1) mod ring_len) in the ring
        let hasher = Sha512::new()
            .chain(&prefix_bytes)
            .chain((alpha * RISTRETTO_BASEPOINT_POINT).compress().as_bytes())
            .chain((alpha * hashed_signer_pubkey).compress().as_bytes());

        c_vec[(pi + 1) % ring_len] = Scalar::from_hash(hasher);

        // Iterate (ring_len - 1 times) to calculate remaining c values
        let mut current_idx = (pi + 1) % ring_len;
        for _ in 1..ring_len {
            let hashed_pubkey = hash_to_ristretto_point(ring[current_idx]);

            let hasher = Sha512::new()
                .chain(&prefix_bytes)
                .chain(
                    (responses[current_idx] * RISTRETTO_BASEPOINT_POINT
                        + c_vec[current_idx] * ring[current_idx])
                        .compress()
                        .to_bytes(),
                )
                .chain(
                    (responses[current_idx] * hashed_pubkey + c_vec[current_idx] * key_image)
                        .compress()
                        .as_bytes(),
                );

            current_idx = (current_idx + 1) % ring_len;
            c_vec[current_idx] = Scalar::from_hash(hasher);
        }

        // Calculate last response to close the ring of signatures
        responses[pi] = alpha - c_vec[pi] * self.secret;

        RingSignatureLinked {
            response_vec: responses,
            c1: c_vec[0],
            message: message,
            ring: ring.clone(),
            key_image,
        }
    }
}
