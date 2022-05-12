use super::field::FieldElement;
use super::modular::{mul_mod_u256, Modular};
use super::scalar::Scalar;
use crate::Curve;

use bigint::U256;

use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

const BASE_16_DIGITS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
];

#[derive(Debug, Clone)]
pub struct Point<C: Curve> {
    x: FieldElement<C>,
    y: FieldElement<C>,
    z: FieldElement<C>,
}

impl<C: Curve> fmt::Display for Point<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f)?;
        writeln!(f, "x: {}", self.x.inner())?;
        writeln!(f, "y: {}", self.y.inner())?;
        writeln!(f, "z: {}", self.z.inner())
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

impl<C: Curve> std::ops::Neg for &Point<C> {
    type Output = Point<C>;
    fn neg(self) -> Self::Output {
        Point {
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

impl<'a, 'b, C: Curve> std::ops::Add<&'b Point<C>> for &'a Point<C> {
    type Output = Point<C>;
    fn add(self, rhs: &'b Point<C>) -> Self::Output {
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

impl<'a, 'b, C: Curve> std::ops::Sub<&'b Point<C>> for &'a Point<C> {
    type Output = Point<C>;
    fn sub(self, rhs: &'b Point<C>) -> Self::Output {
        self + &(-rhs)
    }
}

impl<C: Curve> std::ops::Mul<Scalar<C>> for &Point<C> {
    type Output = Point<C>;
    fn mul(self, rhs: Scalar<C>) -> Self::Output {
        self.scalar_mul(&rhs)
    }
}

impl<'a, 'b, C: Curve> std::ops::Mul<&'b Scalar<C>> for &'a Point<C> {
    type Output = Point<C>;
    fn mul(self, rhs: &'b Scalar<C>) -> Self::Output {
        self.scalar_mul(rhs)
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

    pub fn new(x: FieldElement<C>, y: FieldElement<C>, z: FieldElement<C>) -> Self {
        Self { x, y, z }
    }

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

    pub fn to_affine(&self) -> Self {
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

        let tom_on_sec = SecPoint {
            x: FieldElement::new(Tom256k1::GENERATOR_X),
            y: FieldElement::new(Tom256k1::GENERATOR_Y),
            z: FieldElement::ONE,
        };

        let sec_on_tom = TomPoint {
            x: FieldElement::new(Secp256k1::GENERATOR_X),
            y: FieldElement::new(Secp256k1::GENERATOR_Y),
            z: FieldElement::ONE,
        };
        assert!(!tom_on_sec.is_on_curve());
        assert!(!sec_on_tom.is_on_curve());
    }

    #[test]
    fn point_addition() {
        let g2 = SecPoint::GENERATOR.double();
        assert_eq!(
            g2.x().inner(),
            &U256::from_be_hex("f40af3b6c6fdf9aa5402b9fdc39ac4b67827eb373c92077452348e044f109fc8")
        );
        assert_eq!(
            g2.y().inner(),
            &U256::from_be_hex("56915849f52cc8f76f5fd7e4bf60db4a43bf633e1b1383f85fe89164bfadcbdb")
        );
        assert_eq!(
            g2.z().inner(),
            &U256::from_be_hex("f8783c53dfb2a307b568a6ad931fc97023dc71cdc3eac498b0c6ba5554759a29")
        );

        let random_double = SecPoint {
            x: FieldElement::new(U256::from_be_hex(
                "B8F0170E293FCC9291BEE2665E9CA9B25D3B11810ED68D9EA0CB440D7064E4DA",
            )),
            y: FieldElement::new(U256::from_be_hex(
                "0691AA44502212591132AA6F27582B78F9976998DE355C4EE5960DB05AC0A2A3",
            )),
            z: FieldElement::ONE,
        }
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
    fn scalar_multiplication() {
        let d = TomScalar::new(U256::from_be_hex(
            "c51e4753afdec1e6b6c6a5b992f43f8dd0c7a8933072708b6522468b2ffb06fd",
        ));
        let e = TomScalar::new(U256::from_be_hex(
            "d37f628ece72a462f0145cbefe3f0b355ee8332d37acdd83a358016aea029db7",
        ));
        let f = TomScalar::new(U256::from_be_hex(
            "B8F0170E293FCC9291BEE2665E9CA9B25D3B11810ED68D9EA0CB440D7064E4DA",
        ));

        let t = TomPoint::GENERATOR.scalar_mul(&d).into_affine();
        assert!(t.is_on_curve());
        assert_eq!(
            t.x().inner(),
            &U256::from_be_hex("3758fd961003dc291e21523313f0b4329d732b84e52f0159b2d6627bca8d2db2")
        );
        assert_eq!(
            t.y().inner(),
            &U256::from_be_hex("0c21e4f939a5d91c1473416bb936e61bd688dd91db2778f832a54cdacc207deb")
        );

        let r = TomPoint::GENERATOR.double_mul(&e, &t, &f).into_affine();
        assert!(r.is_on_curve());
        assert_eq!(
            r.x().inner(),
            &U256::from_be_hex("8fdb6195754109cc23c635f41f799fd6e1f6078eb94fe0d9cde1eb80d36e5e31")
        );
        assert_eq!(
            r.y().inner(),
            &U256::from_be_hex("fddd45b8f6f633074edddcf1394a1c9498e6f7b5847b744adf01833f38553c01")
        );

        let mut g12 = TomPoint::IDENTITY;
        for _ in 0..12 {
            g12 = g12 + TomPoint::GENERATOR;
        }

        assert_eq!(
            TomPoint::GENERATOR.scalar_mul(&TomScalar::new(U256::from_u32(12))),
            g12
        );

        let scalars = &[
            (
                TomScalar::new(U256::from_u8(3)),
                TomScalar::new(U256::from_u8(12)),
            ),
            (
                TomScalar::new(U256::from_u8(36)),
                TomScalar::new(U256::from_u8(220)),
            ),
            (
                TomScalar::new(U256::from_u8(189)),
                TomScalar::new(U256::from_u8(89)),
            ),
            (
                TomScalar::new(U256::from_u8(92)),
                TomScalar::new(U256::from_u8(105)),
            ),
        ];

        let h_gen = TomPoint::GENERATOR.scalar_mul(&TomScalar::new(U256::from_u8(250)));

        for (a, b) in scalars {
            let dbl_mul = h_gen.double_mul(a, &TomPoint::GENERATOR, b);
            let dbl_mul_rev = TomPoint::GENERATOR.double_mul(b, &h_gen, a);
            let expected = &h_gen * *a + &TomPoint::GENERATOR * *b;
            assert_eq!(dbl_mul, expected);
            assert_eq!(dbl_mul_rev, expected);
        }
    }
}
