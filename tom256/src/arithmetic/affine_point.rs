use super::field::FieldElement;
use super::modular::{mul_mod_u256, Modular};
use super::point::Point;
use super::scalar::Scalar;
use crate::{Curve, U256};

use std::collections::HashMap;
use std::fmt;

const BASE_16_DIGITS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
];

// z can only be 1 (general point) or 0 (identity)
// This invariable is preserved in the methods
#[derive(Debug, Clone)]
pub struct AffinePoint<C: Curve> {
    x: FieldElement<C>,
    y: FieldElement<C>,
    z: FieldElement<C>,
}

impl<C: Curve> fmt::Display for AffinePoint<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f)?;
        writeln!(f, "x: {}", self.x.inner())?;
        writeln!(f, "y: {}", self.y.inner())?;
        writeln!(f, "z: {}", self.z.inner())
    }
}

impl<C: Curve + PartialEq> PartialEq for AffinePoint<C> {
    fn eq(&self, other: &Self) -> bool {
        (self.is_identity() && other.is_identity()) || (self.x == other.x && self.y == other.y)
    }
}

impl<C: Curve> From<Point<C>> for AffinePoint<C> {
    fn from(point: Point<C>) -> Self {
        point.into_affine()
    }
}

impl<C: Curve> From<AffinePoint<C>> for Point<C> {
    fn from(point: AffinePoint<C>) -> Self {
        point.into_point()
    }
}

impl<C: Curve> From<&AffinePoint<C>> for Point<C> {
    fn from(point: &AffinePoint<C>) -> Self {
        point.to_point()
    }
}

impl<C: Curve> std::ops::Neg for AffinePoint<C> {
    type Output = Self;
    fn neg(self) -> Self {
        AffinePoint {
            x: self.x,
            y: -self.y,
            z: self.z,
        }
    }
}

impl<C: Curve> std::ops::Neg for &AffinePoint<C> {
    type Output = AffinePoint<C>;
    fn neg(self) -> Self::Output {
        AffinePoint {
            x: self.x,
            y: -self.y,
            z: self.z,
        }
    }
}

impl<C: Curve> std::ops::Add for AffinePoint<C> {
    type Output = Point<C>;
    fn add(self, rhs: Self) -> Self::Output {
        self.geometric_add(&rhs)
    }
}

impl<'a, 'b, C: Curve> std::ops::Add<&'b AffinePoint<C>> for &'a AffinePoint<C> {
    type Output = Point<C>;
    fn add(self, rhs: &'b AffinePoint<C>) -> Self::Output {
        self.geometric_add(rhs)
    }
}

impl<C: Curve> std::ops::Add<AffinePoint<C>> for Point<C> {
    type Output = Point<C>;
    fn add(self, rhs: AffinePoint<C>) -> Self::Output {
        self.geometric_add(&rhs.to_point())
    }
}

impl<C: Curve> std::ops::Add<Point<C>> for AffinePoint<C> {
    type Output = Point<C>;
    fn add(self, rhs: Point<C>) -> Self::Output {
        self.geometric_add(&rhs.to_affine())
    }
}

impl<C: Curve> std::ops::AddAssign<&AffinePoint<C>> for Point<C> {
    fn add_assign(&mut self, rhs: &AffinePoint<C>) {
        *self = &*self + &rhs.to_point()
    }
}

impl<C: Curve> std::ops::Sub for AffinePoint<C> {
    type Output = Point<C>;
    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

impl<'a, 'b, C: Curve> std::ops::Sub<&'b AffinePoint<C>> for &'a AffinePoint<C> {
    type Output = Point<C>;
    fn sub(self, rhs: &'b AffinePoint<C>) -> Self::Output {
        self + &(-rhs)
    }
}

impl<C: Curve> std::ops::Mul<Scalar<C>> for &AffinePoint<C> {
    type Output = Point<C>;
    fn mul(self, rhs: Scalar<C>) -> Self::Output {
        self.scalar_mul(&rhs)
    }
}

impl<'a, 'b, C: Curve> std::ops::Mul<&'b Scalar<C>> for &'a AffinePoint<C> {
    type Output = Point<C>;
    fn mul(self, rhs: &'b Scalar<C>) -> Self::Output {
        self.scalar_mul(rhs)
    }
}

impl<C: Curve> AffinePoint<C> {
    pub fn new(x: FieldElement<C>, y: FieldElement<C>) -> Self {
        Self {
            x,
            y,
            z: FieldElement::<C>::new(U256::from_u32(1)),
        }
    }

    pub fn new_identity() -> Self {
        Self {
            x: FieldElement::<C>::new(U256::from_u32(0)),
            y: FieldElement::<C>::new(U256::from_u32(1)),
            z: FieldElement::<C>::new(U256::from_u32(0)),
        }
    }

