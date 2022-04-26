use crate::arithmetic::field::FieldElement;
use crate::arithmetic::modular::{mul_mod_u256, Modular};
use crate::arithmetic::scalar::Scalar;
use crate::Curve;
use crate::point::Point;

use std::cmp::{Eq, PartialEq, PartialOrd, Ord, Ordering};
use std::collections::binary_heap::BinaryHeap;


#[derive(Debug, Clone)]
pub struct Pair<C: Curve> {
    pub scalar: Scalar<C>,
    pub point: Point<C>,
}

pub struct Known<C: Curve> {
    point: Point<C>,
    index: usize,
}

pub struct MultiMult<C: Curve> {
    pairs: Vec<Pair<C>>,
    known: Vec<Known<C>>,
}

impl<C: Curve> MultiMult<C> {
    pub fn add_known(&mut self, pt: Point<C>) {
        if !self.known.iter().any(|known| known.point == pt) {
            self.known.push(Known{point: pt, index: self.known.len()});
        }
    }

    pub fn insert(&mut self, point: Point<C>, scalar: Scalar<C>) {
        if let Some(element) = self.known.iter().find(|known| known.point == point) {
            self.pairs[element.index].scalar = self.pairs[element.index].scalar + scalar;
        } else {
            self.pairs.push(Pair::<C>{point, scalar});
        }
    }

    pub fn evaluate(&mut self) -> Point<C> {
        if self.pairs.len() == 0 {
            return Point::<C>::IDENTITY;
        }
        if self.pairs.len() == 1 {
            return self.pairs[0].point.scalar_mul(&self.pairs[0].scalar);
        }

        let mut pairs_heap = heapify_vec(self.pairs.clone());
        loop {
            // unwrap is fine here because peeking and pre-loop checks guarantee len is at least 1
            let a = pairs_heap.pop().unwrap();
            let c: Pair<C>;
            // If b_option is None -> the heap only has one element
            if let Some(mut b) = pairs_heap.peek_mut() {
                if b.scalar == Scalar::<C>::ZERO {
                    return a.point.scalar_mul(&a.scalar);
                }

                c = Pair {
                    point: a.point.clone(),
                    scalar: a.scalar - b.scalar
                };
                let d = Pair {
                    point: &a.point + &b.point,
                    scalar: b.scalar
                };
                
                *b = d;
            } else {
                return a.point.scalar_mul(&a.scalar);
            }

            if c.scalar != Scalar::<C>::ZERO {
                pairs_heap.push(c);
            }
        }
    }
}

pub fn heapify_vec<T: Ord>(vec: Vec<T>) -> BinaryHeap<T> {
    vec.into_iter().collect()
}


// *************************************** TRAITS ***************************************** //

impl<C: Curve> PartialEq for Pair<C> {
    fn eq(&self, other: &Self) -> bool {
        self.scalar.eq(&other.scalar)
    }
}

impl<C: Curve> Eq for Pair<C> {}

impl <C: Curve> PartialOrd for Pair<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.scalar.partial_cmp(&other.scalar)
    }
}

impl<C: Curve> Ord for Pair<C> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.scalar.cmp(&other.scalar)
    }
}