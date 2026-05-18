use crate::modules::types::WindFnSignatureId;
use crate::{WindTypeRef, WindWhichClauseRef};

#[derive(Debug, Clone)]
pub struct FnSignatureInfo {
    pub id: WindFnSignatureId,
    pub public: bool,
    pub name: String,
    pub params: Vec<(String, WindTypeRef)>,
    pub return_type: Option<WindTypeRef>,
    pub which: Option<Vec<WindWhichClauseRef>>,
}
