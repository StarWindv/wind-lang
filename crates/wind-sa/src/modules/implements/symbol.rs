use crate::modules::types::Symbol;

impl Symbol {
    pub fn name(&self) -> &str {
        match self {
            Symbol::Variable { name, .. } => name,
            Symbol::Function { name, .. } => name,
            Symbol::Struct { name, .. } => name,
            Symbol::Trait { name, .. } => name,
            Symbol::Enum { name, .. } => name,
            Symbol::TypeAlias { name, .. } => name,
            Symbol::Extra { name, .. } => name.as_deref().unwrap_or("<anonymous>"),
            Symbol::Impl { trait_name, .. } => trait_name,
            Symbol::Group { name, .. } => name,
        }
    }
}
