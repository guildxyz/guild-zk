use curve25519_dalek::scalar::Scalar;

use curve25519_dalek::ristretto::RistrettoPoint;

#[allow(unused)]
pub struct RingSignature {
    pub c1: Scalar,
    pub response_vec: Vec<Scalar>,
    pub message: Vec<u8>,
    pub ring: Vec<RistrettoPoint>,
}

#[allow(unused)]
pub struct RingSignatureLinked {
    pub c1: Scalar,
    pub response_vec: Vec<Scalar>,
    pub message: Vec<u8>,
    pub ring: Vec<RistrettoPoint>,
    pub key_image: RistrettoPoint,
}
