#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(unused_crate_dependencies)]

pub mod eval;
pub mod hash;
pub mod pedersen;
pub mod proof;

use ark_ec::short_weierstrass::{Projective, SWCurveConfig};
use ark_ec::{CurveConfig, CurveGroup};
use ark_ff::{BigInteger, Field, PrimeField};
use ark_secp256k1::Config as SecpConfig;
use ark_secq256k1::Config as SecqConfig;

pub trait CycleCurveConfig: CurveConfig
where
    Projective<Self::BaseCurveConfig>: CurveGroup<
        BaseField = <Self as CurveConfig>::ScalarField,
        ScalarField = <Self as CurveConfig>::BaseField,
    >,
{
    type BaseCurveConfig: SWCurveConfig;
    fn to_cycle(base: &<Self::BaseCurveConfig as CurveConfig>::BaseField) -> Self::ScalarField {
        <Self::ScalarField as PrimeField>::from_le_bytes_mod_order(
            &base
                .to_base_prime_field_elements()
                .next()
                .expect("iterator must contain a single entry")
                .into_bigint()
                .to_bytes_le(),
        )
    }
}

impl CycleCurveConfig for SecqConfig {
    type BaseCurveConfig = SecpConfig;
}

#[cfg(test)]
mod test {
    use super::*;
    use ark_ec::short_weierstrass::Affine;
    use ark_ec::AffineRepr;

    #[test]
    fn isomorphism() {
        let g2: Affine<SecpConfig> = (SecpConfig::GENERATOR + SecpConfig::GENERATOR).into();

        let (x, y) = g2.xy().unwrap();

        let cycle_x = SecqConfig::to_cycle(x);
        let cycle_y = SecqConfig::to_cycle(y);

        assert_eq!(x.into_bigint(), cycle_x.into_bigint());
        assert_eq!(y.into_bigint(), cycle_y.into_bigint());
    }
}
