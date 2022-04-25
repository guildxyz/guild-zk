use crate::arithmetic::field::FieldElement;
use crate::arithmetic::modular::{mul_mod_u256, Modular};
use crate::arithmetic::scalar::Scalar;
use crate::Curve;

use bigint::prelude::Encoding;
use bigint::U256;

use std::collections::HashMap;
use std::marker::PhantomData;

use sha3::{Digest, Sha3_256};

const BASE_16_DIGITS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
];

#[derive(Debug, Clone)]
pub struct Point<C: Curve> {
    x: FieldElement<C>,
    y: FieldElement<C>,
    z: FieldElement<C>,
}

impl<C: Curve> Point<C> {
    pub fn hash(&self) -> U256 {
        // create a SHA3-256 object
        let mut hasher = Sha3_256::new();

        // write input message
        hasher.update(self.x.inner().to_be_bytes());
        hasher.update(self.y.inner().to_be_bytes());
        hasher.update(self.z.inner().to_be_bytes());

        // read hash digest
        let result = hasher.finalize();
        U256::from_be_bytes(result[0..32].try_into().unwrap())
    }
}

impl<C: Curve + PartialEq> PartialEq for Point<C> {
    fn eq(&self, other: &Self) -> bool {
        let x0z1 = self.x * other.z;
        let x1z0 = other.x * self.z;
        let y0z1 = self.y * other.z;
        let y1z0 = other.y * self.z;

        x0z1 == x1z0 && y0z1 == y1z0
    }
}

impl<C: Curve> std::ops::Neg for Point<C> {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            x: self.x,
            y: -self.y,
            z: self.z,
        }
    }
}

impl<C: Curve> std::ops::Add for Point<C> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.geometric_add(&rhs)
    }
}

impl<'a, 'b, C: Curve> std::ops::Add<&'b Point<C>> for &'b Point<C> {
    type Output = Point<C>;
    fn add(self, rhs: &Point<C>) -> Self::Output {
        self.geometric_add(rhs)
    }
}

impl<C: Curve> std::ops::AddAssign<&Point<C>> for Point<C> {
    fn add_assign(&mut self, rhs: &Self) {
        *self = &*self + rhs
    }
}

impl<C: Curve> std::ops::Sub for Point<C> {
    type Output = Point<C>;
    fn sub(self, rhs: Self) -> Self {
        self + (-rhs)
    }
}

impl<C: Curve> Point<C> {
    pub const GENERATOR: Self = Self {
        x: FieldElement(C::GENERATOR_X, PhantomData),
        y: FieldElement(C::GENERATOR_Y, PhantomData),
        z: FieldElement::ONE,
    };

    pub const IDENTITY: Self = Self {
        x: FieldElement::ZERO,
        y: FieldElement::ONE,
        z: FieldElement::ZERO,
    };

    pub fn is_on_curve(&self) -> bool {
        let a = FieldElement::new(C::COEFF_A);
        let b = FieldElement::new(C::COEFF_B);

        let y2 = self.y * self.y;
        let y2z = y2 * self.z;
        let x3 = self.x * self.x * self.x;
        let ax = a * self.x;
        let z2 = self.z * self.z;
        let axz2 = ax * z2;
        let z3 = z2 * self.z;
        let bz3 = b * z3;
        let t5 = y2z - (x3 + axz2 + bz3);

        t5.inner() == &U256::ZERO
    }

    pub fn double(&self) -> Self {
        self + self
    }

    pub fn geometric_add(&self, rhs: &Self) -> Self {
        let b3 = FieldElement::new(mul_mod_u256(
            &U256::from_u8(3),
            &C::COEFF_B,
            &C::PRIME_MODULUS,
        ));
        let a = FieldElement::new(C::COEFF_A);

        let mut t0 = self.x * rhs.x;
        let mut t1 = self.y * rhs.y;
        let mut t2 = self.z * rhs.z;
        let mut t3 = self.x + self.y;
        let mut t4 = rhs.x + rhs.y;

        t3 *= t4;
        t4 = t0 + t1;
        t3 -= t4;
        t4 = self.x + self.z;
        let mut t5 = rhs.x + rhs.z;

        t4 *= t5;
        t5 = t0 + t2;
        t4 -= t5;
        t5 = self.y + self.z;
        let mut sum_x = rhs.y + rhs.z;

        t5 *= sum_x;
        sum_x = t1 + t2;
        t5 -= sum_x;
        let mut sum_z = a * t4;
        sum_x = b3 * t2;

        sum_z += sum_x;
        sum_x = t1 - sum_z;
        sum_z += t1;
        let mut sum_y = sum_x * sum_z;
        t1 = t0 + t0;

        t1 += t0;
        t2 = a * t2;
        t4 *= b3;
        t1 += t2;
        t2 = t0 - t2;

        t2 *= a;
        t4 += t2;
        t0 = t1 * t4;
        sum_y += t0;
        t0 = t4 * t5;

        sum_x *= t3;
        sum_x -= t0;
        t0 = t1 * t3;
        sum_z *= t5;
        sum_z += t0;

        Self {
            x: sum_x,
            y: sum_y,
            z: sum_z,
        }
    }

