use bigint::{NonZero, Split, U256, U512};

pub trait Modular: Sized {
    const MODULUS: U256;

    fn new(number: U256) -> Self;

    fn inner(&self) -> &U256;

    fn add(&self, other: &Self) -> Self {
        Self::new(self.inner().add_mod(other.inner(), &Self::MODULUS))
    }

    fn neg(&self) -> Self {
        Self::new(self.inner().neg_mod(&Self::MODULUS))
    }

    fn sub(&self, other: &Self) -> Self {
        Self::new(self.inner().sub_mod(other.inner(), &Self::MODULUS))
    }

    fn mul(&self, other: &Self) -> Self {
        Self::new(mul_mod_u256(self.inner(), other.inner(), &Self::MODULUS))
    }
}

pub fn mul_mod_u256(lhs: &U256, rhs: &U256, modulus: &U256) -> U256 {
    // NOTE modulus is never zero, so unwrap is fine here
    let mod512 = NonZero::new(U512::from((U256::ZERO, *modulus))).unwrap();
    // NOTE facepalm:
    // U512::from((hi, lo))
    // but split returns (lo, hi)
    let (lo, hi) = lhs.mul_wide(rhs);
    let product = U512::from((hi, lo));
    // split the remainder result of a % b into a (lo, hi) U256 pair
    // 'hi' should always be zero because the modulus is an U256 number
    let (hi, lo) = (product % mod512).split();
    debug_assert_eq!(hi, U256::ZERO);
    lo
}

#[cfg(test)]
mod test {
    use super::{mul_mod_u256, U256};

    #[test]
    fn mul_mod_u256_small() {
        let modulus = U256::from_u32(0xfff1);

        let mut a = U256::from_u32(0x1f85);
        let mut b = U256::from_u32(0xdecc);
        assert_eq!(mul_mod_u256(&a, &b, &modulus), U256::from_u32(0xf8c));

        a = U256::from_u32(0x2f7a);
        b = U256::from_u32(0xb7d0);
        assert_eq!(mul_mod_u256(&a, &b, &modulus), U256::from_u32(0xc888));

        a = U256::from_u32(0xce4e);
        b = U256::from_u32(0xd6c5);
        assert_eq!(mul_mod_u256(&a, &b, &modulus), U256::from_u32(0x1ac8));

        a = U256::from_u32(0xe9b7);
        b = U256::from_u32(0x7ae9);
        assert_eq!(mul_mod_u256(&a, &b, &modulus), U256::from_u32(0x8113));

        a = U256::from_u32(0x81cf);
        b = U256::from_u32(0x5066);
        assert_eq!(mul_mod_u256(&a, &b, &modulus), U256::from_u32(0xcc14));
    }

    #[test]
    fn mul_mod_u256_medium() {
        let modulus = U256::from_u128(0xffffddddeeee01234577);
        let mut a = U256::from_u128(0x7ed884b0e74acae13d24);
        let mut b = U256::from_u128(0xa6e784d01d4a07b81612);
        assert_eq!(
            mul_mod_u256(&a, &b, &modulus),
            U256::from_u128(0x72394745f2370b3c9ff)
        );

        a = U256::from_u128(0xce9ffee8f7253aa55e9c);
        b = U256::from_u128(0xc8facfc1741dff31d5ac);
        assert_eq!(
            mul_mod_u256(&a, &b, &modulus),
            U256::from_u128(0xd136bceb80f3c7526553)
        );

        a = U256::from_u128(0xf14e3b80a32e6b760f06);
        b = U256::from_u128(0xc7cae1a1b0a233420225);
        assert_eq!(
            mul_mod_u256(&a, &b, &modulus),
            U256::from_u128(0x133f243f42d8b0f64108)
        );

        a = U256::from_u128(0x9e389b90cafabbcf2e83);
        b = U256::from_u128(0x491520dbfb2dc5346c8b);
        assert_eq!(
            mul_mod_u256(&a, &b, &modulus),
            U256::from_u128(0x83a9533a74aa04a01ca)
        );

        a = U256::from_u128(0xd9f35dd4b28e47de102f);
        b = U256::from_u128(0xdfe631cf3be44427e8c3);
        assert_eq!(
            mul_mod_u256(&a, &b, &modulus),
            U256::from_u128(0x750f5a73304a09deddcf)
        );
    }

    #[test]
    fn mul_mod_u256_large() {
        let modulus =
            U256::from_be_hex("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141");

        let mut a =
            U256::from_be_hex("617652b9bba98825bfe56f8632d46088bcbaf1dbac087c297682f9d2156e5139");
        println!("{}", a.to_string());
        let mut b =
            U256::from_be_hex("e7d95f100dfa1650113d52cde817ae2bbde56dffbe69d1b6afc5d6884934fc4c");
        println!("{}", b.to_string());
        assert_eq!(
            mul_mod_u256(&a, &b, &modulus),
            U256::from_be_hex("d0febbc44b1942b614c343706e565dd7679efce7d1a630d386f7effbf5d5cf90")
        );

        a = U256::from_be_hex("5b864cf99e352a9e227727f1f7b06ae96156a8a9a8bf6870156835b19bb80da8");
        b = U256::from_be_hex("aa83f02c146c7039a039fed8fa82e2a2f48f1b40d7e88f4552d6c51222c948fc");
        assert_eq!(
            mul_mod_u256(&a, &b, &modulus),
            U256::from_be_hex("2039e582a2799f7aafd4db598bc6eca5031e555a9c957401859137f3f95cbf52")
        );

        a = U256::from_be_hex("eff64d997f9210ac3eac68ed417b556c9d678ba590d8541d05be518c2963bc5c");
        b = U256::from_be_hex("373e807ff7fdfeda3a5abbcb0a4fe08c123b5cf04eea7b4dce47858fd4502c1b");
        assert_eq!(
            mul_mod_u256(&a, &b, &modulus),
            U256::from_be_hex("5f3bb9c167e7c6a78402a58a102011d013f23d092b864ff48583053640d669f3")
        );

        a = U256::from_be_hex("a553049ab18ac4bc5b163c7c787f60cd7260e0ac505c7320872efaa6ffd3168e");
        b = U256::from_be_hex("9de6dfce877b25903632efa41f0e42ebc9c7a0aaa944f3efb8d60570e0b71b68");
        assert_eq!(
            mul_mod_u256(&a, &b, &modulus),
            U256::from_be_hex("97bb5058bb25b849015eca05d389e38dc7f903519c37a2cd7e09ca32f07a6493")
        );

        a = U256::from_be_hex("9ab29bc62d7943d0b00d521fe643f571649e9761ffe0f7e6478829fcf9f80d5b");
        b = U256::from_be_hex("565a66c3586e6c0f23753cd5e84b0cd720630d7102172a42e706e4f724baf7dd");
        assert_eq!(
            mul_mod_u256(&a, &b, &modulus),
            U256::from_be_hex("72747f9ecbf73c44f9eec8071c9c53f1648db1bbca95fb19ed4f1cc62ccd4956")
        );
    }
}
