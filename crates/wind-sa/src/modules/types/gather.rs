use crate::{Bindings, MangledName, ScopeTree, SemanticError, WindValueID, WindValuePool};

pub struct GatherContext {
    pub scope_tree: ScopeTree,
    pub value_pool: WindValuePool,
    pub bindings: Bindings,
    pub errors: Vec<SemanticError>,
    pub dead_values: Vec<(MangledName, WindValueID)>,
    pub value_names: std::collections::HashMap<WindValueID, String>,
}
