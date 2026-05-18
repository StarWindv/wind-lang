use crate::{WindExprRef, WindTypeRef, WindWhichClauseRef};

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub public: bool,
    pub is_static: bool,
    pub name: String,
    pub ty: WindTypeRef,
    pub which: Option<Vec<WindWhichClauseRef>>,
    pub conditions: Option<Box<WindExprRef>>,
    pub default_value: Option<Box<WindExprRef>>,
}
