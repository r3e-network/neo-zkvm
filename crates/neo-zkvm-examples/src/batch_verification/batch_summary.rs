pub(super) struct BatchSummary {
    pub(super) total: usize,
    pub(super) valid: usize,
    pub(super) invalid_jobs: Vec<String>,
}
