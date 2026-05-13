use super::ty::WindType;
use super::expr::WindExpr;
use super::which_clause::WindWhichClause;

#[derive(Debug, Clone, PartialEq)]
pub struct WindStructField {
    pub public: bool,
    pub is_static: bool,
    pub name: String,
    pub ty: WindType,
    pub which: Option<Vec<WindWhichClause>>,
    pub conditions: Option<WindExpr>,
    pub default_value: Option<Box<WindExpr>>,
}
