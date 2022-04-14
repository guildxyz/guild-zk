mod ring_signature;
mod signer;
mod utils;
mod verifier;

use curve25519_dalek::scalar::Scalar;

use std::time::Instant;

use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;
use curve25519_dalek::ristretto::RistrettoPoint;
//use curve25519_dalek::constants::ED25519_BASEPOINT_POINT;
//use curve25519_dalek::edwards::EdwardsPoint;

use crate::signer::Signer;
use crate::verifier::Verifier;

use rand_core::OsRng;
use utils::*;

fn main() {
    let now = Instant::now();
    let mut csprng = OsRng;

    // Generate random signer
    let signer_secret = Scalar::random(&mut csprng);
    let signer_pubkey: RistrettoPoint = signer_secret * RISTRETTO_BASEPOINT_POINT;

    let signer = Signer {
        secret: signer_secret,
    };

    // Ring of random pubkeys
    //  signer pubkey replaces one of them
    let ring_len = 10;
    let (mut ring, secret_idx) = generate_ring_containing_address(&signer_pubkey, ring_len);

    let message = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // Sign incorrect position
    //let ring_signature = signer.sign_message(message, &mut ring, (secret_idx + 1) % ring_len);

    //let ring_signature = signer.sign_message(message, &mut ring, secret_idx);
    let ring_signature = signer.sign_message_linked(message, &mut ring, secret_idx);

    //if Verifier::verify(ring_signature) {
    if Verifier::verify_linked(ring_signature) {
        println!("yay");
    } else {
        println!("nay");
    }

    println!("{} ms", now.elapsed().as_millis());
}
