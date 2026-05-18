use std::collections::HashMap;
use crate::modules::types::{Bindings, MangledName, WindValueID, WindValuePool};

impl Bindings {
    pub fn new() -> Self {
        Self {
            name_to_value: HashMap::new(),
            value_to_names: HashMap::new(),
        }
    }

    pub fn bind(&mut self, name: MangledName, value: WindValueID, pool: &mut WindValuePool) -> Option<WindValueID> {
        let mut dead_old = None;
        if let Some(&old_value) = self.name_to_value.get(&name) {
            if pool.dec_ref(old_value) {
                dead_old = Some(old_value);
            }
            self.value_to_names.get_mut(&old_value).map(|names| {
                names.retain(|n| n != &name);
            });
        }
        self.name_to_value.insert(name.clone(), value);
        pool.inc_ref(value);
        self.value_to_names
            .entry(value)
            .or_default()
            .push(name);
        dead_old
    }

    pub fn unbind(&mut self, name: &MangledName, pool: &mut WindValuePool) -> Option<WindValueID> {
        if let Some(&value) = self.name_to_value.get(name) {
            let dead = pool.dec_ref(value);
            self.name_to_value.remove(name);
            self.value_to_names.get_mut(&value).map(|names| {
                names.retain(|n| n != name);
            });
            if dead {
                return Some(value);
            }
            return Some(value);
        }
        None
    }

    pub fn lookup(&self, name: &MangledName) -> Option<WindValueID> {
        self.name_to_value.get(name).copied()
    }

    pub fn get_names_for_value(&self, value: WindValueID) -> &[MangledName] {
        self.value_to_names
            .get(&value)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn unbind_all_in_scope(&mut self, scope_mangled_names: &[MangledName], pool: &mut WindValuePool) -> Vec<(MangledName, WindValueID)> {
        let mut dead = Vec::new();
        for name in scope_mangled_names {
            if let Some(value) = self.name_to_value.remove(name) {
                let is_dead = pool.dec_ref(value);
                self.value_to_names.get_mut(&value).map(|names| {
                    names.retain(|n| n != name);
                });
                if is_dead {
                    dead.push((name.clone(), value));
                }
            }
        }
        dead
    }
}
