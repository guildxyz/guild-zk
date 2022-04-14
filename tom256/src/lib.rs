mod field;
mod point;

use elliptic_curve::bigint::U256;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct TomEdwards256;

impl elliptic_curve::Curve for TomEdwards256 {
    type UInt = U256;
    const ORDER: U256 =
        U256::from_be_hex("ffffffff00000001000000000000000000000000ffffffffffffffffffffffff");
}

impl elliptic_curve::PrimeCurve for TomEdwards256 {}

pub type FieldBytes = elliptic_curve::FieldBytes<TomEdwards256>;
pub type NonZeroScalar = elliptic_curve::NonZeroScalar<TomEdwards256>;



#[cfg(test)]
mod test {
    use super::*;
    use elliptic_curve::Curve;

    #[test]
    fn negated() {
        let modulus = U256::from(7u8);
        let neg = U256::ZERO.sub_mod(&TomEdwards256::ORDER, &modulus);
        let original = neg.add_mod(&TomEdwards256::ORDER, &modulus);
        assert_eq!(original, U256::ZERO);
    }
}
