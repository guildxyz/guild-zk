use ark_crypto_primitives::crh::sha256::Sha256;
use ark_crypto_primitives::crh::CRHScheme;
use ark_ec::models::short_weierstrass::{Affine, SWCurveConfig};
use ark_ec::models::CurveConfig;
use ark_ff::PrimeField;
use ark_serialize::CanonicalSerialize;

pub fn hash_points<C: CurveConfig + SWCurveConfig>(
    points: &[&Affine<C>],
) -> Result<C::ScalarField, String> {
    let mut input = Vec::new();
    for &point in points {
        point
            .serialize_compressed(&mut input)
            .map_err(|e| e.to_string())?;
    }

    let digest: [u8; 32] = Sha256::evaluate(&(), input)
        .expect("this operation never fails; qed")
        .try_into()
        .expect("digest is 32 bytes; qed");

    Ok(C::ScalarField::from_le_bytes_mod_order(&digest))
}

#[cfg(test)]
mod test {
    use super::{hash_points, CurveConfig, SWCurveConfig};
    use ark_ff::BigInt;
    use ark_secp256k1::Config;

    #[test]
    fn point_hasing() {
        let points = &[
            &Config::GENERATOR,
            &Config::mul_affine(&Config::GENERATOR, &[2, 3, 4, 5]).into(),
            &Config::mul_affine(&Config::GENERATOR, &[3, 2, 1, 0]).into(),
        ];

        let expected = <Config as CurveConfig>::ScalarField::new(BigInt([
            6366269230117199052,
            14972045844700620618,
            7250289259405744416,
            17496338974118635249,
        ]));

        assert_eq!(hash_points(points), Ok(expected));
    }
}
