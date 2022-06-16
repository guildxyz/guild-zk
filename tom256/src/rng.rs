use rand::rngs::{OsRng, StdRng};
use rand_core::{CryptoRng, RngCore, SeedableRng};

pub trait RngDefault {
    fn default() -> Self;
}

pub trait CryptoCoreRng: RngCore + CryptoRng + RngDefault {}

impl RngDefault for OsRng {
    fn default() -> Self {
        OsRng
    }
}

impl RngDefault for StdRng {
    fn default() -> Self {
        StdRng::from_entropy()
    }
}

impl CryptoCoreRng for OsRng {}
impl CryptoCoreRng for StdRng {}
