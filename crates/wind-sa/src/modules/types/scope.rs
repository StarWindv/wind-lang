use std::collections::HashMap;
use crate::modules::types::{MangledName, ScopeKind, Symbol, WindScopeId};

#[derive(Debug, Clone)]
pub struct Scope {
    pub id: WindScopeId,
    pub kind: ScopeKind,
    pub parent: Option<WindScopeId>,
    pub symbols: HashMap<String, Symbol>,
    pub local_mangled_names: Vec<MangledName>,
}
