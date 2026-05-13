use super::fn_param::WindFnParam;
use super::ty::WindType;
use super::which_clause::WindWhichClause;

#[derive(Debug, Clone, PartialEq)]
pub struct WindFnSignature {
    pub public: bool,
    pub name: String,
    pub params: Vec<WindFnParam>,
    pub return_type: Option<WindType>,
    pub which: Option<Vec<WindWhichClause>>,
}
