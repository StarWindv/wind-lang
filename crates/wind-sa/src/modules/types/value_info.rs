use crate::modules::types::{ValueKind, WindResolvedType, WindValueID};


#[derive(Debug, Clone)]
pub struct ValueInfo {
    pub id: WindValueID,
    pub ty: Option<WindResolvedType>,
    pub kind: ValueKind,
    pub ref_count: usize,
}
