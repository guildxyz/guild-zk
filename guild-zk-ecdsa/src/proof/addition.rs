use super::equality::EqualityProof;
use super::multiplication::MultiplicationProof;
use crate::eval::MultiMult;
use crate::pedersen::Parameters;
use crate::CycleCurveConfig;
use ark_ec::short_weierstrass::{Affine, Projective, SWCurveConfig};
use ark_ec::{AffineRepr, CurveConfig, CurveGroup};
use ark_ff::{Field, One, Zero};
use ark_std::{rand::Rng, UniformRand};

pub struct Commitment<C: SWCurveConfig> {
    pub px: Affine<C>,
    pub py: Affine<C>,
    pub qx: Affine<C>,
    pub qy: Affine<C>,
    pub rx: Affine<C>,
    pub ry: Affine<C>,
}

pub struct Randomness<C: SWCurveConfig> {
    pub px: C::ScalarField,
    pub py: C::ScalarField,
    pub qx: C::ScalarField,
    pub qy: C::ScalarField,
    pub rx: C::ScalarField,
    pub ry: C::ScalarField,
}

/// Proof that `P + Q = R` where all three points are on curve `C`.
pub struct AdditionProof<C, CC: SWCurveConfig> {
    base: core::marker::PhantomData<C>,
    eq_proof_x: EqualityProof<CC>,
    eq_proof_y: EqualityProof<CC>,
    mul_proof_8: MultiplicationProof<CC>,
    mul_proof_10: MultiplicationProof<CC>,
    mul_proof_11: MultiplicationProof<CC>,
    mul_proof_13: MultiplicationProof<CC>,
    commitment_to_aux_8: Affine<CC>,
    commitment_to_aux_10: Affine<CC>,
    commitment_to_aux_11: Affine<CC>,
    commitment_to_aux_13: Affine<CC>,
}

impl<C, CC> AdditionProof<C, CC>
where
    C: SWCurveConfig,
    CC: SWCurveConfig + CycleCurveConfig<BaseCurveConfig = C>,
    Projective<C>: CurveGroup<
        BaseField = <CC as CurveConfig>::ScalarField,
        ScalarField = <CC as CurveConfig>::BaseField,
    >,
{
    pub fn construct<R: Rng + ?Sized>(
        rng: &mut R,
        parameters: &Parameters<CC>,
        commitment: &Commitment<CC>,
        randomness: &Randomness<CC>,
        point_p: Affine<C>,
        point_q: Affine<C>,
        point_r: Affine<C>,
    ) -> Self {
        // NOTE unwraps below are fine because it is
        // assumed that all input points are valid
        let aux_7 = *point_q.x().unwrap() - point_p.x().unwrap();
        let aux_8 = aux_7.inverse().unwrap_or_default();
        let aux_9 = *point_q.y().unwrap() - point_p.y().unwrap();
        let aux_10 = aux_8 * aux_9;
        let aux_11 = aux_10 * aux_10;
        let aux_12 = *point_p.x().unwrap() - point_r.x().unwrap();
        let aux_13 = aux_10 * aux_12;

        let aux_7 = CC::to_cycle(&aux_7);
        let randomness_to_aux_7 = randomness.qx - randomness.px;
        let commitment_to_aux_7 = (commitment.qx - commitment.px).into();
        let aux_8 = CC::to_cycle(&aux_8);
        let randomness_to_aux_8 = CC::ScalarField::rand(rng);
        let commitment_to_aux_8 = parameters.commit(aux_8, randomness_to_aux_8);
        let aux_9 = CC::to_cycle(&aux_9);
        let randomness_to_aux_9 = randomness.qy - randomness.py;
        let commitment_to_aux_9 = (commitment.qy - commitment.py).into();
        let aux_10 = CC::to_cycle(&aux_10);
        let randomness_to_aux_10 = CC::ScalarField::rand(rng);
        let commitment_to_aux_10 = parameters.commit(aux_10, randomness_to_aux_10);
        let aux_11 = CC::to_cycle(&aux_11);
        let randomness_to_aux_11 = CC::ScalarField::rand(rng);
        let commitment_to_aux_11 = parameters.commit(aux_11, randomness_to_aux_11);
        let aux_12 = CC::to_cycle(&aux_12);
        let randomness_to_aux_12 = CC::ScalarField::rand(rng);
        let commitment_to_aux_12 = parameters.commit(aux_12, randomness_to_aux_12);
        let aux_13 = CC::to_cycle(&aux_13);
        let randomness_to_aux_13 = CC::ScalarField::rand(rng);
        let commitment_to_aux_13 = parameters.commit(aux_13, randomness_to_aux_13);

        let mul_proof_8 = MultiplicationProof::construct(
            rng,
            parameters,
            commitment_to_aux_7,
            commitment_to_aux_8,
            CC::GENERATOR,
            randomness_to_aux_7,
            randomness_to_aux_8,
            CC::ScalarField::zero(),
            aux_7,
            aux_8,
            CC::ScalarField::one(),
        );

        let mul_proof_10 = MultiplicationProof::construct(
            rng,
            parameters,
            commitment_to_aux_8,
            commitment_to_aux_9,
            commitment_to_aux_10,
            randomness_to_aux_8,
            randomness_to_aux_9,
            randomness_to_aux_10,
            aux_8,
            aux_9,
            aux_10,
        );

        let mul_proof_11 = MultiplicationProof::construct(
            rng,
            parameters,
            commitment_to_aux_10,
            commitment_to_aux_10,
            commitment_to_aux_11,
            randomness_to_aux_10,
            randomness_to_aux_10,
            randomness_to_aux_11,
            aux_10,
            aux_10,
            aux_11,
        );

        let mul_proof_13 = MultiplicationProof::construct(
            rng,
            parameters,
            commitment_to_aux_10,
            commitment_to_aux_12,
            commitment_to_aux_13,
            randomness_to_aux_10,
            randomness_to_aux_12,
            randomness_to_aux_13,
            aux_10,
            aux_12,
            aux_13,
        );

        let eq_proof_x = EqualityProof::construct(
            rng,
            parameters,
            commitment_to_aux_11,
            (commitment.rx + commitment.px + commitment.qx).into(),
            randomness_to_aux_11,
            randomness.rx + randomness.px + randomness.qx,
            aux_11,
        );

        let eq_proof_y = EqualityProof::construct(
            rng,
            parameters,
            commitment_to_aux_13,
            (commitment.ry + commitment.py).into(),
            randomness_to_aux_13,
            randomness.ry + randomness.py,
            aux_13,
        );

        Self {
            base: core::marker::PhantomData,
            eq_proof_x,
            eq_proof_y,
            mul_proof_8,
            mul_proof_10,
            mul_proof_11,
            mul_proof_13,
            commitment_to_aux_8,
            commitment_to_aux_10,
            commitment_to_aux_11,
            commitment_to_aux_13,
        }
    }

    pub fn aggregate<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        parameters: &Parameters<CC>,
        commitment: &Commitment<CC>,
        multimult: &mut MultiMult<CC>,
    ) {
        let commitment_to_aux_7 = (commitment.qx - commitment.px).into();
        let commitment_to_aux_9 = (commitment.qy - commitment.py).into();
        let commitment_to_aux_12 = (commitment.px - commitment.rx).into();

        self.mul_proof_8.aggregate(
            rng,
            parameters,
            commitment_to_aux_7,
            self.commitment_to_aux_8,
            CC::GENERATOR,
            multimult,
        );

        self.mul_proof_10.aggregate(
            rng,
            parameters,
            self.commitment_to_aux_8,
            commitment_to_aux_9,
            self.commitment_to_aux_10,
            multimult,
        );

        self.mul_proof_11.aggregate(
            rng,
            parameters,
            self.commitment_to_aux_10,
            self.commitment_to_aux_10,
            self.commitment_to_aux_11,
            multimult,
        );

        self.mul_proof_13.aggregate(
            rng,
            parameters,
            self.commitment_to_aux_10,
            commitment_to_aux_12,
            self.commitment_to_aux_13,
            multimult,
        );

        self.eq_proof_x.aggregate(
            rng,
            parameters,
            self.commitment_to_aux_11,
            (commitment.rx + commitment.px + commitment.qx).into(),
            multimult,
        );

        self.eq_proof_y.aggregate(
            rng,
            parameters,
            self.commitment_to_aux_13,
            (commitment.py + commitment.ry).into(),
            multimult,
        );
    }

    pub fn verify<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        parameters: &Parameters<CC>,
        commitment: &Commitment<CC>,
    ) -> bool {
        let mut multimult = MultiMult::new();
        self.aggregate(rng, parameters, commitment, &mut multimult);
        multimult.evaluate() == Affine::identity()
    }
}

