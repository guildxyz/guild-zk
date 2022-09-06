#[derive(Debug, Clone, Copy)]
pub struct Parameters {
    nodes: usize,
    threshold: usize,
}

impl Parameters {
    /// Panics when `nodes < threshold or threshold is zero
    pub fn new(nodes: usize, threshold: usize) -> Self {
        assert!(nodes >= threshold && threshold > 0, "invalid parameters");
        Self { nodes, threshold }
    }

    pub fn nodes(&self) -> usize {
        self.nodes
    }

    pub fn threshold(&self) -> usize {
        self.threshold
    }
}
