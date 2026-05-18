use crate::modules::types::{MangledName, WindScopeId};

impl MangledName {
    pub fn new(scope_id: WindScopeId, fn_name: &str, subscope_id: u64, var_name: &str, version: u64) -> Self {
        Self {
            scope_id,
            fn_name: fn_name.to_string(),
            subscope_id,
            var_name: var_name.to_string(),
            version,
        }
    }

    pub fn display(&self) -> String {
        if self.scope_id.get() == 1 {
            format!("1_global_{}", self.var_name)
        } else {
            format!(
                "{}_{}_{}_{}:{}",
                self.scope_id.get(),
                self.fn_name,
                self.subscope_id,
                self.var_name,
                self.version
            )
        }
    }
}

impl std::fmt::Display for MangledName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}
