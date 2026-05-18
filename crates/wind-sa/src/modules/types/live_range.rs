use crate::modules::types::WindValueID;

#[derive(Debug, Clone)]
pub struct LiveRange {
    pub value: WindValueID,
    pub description: String,
    pub born_at: usize,
    pub last_use: usize,
    pub drop_at: Option<usize>,
    pub dropped_by_scope_exit: bool,
}
