#[derive(Debug, Clone, Copy)]
pub struct Parameters {
    nodes: usize,
    threshold: usize,
}

impl Parameters {
    /// Panics when `nodes < threshold`
    pub fn new(nodes: usize, threshold: usize) -> Self {
        assert!(
            nodes >= threshold,
            "threshold is greater than the total number of participants"
        );
        Self { nodes, threshold }
    }

    pub fn nodes(&self) -> usize {
        self.nodes
    }

    pub fn threshold(&self) -> usize {
        self.threshold
    }
}
