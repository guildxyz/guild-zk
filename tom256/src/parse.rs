use crate::arithmetic::{FieldElement, Modular, Point, Scalar};
use crate::curve::Curve;
use crate::U256;

pub fn address_to_scalar<C: Curve>(address: &str) -> Result<Scalar<C>, String> {
    let stripped = address.trim_start_matches("0x");
    let mut padded = "000000000000000000000000".to_string(); // 24 zeros to pad 20 bit address to 32 bit scalar
    padded.push_str(stripped);
    // NOTE this check avoids explicit panics by `from_be_hex`
    if padded.len() != 64 {
        return Err("invalid address".to_string());
    }
    Ok(Scalar::new(U256::from_be_hex(&padded)))
}

enum Parse {
    Pubkey,
    Signature,
}

pub fn parse_pubkey<C: Curve>(pubkey: &str) -> Result<Point<C>, String> {
    let (x, y) = parse_str(pubkey, Parse::Pubkey)?;
    Ok(Point::new(
        FieldElement::<C>::new(x),
        FieldElement::<C>::new(y),
        FieldElement::<C>::ONE,
    ))
}

pub fn parse_signature<C: Curve>(signature: &str) -> Result<(Scalar<C>, Scalar<C>), String> {
    let (r, s) = parse_str(signature, Parse::Signature)?;
    Ok((Scalar::new(r), Scalar::new(s)))
}

fn parse_str(slice: &str, into: Parse) -> Result<(U256, U256), String> {
    let trimmed = slice.trim_start_matches("0x");
    if trimmed.len() != 130 {
        return Err("invalid bytes".to_string());
    }
    match into {
        Parse::Pubkey => {
            // NOTE pubkeys always start with 0x04
            let x = U256::from_be_hex(&trimmed[2..66]);
            let y = U256::from_be_hex(&trimmed[66..]);
            Ok((x, y))
        }
        Parse::Signature => {
            let r = U256::from_be_hex(&trimmed[0..64]);
            let s = U256::from_be_hex(&trimmed[64..128]);
            // NOTE last 2 bytes represent the recovery `v` parameter
            Ok((r, s))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::arithmetic::Modular;
    use crate::curve::{Secp256k1, Tom256k1};
    use crate::U256;

    #[test]
    fn address_conversion() {
        let address = "0x0123456789012345678901234567890123456789";
        let scalar = address_to_scalar::<Tom256k1>(address).unwrap();
        assert_eq!(
            scalar,
            Scalar::new(U256::from_be_hex(
                "0000000000000000000000000123456789012345678901234567890123456789"
            ))
        );

        let address = "0000000000000000000000000000000000000000";
        let scalar = address_to_scalar::<Tom256k1>(address).unwrap();
        assert_eq!(scalar, Scalar::<Tom256k1>::ZERO);

        let address = "0x12345";
        assert!(address_to_scalar::<Tom256k1>(address).is_err());

        let address = "3".repeat(42);
        assert!(address_to_scalar::<Tom256k1>(&address).is_err());
    }

    #[test]
    fn parsing() {
        let signature = "0x45c4039b611c0cc207ff7fb7a6899ea0431aac2cf37515d74a71f2df00e2c3e0096fad5e7eda762898fffd4644f8a7a406bf6bde868814ea03058c882fcd23311c";

        let (r, s) = parse_signature(signature).unwrap();
        assert_eq!(
            r,
            Scalar::<Secp256k1>::new(U256::from_be_hex(
                "45c4039b611c0cc207ff7fb7a6899ea0431aac2cf37515d74a71f2df00e2c3e0"
            ))
        );
        assert_eq!(
            s,
            Scalar::<Secp256k1>::new(U256::from_be_hex(
                "096fad5e7eda762898fffd4644f8a7a406bf6bde868814ea03058c882fcd2331"
            ))
        );

        let pubkey = "0x0408c6cd9400645819c8c556a6e83e0a7728f070a813bb9d24d5c24290e21fc5e438396f9333264d3e7c1d3e6ee1bc572b2f00b98db7065e9bf278f2b8dbe02718";

        let pubkey_point = parse_pubkey(pubkey).unwrap();
        assert_eq!(
            pubkey_point.x(),
            &FieldElement::<Secp256k1>::new(U256::from_be_hex(
                "08c6cd9400645819c8c556a6e83e0a7728f070a813bb9d24d5c24290e21fc5e4"
            ))
        );
        assert_eq!(
            pubkey_point.y(),
            &FieldElement::<Secp256k1>::new(U256::from_be_hex(
                "38396f9333264d3e7c1d3e6ee1bc572b2f00b98db7065e9bf278f2b8dbe02718"
            ))
        );

        assert_eq!(pubkey_point.z(), &FieldElement::ONE);
    }
}
