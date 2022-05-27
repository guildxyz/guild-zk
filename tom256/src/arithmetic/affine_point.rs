use super::field::FieldElement;
use super::modular::Modular;
use super::Point;
use crate::arithmetic::AffinePoint;
use crate::curve::Curve;

impl<C: Curve + PartialEq> PartialEq for AffinePoint<C> {
    fn eq(&self, other: &Self) -> bool {
        (self.is_identity() && other.is_identity()) || (self.x == other.x && self.y == other.y)
    }
}

impl<C: Curve> From<Point<C>> for AffinePoint<C> {
    fn from(point: Point<C>) -> AffinePoint<C> {
        if point.is_identity() {
            AffinePoint::<C>::IDENTITY
        } else {
            let z_inv = point.z.inverse();
            AffinePoint::<C>::new(point.x * z_inv, point.y * z_inv, FieldElement::<C>::ONE)
        }
    }
}

impl<C: Curve> From<&Point<C>> for AffinePoint<C> {
    fn from(point: &Point<C>) -> AffinePoint<C> {
        if point.is_identity() {
            AffinePoint::<C>::IDENTITY
        } else {
            let z_inv = point.z.inverse();
            AffinePoint::<C>::new(point.x * z_inv, point.y * z_inv, FieldElement::<C>::ONE)
        }
    }
}

impl<C: Curve> From<AffinePoint<C>> for Point<C> {
    fn from(point: AffinePoint<C>) -> Point<C> {
        Point::<C>::new(point.x, point.y, point.z)
    }
}

impl<C: Curve> From<&AffinePoint<C>> for Point<C> {
    fn from(point: &AffinePoint<C>) -> Point<C> {
        Point::<C>::new(point.x, point.y, point.z)
    }
}

impl<C: Curve> AffinePoint<C> {
    pub fn new(x: FieldElement<C>, y: FieldElement<C>, z: FieldElement<C>) -> Self {
        let z = if z != FieldElement::<C>::ZERO {
            FieldElement::<C>::ONE
        } else {
            FieldElement::<C>::ZERO
        };

        Self { x, y, z }
    }

    #[inline(always)]
    pub fn x(&self) -> &FieldElement<C> {
        &self.x
    }

    #[inline(always)]
    pub fn y(&self) -> &FieldElement<C> {
        &self.y
    }

    #[inline(always)]
    pub fn z(&self) -> &FieldElement<C> {
        &self.z
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::arithmetic::Scalar;
    use crate::{Secp256k1, Tom256k1};

    use bigint::U256;

    type SecPoint = Point<Secp256k1>;
    type TomPoint = Point<Tom256k1>;

    type SecAffine = AffinePoint<Secp256k1>;

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
        );

        let sec_on_tom = TomPoint::new(
            FieldElement::new(Secp256k1::GENERATOR_X),
            FieldElement::new(Secp256k1::GENERATOR_Y),
            FieldElement::ONE,
        );
        assert!(!tom_on_sec.is_on_curve());
        assert!(!sec_on_tom.is_on_curve());
    }

    #[test]
    fn point_addition() {
        let g2: SecAffine = SecPoint::GENERATOR.double().into();
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

        let random_double: SecAffine = SecPoint::new(
            FieldElement::new(U256::from_be_hex(
                "B8F0170E293FCC9291BEE2665E9CA9B25D3B11810ED68D9EA0CB440D7064E4DA",
            )),
            FieldElement::new(U256::from_be_hex(
                "0691AA44502212591132AA6F27582B78F9976998DE355C4EE5960DB05AC0A2A3",
            )),
            FieldElement::ONE,
        )
        .double()
        .into();
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
        let g2_affine: SecAffine = g2.into();
        assert_eq!(
            g2_affine.x().inner(),
            &U256::from_be_hex("c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5")
        );
        assert_eq!(
            g2_affine.y().inner(),
            &U256::from_be_hex("1ae168fea63dc339a3c58419466ceaeef7f632653266d0e1236431a950cfe52a")
        );
        assert_eq!(g2_affine.z(), &FieldElement::ONE);

        let id_aff: SecAffine = SecPoint::IDENTITY.into();
        assert_eq!(id_aff, SecPoint::IDENTITY.into());

        let g5: SecAffine = SecPoint::GENERATOR
            .scalar_mul(&SecScalar::new(U256::from_u8(5)))
            .into();
        let g2: SecAffine = SecPoint::GENERATOR.double().into();
        let g4: SecAffine = g2.double().into();
        assert_eq!(g5, (g4 + SecAffine::GENERATOR).into());
    }
}
