use generic_array::GenericArray;
use k256::ecdsa::signature::Signer;
use k256::ecdsa::{Signature, SigningKey, VerifyingKey};
use k256::elliptic_curve::group::GroupEncoding;
use k256::elliptic_curve::ops::Reduce;
use k256::elliptic_curve::rand_core::RngCore;
use k256::elliptic_curve::sec1::{Coordinates, ToEncodedPoint};
use k256::elliptic_curve::{AffineXCoordinate, Field, PrimeField};
use k256::{AffinePoint, FieldBytes, PublicKey, Scalar};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

pub fn ecdsa_zkp() {
    // 1) generate ecdsa signature (r, s)
    let secret_key = SigningKey::random(&mut OsRng);
    let public_key = PublicKey::from(secret_key.verifying_key())
        .to_projective()
        .to_affine();
    let message = b"hello world";
    let mut hasher = Sha256::new();
    hasher.update(message);
    let pre_hashed = hasher.finalize();

    let signature: Signature = secret_key.sign(&pre_hashed);
    let (r, s) = signature.split_scalars();
    let z = Scalar::from_repr(pre_hashed).unwrap();
    // 2) recover point R
    let s_inv = s.invert().unwrap();
    let u1 = s_inv * z;
    let u2 = s_inv * r.as_ref();
    let R = (AffinePoint::GENERATOR * u1 + public_key * u2).to_affine();
    // NOTE just a test
    //let (x, y) = split_coordinates(&R);
    //assert_eq!(R.x(), FieldBytes::from(x));
    let r_inv = r.invert().unwrap();
    let s1 = r_inv * s.as_ref();
    let z1 = r_inv * z;
    let Q = (AffinePoint::GENERATOR * z1).to_affine();
    // 3) create pedersen commitments for signature and public key
    let pedersen = Pedersen::new();
    let s1_commitment = pedersen.commit(s1);
    let pk_pedersen = Pedersen::new();
    let (px, py) = split_coordinates(&public_key);
    let px_commitment = pk_pedersen.commit(px);
    let py_commitment = pk_pedersen.commit(py);
    // 4) generate exp proof based on the commitments

    // 5) verify zero knowledge proof
}

// TODO point addition proof
// TODO point exponentiation proof

fn split_coordinates(point: &AffinePoint) -> (Scalar, Scalar) {
    let encoded = point.to_encoded_point(false); // false - uncompressed
    match encoded.coordinates() {
        Coordinates::Uncompressed { x, y } => (
            Scalar::from_repr(*x).unwrap(),
            Scalar::from_repr(*y).unwrap(),
        ),
        _ => panic!("should be uncompressed"),
    }
}

struct Pedersen {
    pub generator: AffinePoint,
}

impl Pedersen {
    pub fn new() -> Self {
        let blinding_factor = Scalar::random(OsRng);
        Self {
            generator: (AffinePoint::GENERATOR * blinding_factor).into(),
        }
    }

    pub fn commit(&self, commitment: Scalar) -> AffinePoint {
        let random_point = self.generator * Scalar::random(OsRng);
        (AffinePoint::GENERATOR * commitment + random_point).into()
    }
}
