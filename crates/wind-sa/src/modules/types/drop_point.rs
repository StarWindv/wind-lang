use crate::modules::types::WindValueID;

#[derive(Debug, Clone)]
pub struct DropPoint {
    pub value: WindValueID,
    pub description: String,
    pub at_position: usize,
}
