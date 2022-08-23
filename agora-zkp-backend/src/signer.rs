use blake2::{digest::consts::U32, Blake2b, Digest};
use chrono;
use libsecp256k1::{Message, PublicKey, SecretKey, Signature};
use std::fmt::Write;
use std::num::ParseIntError;
use uuid::Uuid;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SignedResponse {
    pub pubkeys: Vec<String>,
    pub hash: String,
    pub nonce: String,
    pub timestamp: i64,
    pub signature: String,
}

pub struct Signer {
    privkey: SecretKey,
    pubkey: PublicKey,
}

impl Signer {
    pub fn new(private_key: String) -> Signer {
        let privkey = decode_hex(private_key.as_str());
        let privkey = SecretKey::parse_slice(&privkey).expect("failed to parse private key");
        let pubkey = PublicKey::from_secret_key(&privkey);
        Signer { privkey, pubkey }
    }

    pub fn sign_pubkeys(&self, pubkeys: Vec<String>) -> SignedResponse {
        let nonce = Uuid::new_v4();
        let timestamp = chrono::offset::Utc::now().timestamp();

        let hash = hash_message(&nonce.to_string(), timestamp.to_string(), &pubkeys);
        let signature = sign_message_hash(&self.privkey, &hash);

        let resp = SignedResponse {
            pubkeys,
            hash: encode_hex(&hash),
            nonce: nonce.to_string(),
            timestamp,
            signature: encode_hex(&signature),
        };
        resp
    }

    pub fn verify(&self, resp: &SignedResponse) -> bool {
        let hash = hash_message(&resp.nonce, resp.timestamp.to_string(), &resp.pubkeys);
        verify_message_hash(&self.pubkey, hash, decode_hex(&resp.signature))
    }
}

type Blake2b256 = Blake2b<U32>;

fn hash_message(nonce: &String, timestamp: String, pubkeys: &Vec<String>) -> Vec<u8> {
    let mut hasher = Blake2b256::new();
    hasher.update(nonce);
    hasher.update(timestamp.to_string());
    for pubkey in pubkeys.iter() {
        hasher.update(pubkey);
    }
    let res = hasher.finalize();
    res.to_vec()
}

fn sign_message_hash(private_key: &SecretKey, hash: &Vec<u8>) -> Vec<u8> {
    let message = Message::parse_slice(hash).expect("failed to parse hash");
    let (signature, recovery_id) = libsecp256k1::sign(&message, private_key);
    let signature = signature.serialize();
    let mut signature = signature.to_vec();
    signature.push(recovery_id.serialize());
    signature
}

fn verify_message_hash(pubkey: &PublicKey, hash: Vec<u8>, signature: Vec<u8>) -> bool {
    let message = Message::parse_slice(&hash).expect("failed to parse hash");
    if signature.len() != 65 {
        return false;
    }
    let mut sig_wo_recovery_id: [u8; 64] = [0; 64];
    for i in 0..64 {
        sig_wo_recovery_id[i] = signature[i];
    }
    let sig = match Signature::parse_standard(&sig_wo_recovery_id) {
        Ok(res) => res,
        Err(_) => return false,
    };
    libsecp256k1::verify(&message, &sig, &pubkey)
}

fn decode_hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect::<Result<Vec<u8>, ParseIntError>>()
        .expect("failed to convert hex str to bytes")
}

fn encode_hex(bytes: &Vec<u8>) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes.iter() {
        write!(&mut s, "{:02x}", b).expect("failed to convert bytes ot hex string");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_pubkeys() {
        let privkey =
            String::from("1111111111111111111111111111111111111111111111111111111111111111");

        let signer = Signer::new(privkey);
        let pubkeys = vec![String::from("1"), String::from("2"), String::from("3")];

        let mut resp = signer.sign_pubkeys(pubkeys);
        println!("{:?}", resp);
        let ok = signer.verify(&resp);
        assert!(ok);

        // change timestamp to invalidate signature
        resp.timestamp = 1;
        let ok = signer.verify(&resp);
        assert!(!ok);
    }

    #[test]
    fn test_sign_verify_message_hash() {
        let mut hasher = Blake2b256::new();
        let msg = String::from("Hello World!");
        hasher.update(msg.clone());
        let res = hasher.finalize();
        let hash = res.to_vec();

        let privkey =
            String::from("1111111111111111111111111111111111111111111111111111111111111111");
        let privkey = decode_hex(privkey.as_str());
        let privkey = SecretKey::parse_slice(&privkey).expect("failed to parse private key");
        let pubkey = PublicKey::from_secret_key(&privkey);
        let signature = sign_message_hash(&privkey, &hash);
        let want = String::from("9684ca7f7b9c91250ffdd8a28d00c295606193747c88333032ce9b928bb2bc5a4b36935c6c5ab01fcf9f7db9ca8938c0bc71d5dd88e556f4165af29d5fbb8d3700");
        assert_eq!(encode_hex(&signature), want);

        let ok = verify_message_hash(&pubkey, hash, signature);
        assert!(ok);
    }
}