    pub fn scalar_mul(&self, scalar: &Scalar<C>) -> Self {
        let mut q = Self::IDENTITY;
        let mut current = Self::IDENTITY;
        let mut lookup = HashMap::with_capacity(16);
        for digit in &BASE_16_DIGITS {
            lookup.insert(digit, current.clone());
            current += self;
        }
        for ch in scalar.to_unpadded_string().chars() {
            q = q.double();
            q = q.double();
            q = q.double();
            q = q.double();
            // NOTE: unwrap is fine because ch is definitely
            // one of the keys in the map
            q += lookup.get(&ch).unwrap()
        }
        q
    }

    pub fn double_mul(
        &self,
        this_scalar: &Scalar<C>,
        other_point: &Self,
        other_scalar: &Scalar<C>,
    ) -> Self {
        let mut q = Self::IDENTITY;
        let mut this_current = Self::IDENTITY;
        let mut other_current = Self::IDENTITY;
        let mut this_lookup = HashMap::with_capacity(16);
        let mut other_lookup = HashMap::with_capacity(16);
        for digit in &BASE_16_DIGITS {
            this_lookup.insert(digit, this_current.clone());
            other_lookup.insert(digit, other_current.clone());
            this_current += self;
            other_current += other_point;
        }

        let (this_string, other_string) = this_scalar.pad_to_equal_len_strings(other_scalar);
        for (this_ch, other_ch) in this_string.chars().zip(other_string.chars()) {
            q = q.double();
            q = q.double();
            q = q.double();
            q = q.double();
            // NOTE: both unwraps are fine because chars are definitely
            // one of the keys in the respective maps
            q += this_lookup.get(&this_ch).unwrap();
            q += other_lookup.get(&other_ch).unwrap();
        }
        q
    }

    pub fn is_identity(&self) -> bool {
        self.x == FieldElement::ZERO && self.y != FieldElement::ZERO && self.z == FieldElement::ZERO
    }

    pub fn into_affine(self) -> Self {
        if self.is_identity() {
            Self::IDENTITY
        } else {
            let z_inv = self.z.inverse();
            Self {
                x: self.x * z_inv,
                y: self.y * z_inv,
                z: FieldElement::ONE,
            }
        }
    }

    pub fn x(&self) -> &FieldElement<C> {
        &self.x
    }

    pub fn y(&self) -> &FieldElement<C> {
        &self.x
    }

    pub fn z(&self) -> &FieldElement<C> {
        &self.z
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Secp256k1, Tom256k1};

    type SecPoint = Point<Secp256k1>;
    type TomPoint = Point<Tom256k1>;

    type SecScalar = Scalar<Secp256k1>;
    type TomScalar = Scalar<Tom256k1>;

    #[test]
    fn on_curve_check() {
        assert!(SecPoint::GENERATOR.is_on_curve());
        assert!(TomPoint::GENERATOR.is_on_curve());
        assert!(SecPoint::GENERATOR.double().is_on_curve());
        assert!(TomPoint::GENERATOR.double().is_on_curve());
        let sec_scalar = SecScalar::new(U256::from_u32(123456));
        let sec_point = SecPoint::GENERATOR.scalar_mul(&sec_scalar);
        assert!(sec_point.is_on_curve());

        let tom_scalar = TomScalar::new(U256::from_u32(678910));
        let tom_point = TomPoint::GENERATOR.scalar_mul(&tom_scalar);
        assert!(tom_point.is_on_curve());
    }

    #[test]
    fn point_addition_test() {
        let four = SecScalar::new(U256::from_u8(4));
        let g2 = SecPoint::GENERATOR.double();
        let g4 = SecPoint::GENERATOR.scalar_mul(&four);
        assert_eq!(g2.double(), g4);
        assert_eq!(&g2 + &g2, g4);
    }

    #[test]
    fn affine_point() {
        let g2 = SecPoint::GENERATOR.double();
        let g2_affine = g2.into_affine();
        assert_eq!(g2_affine.z(), &FieldElement::ONE);

        let mut id = SecPoint::IDENTITY;
        id = id.into_affine();
        assert_eq!(id, SecPoint::IDENTITY);

        let g5 = SecPoint::GENERATOR
            .scalar_mul(&SecScalar::new(U256::from_u8(5)))
            .into_affine();
        let g2 = SecPoint::GENERATOR.double().into_affine();
        let g4 = g2.double().into_affine();
        assert_eq!((g4 + SecPoint::GENERATOR).into_affine(), g5);
    }

    #[test]
    fn point_hash_test() {
        let expected_hash = "97FAA02BE4E7F5F9306261D1616841C83603E8699E86A0161ED8F8DDCEEAE0A8";
        assert_eq!(
            Point::<Secp256k1>::GENERATOR.hash(),
            U256::from_be_hex(expected_hash)
        );

        let expected_hash = "FDC209252A1B98A0E4A6958FC0305A5A947D5FB6E066A171FBF22B612CB9C3D1";
        assert_eq!(
            Point::<Tom256k1>::GENERATOR.hash(),
            U256::from_be_hex(expected_hash)
        );
    }
}
