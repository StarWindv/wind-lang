use crate::modules::types::WindValueID;

#[derive(Debug, Clone, PartialEq)]
pub enum ValueKind {
    ScalarConst,
    Allocated,
    Copied,
    Reference { target: WindValueID },
    Returned { source: WindValueID },
}
