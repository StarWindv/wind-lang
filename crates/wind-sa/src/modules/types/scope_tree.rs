use std::collections::HashMap;
use crate::modules::types::{Scope, WindScopeId};

#[derive(Debug)]
pub struct ScopeTree {
    pub scopes: HashMap<WindScopeId, Scope>,
    pub current: WindScopeId,
    pub(crate) next_id: u64,
}
