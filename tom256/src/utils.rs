use crate::arithmetic::{Modular, Point};
use crate::Curve;

use bigint::{Encoding, U256};
use sha3::{Digest, Sha3_256};

pub struct PointHasher {
    hasher: Sha3_256,
}

impl PointHasher {
    pub fn new(hash_id: &[u8]) -> Self {
        let mut hasher = Sha3_256::new();
        hasher.update(hash_id);

        Self { hasher }
    }

    pub fn insert_point<C: Curve>(&mut self, pt: &Point<C>) {
        self.hasher.update(pt.x().inner().to_be_bytes());
        self.hasher.update(pt.y().inner().to_be_bytes());
        self.hasher.update(pt.z().inner().to_be_bytes());
    }

    pub fn insert_points<C: Curve>(&mut self, points: &[&Point<C>]) {
        for p in points {
            // write input message
            self.hasher.update(p.x().inner().to_be_bytes());
            self.hasher.update(p.y().inner().to_be_bytes());
            self.hasher.update(p.z().inner().to_be_bytes());
        }
    }

    pub fn finalize(self) -> U256 {
        let finalized = self.hasher.finalize();
        U256::from_be_bytes(finalized[0..32].try_into().unwrap())
    }
}

#[test]
fn points_hash_test() {
    let hash_id = "test".as_bytes();
    let g = Point::<crate::Secp256k1>::GENERATOR;
    let g2 = Point::<crate::Secp256k1>::GENERATOR.double();
    let points = vec![&g, &g2];
    let expected_hash = "C9B5BD2009A84423D2CBCEB411CDDAF7423B372B5F63821DACFFFA0041A6B8F7";
    let mut hasher = PointHasher::new(hash_id);
    hasher.insert_points(&points);
    assert_eq!(hasher.finalize(), U256::from_be_hex(expected_hash));
}
