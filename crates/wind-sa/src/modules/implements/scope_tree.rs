use std::collections::HashMap;
use crate::modules::types::{Scope, ScopeKind, ScopeTree, Symbol, WindScopeId};

impl ScopeTree {
    pub fn new() -> Self {
        let global_id = WindScopeId::new(1);
        let global = Scope::new(global_id, ScopeKind::Global, None);
        let mut scopes = HashMap::new();
        scopes.insert(global_id, global);
        Self {
            scopes,
            current: global_id,
            next_id: 2,
        }
    }

    pub fn push_scope(&mut self, kind: ScopeKind) -> WindScopeId {
        let id = WindScopeId::new(self.next_id);
        self.next_id += 1;
        let parent = self.current;
        let scope = Scope::new(id, kind, Some(parent));
        self.scopes.insert(id, scope);
        self.current = id;
        id
    }

    pub fn pop_scope(&mut self) -> Option<WindScopeId> {
        let current = self.current;
        if let Some(scope) = self.scopes.get(&current) {
            let parent = scope.parent;
            if let Some(pid) = parent {
                self.current = pid;
            }
        }
        Some(current)
    }

    pub fn current_scope(&self) -> &Scope {
        &self.scopes[&self.current]
    }

    pub fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes.get_mut(&self.current).unwrap()
    }

    pub fn get_scope(&self, id: WindScopeId) -> Option<&Scope> {
        self.scopes.get(&id)
    }

    pub fn get_scope_mut(&mut self, id: WindScopeId) -> Option<&mut Scope> {
        self.scopes.get_mut(&id)
    }

    pub fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
        let mut current = Some(self.current);
        while let Some(cid) = current {
            if let Some(scope) = self.scopes.get(&cid) {
                if let Some(sym) = scope.lookup(name) {
                    return Some(sym);
                }
                current = scope.parent;
            } else {
                break;
            }
        }
        None
    }

    pub fn lookup_symbol_in_scope(&self, scope_id: WindScopeId, name: &str) -> Option<&Symbol> {
        self.scopes.get(&scope_id)?.lookup(name)
    }

    pub fn insert_symbol(&mut self, name: &str, symbol: Symbol) {
        self.current_scope_mut().insert_symbol(name, symbol);
    }
}
