use std::collections::HashMap;
use crate::modules::types::{MangledName, Scope, ScopeKind, Symbol, WindScopeId};

impl Scope {
    pub fn new(id: WindScopeId, kind: ScopeKind, parent: Option<WindScopeId>) -> Self {
        Self {
            id,
            kind,
            parent,
            symbols: HashMap::new(),
            local_mangled_names: Vec::new(),
        }
    }

    pub fn insert_symbol(&mut self, name: &str, symbol: Symbol) {
        self.symbols.insert(name.to_string(), symbol);
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    pub fn add_mangled_name(&mut self, name: MangledName) {
        self.local_mangled_names.push(name);
    }
}
