use super::ty::WindType;

#[derive(Debug, Clone, PartialEq)]
pub enum WindGroupRule {
    Simple { field: String, ty: WindType },
    SelfField { field: String, ty: WindType },
}
