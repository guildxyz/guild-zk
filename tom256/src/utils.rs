use rand_core::{RngCore, CryptoRng};

use bigint::prelude::Encoding;
use bigint::U256;

use subtle::ConstantTimeLess;

use crate::arithmetic::modular::Modular;

fn get_random_u256<R: CryptoRng + RngCore>(rng: &mut R) -> U256 {
    let mut bytes = [0_u8; 32];
    rng.fill_bytes(&mut bytes);
    U256::from_be_bytes(bytes)
}

pub fn random_mod_u256<T: Modular, R: CryptoRng + RngCore>(rng: &mut R) -> T {
    loop {
        let random_number = get_random_u256(rng);
        if random_number.ct_lt(&T::MODULUS).into() {
            return T::new(random_number);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bigint::NonZero;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct TestModular(U256);

    // assumed to be < 255
    const MOD: u32 = 17;

    impl Modular for TestModular {
        const MODULUS: U256 = U256::from_u32(MOD);
        fn new(number: U256) -> Self {
            let reduced = if number < Self::MODULUS {
                number
            } else {
                // NOTE unwrap is fine here because the modulus
                // can be safely assumed to be nonzero
                number % NonZero::new(Self::MODULUS).unwrap()
            };
            Self(reduced)
        }

        fn inner(&self) -> &U256 {
            &self.0
        }
    }

    // assumed: mod_byte_number <= 4
    // Only for tests
    #[allow(unused)]
    fn get_random_small_modular<T: Modular, R: CryptoRng + RngCore>(mod_byte_number: u8, rng: &mut R) -> T {
        loop {
            let random_number_bytes = get_random_u256(rng).to_be_bytes();

            for small_bytes in random_number_bytes.chunks_exact(mod_byte_number as usize) {
                let mut random_number = 0_u32;
                for i in 0..mod_byte_number as usize {
                    random_number = (random_number << 8) + (small_bytes[i] as u32);
                }
                let random_number = U256::from_u32(random_number);

                if random_number.ct_lt(&T::MODULUS).into() {
                    return T::new(random_number);
                }
            }
        }
    }

    /*
    // Don't do this if you value your time
    // It works, just trust me
    #[test]
    fn test_rand() {
        let mut vec = vec![0; MOD as usize];
        let mut rng = ChaChaRng::from_entropy();
        for i in 0..1000000 {
            let rand_num = get_random_small_modular::<TestModular, ChaChaRng>(1, &mut rng);
            vec[rand_num.inner().into_limbs()[0].0 as usize] += 1;
        }
        println!("{:?}", vec);
    }
    */
}
