use crate::field::FieldElement;

pub struct Point {
    x: FieldElement,
    y: FieldElement,
    z: FieldElement,
    t: FieldElement,
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        todo!();
    }
}
