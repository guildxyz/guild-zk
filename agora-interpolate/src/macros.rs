macro_rules! test_polynomial {
    ($scalar: ty, $point: ty) => {
        #[cfg(test)]
        mod test {
            use crate::{GroupElement, Interpolate, InterpolationError, Polynomial};
            use std::ops::Neg;

            type TestScalar = $scalar;
            type TestPoint = $point;

            #[test]
            fn interpolate_and_evaluate_with_scalars() {
                // input slices of unequal length
                let x = vec![<TestScalar as Interpolate>::from_u64(3_u64); 3];
                let y = vec![<TestScalar as Interpolate>::from_u64(5_u64); 4];
                assert_eq!(
                    Polynomial::interpolate(&x, &y),
                    Err(InterpolationError::InvalidInputLengths(3, 4))
                );

                // constant polynomial (y = 53)
                let x = vec![<TestScalar as Interpolate>::from_u64(3_u64); 1];
                let y = vec![<TestScalar as Interpolate>::from_u64(53_u64); 1];
                let poly = Polynomial::interpolate(&x, &y).unwrap();
                assert_eq!(
                    poly.coeffs()[0],
                    <TestScalar as Interpolate>::from_u64(53_u64)
                );
                assert_eq!(
                    poly.evaluate(<TestScalar as Interpolate>::from_u64(123456_u64),),
                    <TestScalar as Interpolate>::from_u64(53_u64),
                );
                assert_eq!(
                    poly.evaluate(<TestScalar as Interpolate>::from_u64(78910_u64),),
                    <TestScalar as Interpolate>::from_u64(53_u64),
                );

                // simple first order polynomial (y = x)
                let x = vec![
                    <TestScalar as Interpolate>::from_u64(1_u64),
                    <TestScalar as Interpolate>::from_u64(2_u64),
                    <TestScalar as Interpolate>::from_u64(3_u64),
                ];

                let y = x.clone();
                let poly = Polynomial::interpolate(&x, &y).unwrap();
                assert_eq!(poly.coeffs()[0], <TestScalar as Interpolate>::zero()); // c_0
                assert_eq!(poly.coeffs()[1], <TestScalar as Interpolate>::one()); // c_1

                // first order polynomial (y = 32 * x - 13)
                let x = vec![
                    <TestScalar as Interpolate>::from_u64(2_u64),
                    <TestScalar as Interpolate>::from_u64(3_u64),
                ];
                let y = vec![
                    <TestScalar as Interpolate>::from_u64(51_u64),
                    <TestScalar as Interpolate>::from_u64(83_u64),
                ];
                let poly = Polynomial::interpolate(&x, &y).unwrap();
                assert_eq!(
                    poly.coeffs()[0],
                    <TestScalar as Interpolate>::from_u64(13_u64).neg()
                );
                assert_eq!(
                    poly.coeffs()[1],
                    <TestScalar as Interpolate>::from_u64(32_u64)
                );

                assert_eq!(poly.evaluate(x[0]), y[0]);
                assert_eq!(poly.evaluate(x[1]), y[1]);
                assert_eq!(
                    poly.evaluate(<TestScalar as Interpolate>::from_u64(100_u64)),
                    <TestScalar as Interpolate>::from_u64(3187_u64)
                );

                // fourth order polynomial
                // y = x^4 + 0 * x^3 + 3 * x^2 + 2 * x + 14
                let x = vec![
                    <TestScalar as Interpolate>::from_u64(1_u64),
                    <TestScalar as Interpolate>::from_u64(2_u64),
                    <TestScalar as Interpolate>::from_u64(3_u64),
                    <TestScalar as Interpolate>::from_u64(4_u64),
                    <TestScalar as Interpolate>::from_u64(5_u64),
                    <TestScalar as Interpolate>::from_u64(6_u64),
                ];
                let y = vec![
                    <TestScalar as Interpolate>::from_u64(20_u64),
                    <TestScalar as Interpolate>::from_u64(46_u64),
                    <TestScalar as Interpolate>::from_u64(128_u64),
                    <TestScalar as Interpolate>::from_u64(326_u64),
                    <TestScalar as Interpolate>::from_u64(724_u64),
                    <TestScalar as Interpolate>::from_u64(1430_u64),
                ];
                let poly = Polynomial::interpolate(&x, &y).unwrap();
                assert_eq!(
                    poly.coeffs()[0],
                    <TestScalar as Interpolate>::from_u64(14_u64)
                ); // c0 (x^0)
                assert_eq!(
                    poly.coeffs()[1],
                    <TestScalar as Interpolate>::from_u64(2_u64)
                ); // c1 (x^1)
                assert_eq!(
                    poly.coeffs()[2],
                    <TestScalar as Interpolate>::from_u64(3_u64)
                ); // c2 (x^2)
                assert_eq!(
                    poly.coeffs()[3],
                    <TestScalar as Interpolate>::from_u64(0_u64)
                ); // c3 (x^3)
                assert_eq!(
                    poly.coeffs()[4],
                    <TestScalar as Interpolate>::from_u64(1_u64)
                ); // c4 (x^4)

                assert_eq!(poly.evaluate(x[0]), y[0]);
                assert_eq!(poly.evaluate(x[1]), y[1]);
                assert_eq!(poly.evaluate(x[2]), y[2]);
                assert_eq!(poly.evaluate(x[3]), y[3]);
                assert_eq!(poly.evaluate(x[4]), y[4]);
            }

            #[test]
            fn interpolate_and_evaluate_with_points() {
                // constant polynomial y = G
                let x = vec![
                    <TestScalar as Interpolate>::from_u64(10_u64),
                    <TestScalar as Interpolate>::from_u64(111_u64),
                    <TestScalar as Interpolate>::from_u64(1222_u64),
                ];
                let y = vec![
                    <TestPoint as GroupElement>::generator(),
                    <TestPoint as GroupElement>::generator(),
                    <TestPoint as GroupElement>::generator(),
                ];

                let poly = Polynomial::interpolate(&x, &y).unwrap();
                assert_eq!(poly.coeffs()[0], <TestPoint as GroupElement>::generator());
                assert_eq!(poly.coeffs()[1], <TestPoint as GroupElement>::identity());
                assert_eq!(poly.coeffs()[2], <TestPoint as GroupElement>::identity());

                assert_eq!(
                    poly.evaluate(x[0]),
                    <TestPoint as GroupElement>::generator()
                );
                assert_eq!(
                    poly.evaluate(<TestScalar as Interpolate>::from_u64(0_u64)),
                    <TestPoint as GroupElement>::generator()
                );

                // third order polynomial y = 2G + 5G * x + G * x^2 + G * x^3
                let x = vec![
                    <TestScalar as Interpolate>::from_u64(0_u64),
                    <TestScalar as Interpolate>::from_u64(1_u64),
                    <TestScalar as Interpolate>::from_u64(2_u64),
                    <TestScalar as Interpolate>::from_u64(3_u64),
                ];

                let y = vec![
                    <TestPoint as GroupElement>::generator()
                        * <TestScalar as Interpolate>::from_u64(2_u64),
                    <TestPoint as GroupElement>::generator()
                        * <TestScalar as Interpolate>::from_u64(9_u64),
                    <TestPoint as GroupElement>::generator()
                        * <TestScalar as Interpolate>::from_u64(24_u64),
                    <TestPoint as GroupElement>::generator()
                        * <TestScalar as Interpolate>::from_u64(53_u64),
                ];

                let poly = Polynomial::interpolate(&x, &y).unwrap();
                assert_eq!(
                    poly.coeffs()[0],
                    <TestPoint as GroupElement>::generator()
                        * <TestScalar as Interpolate>::from_u64(2_u64)
                );
                assert_eq!(
                    poly.coeffs()[1],
                    <TestPoint as GroupElement>::generator()
                        * <TestScalar as Interpolate>::from_u64(5_u64)
                );
                assert_eq!(poly.coeffs()[2], <TestPoint as GroupElement>::generator());
                assert_eq!(poly.coeffs()[3], <TestPoint as GroupElement>::generator());

                assert_eq!(
                    poly.evaluate(<TestScalar as Interpolate>::from_u64(0_u64)),
                    <TestPoint as GroupElement>::generator()
                        * <TestScalar as Interpolate>::from_u64(2_u64)
                );
                assert_eq!(
                    poly.evaluate(<TestScalar as Interpolate>::from_u64(5_u64)),
                    <TestPoint as GroupElement>::generator()
                        * <TestScalar as Interpolate>::from_u64(177_u64)
                );
            }

            #[test]
            fn arithmetic_check_evaluate() {
                let gen = <TestPoint as GroupElement>::generator();
                let secret_key = <TestScalar as Interpolate>::from_u64(123456789_u64);

                // p(x) = 111 + 222x
                let secret_coeffs = vec![
                    <TestScalar as Interpolate>::from_u64(111_u64),
                    <TestScalar as Interpolate>::from_u64(222_u64),
                ];

                // pg(x) = g^(111) + (g^(222))^x = g^(111 + 222x)
                let public_coeffs = secret_coeffs
                    .iter()
                    .map(|coeff| &gen * coeff)
                    .collect::<Vec<TestPoint>>();

                let secret_poly = Polynomial::new(secret_coeffs);
                let public_poly = Polynomial::new(public_coeffs);

                let secret_eval = secret_poly.evaluate(secret_key);
                let public_eval = public_poly.evaluate(secret_key);

                assert_eq!(gen * secret_eval, public_eval);
            }
        }
    };
}

pub(crate) use test_polynomial;
