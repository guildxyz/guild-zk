use crate::arithmetic::{Modular, Point};
use crate::Curve;

use bigint::{Encoding, U256};
use sha3::{Digest, Sha3_256};

pub fn hash_points<C: Curve>(hash_id: &[u8], points: &[&Point<C>]) -> U256 {
    // create a SHA3-256 object
    let mut hasher = Sha3_256::new();

    hasher.update(hash_id);
    for p in points {
        // write input message
        hasher.update(p.x().inner().to_be_bytes());
        hasher.update(p.y().inner().to_be_bytes());
        hasher.update(p.z().inner().to_be_bytes());
    }

    // read hash digest
    let result = hasher.finalize();
    U256::from_be_bytes(result[0..32].try_into().unwrap())
}

#[test]
fn points_hash_test() {
    let hash_id = "test".as_bytes();
    let g = Point::<crate::Secp256k1>::GENERATOR;
    let g2 = Point::<crate::Secp256k1>::GENERATOR.double();
    let points = vec![&g, &g2];
    let expected_hash = "C9B5BD2009A84423D2CBCEB411CDDAF7423B372B5F63821DACFFFA0041A6B8F7";
    assert_eq!(
        hash_points(hash_id, &points),
        U256::from_be_hex(expected_hash)
    );
}
