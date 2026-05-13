use super::fn_param::FnParam;
use super::ty::Type;
use super::which_clause::WhichClause;

#[derive(Debug, Clone, PartialEq)]
pub struct FnSignature {
    pub public: bool,
    pub name: String,
    pub params: Vec<FnParam>,
    pub return_type: Option<Type>,
    pub which: Option<Vec<WhichClause>>,
}
