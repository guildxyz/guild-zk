#[derive(Debug, Clone, Copy)]
pub struct Parameters {
    nodes: usize,
    threshold: usize,
    original_member: bool,
}

impl Parameters {
    /// Panics when `nodes < threshold or threshold is zero
    pub fn new(threshold: usize, nodes: usize, original_member: bool) -> Self {
        assert!(nodes >= threshold && threshold > 0, "invalid parameters");
        Self {
            nodes,
            threshold,
            original_member,
        }
    }

    pub fn nodes(&self) -> usize {
        self.nodes
    }

    pub fn threshold(&self) -> usize {
        self.threshold
    }

    pub fn is_original_member(&self) -> bool {
        self.original_member
    }

    pub fn set_original_member(&mut self) {
        self.original_member = true;
    }
}
