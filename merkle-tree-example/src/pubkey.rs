
use ark_ff::bytes::ToBytes;
use ark_std::io::{Result as IoResult, Write};

#[derive(Clone)]
pub struct Pubkey {
    pub secret_key: [u8; 32],
}

impl Pubkey {
    pub fn as_vec(&self) -> Vec<u8> {
        self.secret_key.to_vec()
    }
}

impl ToBytes for Pubkey {
    #[inline]
    fn write<W: Write>(&self, writer: W) -> IoResult<()> {
        self.secret_key.write(writer)
    }
}