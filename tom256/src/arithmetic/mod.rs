mod affine_point;
mod field;
mod modular;
pub mod multimult;
mod point;
pub mod point_arithmetic;
mod scalar;

pub use field::FieldElement;
pub use modular::Modular;
pub use point_arithmetic::*;
pub use scalar::Scalar;

use crate::curve::Curve;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Point<C: Curve> {
    x: FieldElement<C>,
    y: FieldElement<C>,
    z: FieldElement<C>,
}

// z can only be 1 (general point) or 0 (identity)
// This invariable is preserved in the methods
#[derive(Debug, Clone)]
pub struct AffinePoint<C: Curve> {
    x: FieldElement<C>,
    y: FieldElement<C>,
    z: FieldElement<C>,
}
