use super::ty::Type;
use super::expr::Expr;
use super::which_clause::WhichClause;

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub public: bool,
    pub name: String,
    pub ty: Type,
    pub which: Option<Vec<WhichClause>>,
    pub conditions: Option<Expr>,
}
