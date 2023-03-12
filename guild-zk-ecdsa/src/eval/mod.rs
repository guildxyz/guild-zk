mod multimult;
mod pair;
mod relation;

pub use multimult::MultiMult;
pub use pair::Pair;
pub use relation::Relation;

use std::collections::binary_heap::BinaryHeap;

fn heapify_vec<T: std::cmp::Ord>(vec: Vec<T>) -> BinaryHeap<T> {
    vec.into_iter().collect()
}
