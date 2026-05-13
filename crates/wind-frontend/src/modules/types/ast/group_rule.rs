use super::ty::Type;

#[derive(Debug, Clone, PartialEq)]
pub enum GroupRule {
    Simple {
        field: String,
        ty: Type,
    },
    SelfField {
        field: String,
        ty: Type,
    },
}
