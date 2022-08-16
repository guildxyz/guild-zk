use k256::elliptic_curve::Field;
use k256::{AffinePoint, Scalar};
use rand_core::OsRng;

pub struct Keypair {
    pub public: AffinePoint,
    pub private: Scalar,
}

impl Keypair {
    pub fn random() -> Self {
        let private = Scalar::random(OsRng);
        Self {
            public: (AffinePoint::GENERATOR * private).to_affine(),
            private,
        }
    }
}
