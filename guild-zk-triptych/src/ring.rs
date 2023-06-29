use ark_ec::CurveConfig;
use ark_ff::fields::PrimeField;
use ark_secp256k1::Affine;
use ark_secp256k1::fq::Fq;
use ark_serialize::CanonicalDeserialize;

pub struct Ring(Vec<Affine>);

impl Ring {
    pub fn parse(ring: Vec<String>) -> Result<Self, String> {
        // TODO throw if empty
        todo!()
    }

    fn pad(&mut self) {
        todo!()
    }

    pub fn log2_len(&self) -> usize {
        self.0.len().ilog2() as usize
    }
}

fn parse_pubkey(pk_string: &str) -> Result<Affine, String> {
    let decoded_pubkey = hex::decode(pk_string.trim_start_matches("0x04")).map_err(|e| e.to_string())?;
    let x = Fq::from_be_bytes_mod_order(&decoded_pubkey[0..32]);
    let y = Fq::from_be_bytes_mod_order(&decoded_pubkey[32..]);
    Ok(Affine::new(x, y))
}

#[cfg(test)]
mod test {
    use super::*;
    use ark_ec::AffineRepr;
    use ark_serialize::CanonicalSerialize;

    #[test]
    fn pubkey_parsing_works() {
        let pubkey_str = "0x0454e32170dd5a0b7b641aa77daa1f3f31b8df17e51aaba6cfcb310848d26351180b6ac0399d21460443d10072700b64b454d70bfba5e93601536c740bbd099682";

        let pubkey = parse_pubkey(pubkey_str).unwrap();

        let mut expected_bytes = Vec::new();
        pubkey.serialize_uncompressed(&mut expected_bytes).unwrap();

        assert_eq!(pubkey_str.trim_start_matches("0x"), hex::encode(expected_bytes));
    }
}