// TODO test what happens if P = Q
#[cfg(test)]
mod test {
    use super::*;
    use ark_ec::CurveConfig;
    use ark_secp256k1::Config as SecpConfig;
    use ark_secq256k1::Config as SecqConfig;
    use ark_std::{
        rand::{rngs::StdRng, SeedableRng},
        UniformRand,
    };

    const SEED: u64 = 1234567890;

    #[test]
    fn completeness() {
        let mut rng = StdRng::seed_from_u64(SEED);
        let p = Affine::from(
            SecpConfig::GENERATOR * <SecpConfig as CurveConfig>::ScalarField::rand(&mut rng),
        );
        let q = Affine::from(
            SecpConfig::GENERATOR * <SecpConfig as CurveConfig>::ScalarField::rand(&mut rng),
        );
        let r = Affine::from(p + q);

        let parameters = Parameters::<SecqConfig>::new(&mut rng);

        let randomness = Randomness {
            px: <SecqConfig as CurveConfig>::ScalarField::rand(&mut rng),
            py: <SecqConfig as CurveConfig>::ScalarField::rand(&mut rng),
            qx: <SecqConfig as CurveConfig>::ScalarField::rand(&mut rng),
            qy: <SecqConfig as CurveConfig>::ScalarField::rand(&mut rng),
            rx: <SecqConfig as CurveConfig>::ScalarField::rand(&mut rng),
            ry: <SecqConfig as CurveConfig>::ScalarField::rand(&mut rng),
        };

        let commitment = Commitment {
            px: parameters.commit(SecqConfig::to_cycle(p.x().unwrap()), randomness.px),
            py: parameters.commit(SecqConfig::to_cycle(p.y().unwrap()), randomness.py),
            qx: parameters.commit(SecqConfig::to_cycle(q.x().unwrap()), randomness.qx),
            qy: parameters.commit(SecqConfig::to_cycle(q.y().unwrap()), randomness.qy),
            rx: parameters.commit(SecqConfig::to_cycle(r.x().unwrap()), randomness.rx),
            ry: parameters.commit(SecqConfig::to_cycle(r.y().unwrap()), randomness.ry),
        };

        let proof =
            AdditionProof::construct(&mut rng, &parameters, &commitment, &randomness, p, q, r);

        assert!(proof.verify(&mut rng, &parameters, &commitment));
    }
}