    pub fn into_point(self) -> Point<C> {
        Point::<C>::new(self.x, self.y, self.z)
    }

    pub fn to_point(&self) -> Point<C> {
        Point::<C>::new(self.x, self.y, self.z)
    }

    pub fn is_on_curve(&self) -> bool {
        let a = FieldElement::new(C::COEFF_A);
        let b = FieldElement::new(C::COEFF_B);

        let y2 = self.y * self.y;
        let x3 = self.x * self.x * self.x;
        let ax = a * self.x;
        let t5 = y2 - (x3 + ax + b);

        t5.inner() == &U256::ZERO
    }

    pub fn double(&self) -> Point<C> {
        self + self
    }

    pub fn geometric_add(&self, rhs: &Self) -> Point<C> {
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

        Point::<C>::new(sum_x, sum_y, sum_z)
    }

    pub fn scalar_mul(&self, scalar: &Scalar<C>) -> Point<C> {
        let mut q = Point::<C>::IDENTITY;
        let mut current = Point::<C>::IDENTITY;
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
    ) -> Point<C> {
        let mut q = Point::<C>::IDENTITY;
        let mut this_current = Point::<C>::IDENTITY;
        let mut other_current = Point::<C>::IDENTITY;
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

    pub fn x(&self) -> &FieldElement<C> {
        &self.x
    }

    pub fn y(&self) -> &FieldElement<C> {
        &self.y
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

        let tom_on_sec = SecPoint::new(
            FieldElement::new(Tom256k1::GENERATOR_X),
            FieldElement::new(Tom256k1::GENERATOR_Y),
            FieldElement::ONE,
        )
        .into_affine();

        let sec_on_tom = TomPoint::new(
            FieldElement::new(Secp256k1::GENERATOR_X),
            FieldElement::new(Secp256k1::GENERATOR_Y),
            FieldElement::ONE,
        )
        .into_affine();
        assert!(!tom_on_sec.is_on_curve());
        assert!(!sec_on_tom.is_on_curve());
    }

    #[test]
    fn point_addition() {
        let g2 = SecPoint::GENERATOR.double().into_affine();
        assert_eq!(
            g2.x().inner(),
            &U256::from_be_hex("c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5")
        );
        assert_eq!(
            g2.y().inner(),
            &U256::from_be_hex("1ae168fea63dc339a3c58419466ceaeef7f632653266d0e1236431a950cfe52a")
        );
        assert_eq!(
            g2.z().inner(),
            &U256::from_be_hex("0000000000000000000000000000000000000000000000000000000000000001")
        );

        let random_double = SecPoint::new(
            FieldElement::new(U256::from_be_hex(
                "B8F0170E293FCC9291BEE2665E9CA9B25D3B11810ED68D9EA0CB440D7064E4DA",
            )),
            FieldElement::new(U256::from_be_hex(
                "0691AA44502212591132AA6F27582B78F9976998DE355C4EE5960DB05AC0A2A3",
            )),
            FieldElement::ONE,
        )
        .into_affine()
        .double()
        .into_affine();
        assert!(random_double.is_on_curve());
        assert_eq!(
            random_double.x().inner(),
            &U256::from_be_hex("d99bdf80fe99540ed7c33669cc43ac72fa2fa2c14b731ae6758c1c17eaf7b26e")
        );
        assert_eq!(
            random_double.y().inner(),
            &U256::from_be_hex("cac2c38a379655150567315c7cf7f596585b577b28e03108b0d2df2b9c83af52")
        );
        assert_eq!(random_double.z().inner(), &U256::ONE);

        let four = SecScalar::new(U256::from_u8(4));
        let g4 = SecPoint::GENERATOR.scalar_mul(&four);
        assert_eq!(g2.double(), g4);
        assert_eq!(&g2 + &g2, g4);
    }

    #[test]
    fn affine_point() {
        let g2 = SecPoint::GENERATOR.double();
        let g2_affine = g2.into_affine();
        assert_eq!(
            g2_affine.x().inner(),
            &U256::from_be_hex("c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5")
        );
        assert_eq!(
            g2_affine.y().inner(),
            &U256::from_be_hex("1ae168fea63dc339a3c58419466ceaeef7f632653266d0e1236431a950cfe52a")
        );
        assert_eq!(g2_affine.z(), &FieldElement::ONE);

        let id_aff = SecPoint::IDENTITY.into_affine();
        assert_eq!(id_aff, SecPoint::IDENTITY.into_affine());

        let g5 = SecPoint::GENERATOR
            .scalar_mul(&SecScalar::new(U256::from_u8(5)))
            .into_affine();
        let g2 = SecPoint::GENERATOR.double().into_affine();
        let g4 = g2.double().into_affine();
        assert_eq!((g4 + SecPoint::GENERATOR).into_affine(), g5);
    }
}
