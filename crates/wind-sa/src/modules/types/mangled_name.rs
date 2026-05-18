use crate::modules::types::WindScopeId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MangledName {
    pub scope_id: WindScopeId,
    pub fn_name: String,
    pub subscope_id: u64,
    pub var_name: String,
    pub version: u64,
}
