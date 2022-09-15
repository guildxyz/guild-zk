#[derive(Debug, Clone, Copy)]
pub struct Parameters {
    threshold: usize,
    nodes: usize,
}

impl Parameters {
    /// Panics when `nodes < threshold or threshold is zero
    pub fn new(threshold: usize, nodes: usize) -> Self {
        assert!(nodes >= threshold && threshold > 0, "invalid parameters");
        Self { threshold, nodes }
    }

    pub fn nodes(&self) -> usize {
        self.nodes
    }

    pub fn threshold(&self) -> usize {
        self.threshold
    }
}
