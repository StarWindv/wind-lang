use crate::modules::types::WindTypeRef;

#[derive(Debug, Clone)]
pub struct GroupRuleInfo {
    pub kind: GroupRuleKind,
    pub ty: WindTypeRef,
}

#[derive(Debug, Clone)]
pub enum GroupRuleKind {
    Simple { param: String },
    SelfField { field: String },
}
