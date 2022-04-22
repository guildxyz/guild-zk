use crate::arithmetic::field::FieldElement;
use crate::arithmetic::modular::{mul_mod_u256, Modular};
use crate::Curve;
use bigint::U256;

use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct Point<C: Curve> {
    x: FieldElement<C>,
    y: FieldElement<C>,
    z: FieldElement<C>,
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
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Curve, Secp256k1, Tom256k1};

    #[test]
    fn on_curve_check() {
        assert!(Point::<Secp256k1>::GENERATOR.is_on_curve());
        assert!(Point::<Tom256k1>::GENERATOR.is_on_curve());
        assert!(Point::<Secp256k1>::GENERATOR.double().is_on_curve());
        assert!(Point::<Tom256k1>::GENERATOR.double().is_on_curve());
    }

    #[test]
    fn point_addition_test() {
        // TODO
    }
}
