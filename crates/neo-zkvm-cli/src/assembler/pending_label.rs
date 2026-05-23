#[derive(Debug, Clone)]
pub(super) struct PendingLabel {
    pub(super) pos: usize,
    pub(super) base_ip: usize,
    pub(super) label: String,
    pub(super) line_num: usize,
    pub(super) is_long_jump: bool,
}
