use ark_crypto_primitives::crh::sha256::Sha256;
use ark_crypto_primitives::crh::CRHScheme;
use ark_ec::models::short_weierstrass::{Affine, SWCurveConfig};
use ark_ff::PrimeField;
use ark_serialize::CanonicalSerialize;

pub fn hash_points<C: SWCurveConfig>(points: &[&Affine<C>]) -> C::ScalarField {
    let mut input = Vec::new();
    for &point in points {
        point
            .serialize_compressed(&mut input)
            .expect("this operation never fails; qed");
    }

    let digest: [u8; 32] = Sha256::evaluate(&(), input)
        .expect("this operation never fails; qed")
        .try_into()
        .expect("digest is 32 bytes; qed");

    C::ScalarField::from_le_bytes_mod_order(&digest)
}

#[cfg(test)]
mod test {
    use super::{hash_points, SWCurveConfig};
    use ark_ec::models::CurveConfig;
    use ark_ff::BigInt;
    use ark_secp256k1::Config;

    #[test]
    fn point_hasing() {
        let points = &[
            &Config::GENERATOR,
            &(Config::GENERATOR * <Config as CurveConfig>::ScalarField::new(BigInt([2, 3, 4, 5])))
                .into(),
            &(Config::GENERATOR * <Config as CurveConfig>::ScalarField::new(BigInt([3, 2, 1, 0])))
                .into(),
        ];

        let expected = <Config as CurveConfig>::ScalarField::new(BigInt([
            6366269230117199052,
            14972045844700620618,
            7250289259405744416,
            17496338974118635249,
        ]));

        assert_eq!(hash_points(points), expected);
    }
}
