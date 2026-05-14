use std::collections::HashMap;
use std::num::NonZeroU64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueId(NonZeroU64);

impl ValueId {
    pub fn new(id: u64) -> Self {
        Self(NonZeroU64::new(id).expect("ValueId must be non-zero"))
    }

    pub fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(NonZeroU64);

impl ScopeId {
    pub fn new(id: u64) -> Self {
        Self(NonZeroU64::new(id).expect("ScopeId must be non-zero"))
    }

    pub fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(NonZeroU64);

impl TypeId {
    pub fn new(id: u64) -> Self {
        Self(NonZeroU64::new(id).expect("TypeId must be non-zero"))
    }

    pub fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FnSignatureId(NonZeroU64);

impl FnSignatureId {
    pub fn new(id: u64) -> Self {
        Self(NonZeroU64::new(id).expect("FnSignatureId must be non-zero"))
    }

    pub fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructId(NonZeroU64);

impl StructId {
    pub fn new(id: u64) -> Self {
        Self(NonZeroU64::new(id).expect("StructId must be non-zero"))
    }

    pub fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TraitId(NonZeroU64);

impl TraitId {
    pub fn new(id: u64) -> Self {
        Self(NonZeroU64::new(id).expect("TraitId must be non-zero"))
    }

    pub fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnumId(NonZeroU64);

impl EnumId {
    pub fn new(id: u64) -> Self {
        Self(NonZeroU64::new(id).expect("EnumId must be non-zero"))
    }

    pub fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConditionId(NonZeroU64);

impl ConditionId {
    pub fn new(id: u64) -> Self {
        Self(NonZeroU64::new(id).expect("ConditionId must be non-zero"))
    }

    pub fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MangledName {
    pub scope_id: ScopeId,
    pub fn_name: String,
    pub subscope_id: u64,
    pub var_name: String,
    pub version: u64,
}

impl MangledName {
    pub fn new(scope_id: ScopeId, fn_name: &str, subscope_id: u64, var_name: &str, version: u64) -> Self {
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

#[derive(Debug, Clone, PartialEq)]
pub enum ValueKind {
    ScalarConst,
    Allocated,
    Copied,
    Reference { target: ValueId },
    Returned { source: ValueId },
}

#[derive(Debug, Clone)]
pub struct ValueInfo {
    pub id: ValueId,
    pub ty: Option<WindResolvedType>,
    pub kind: ValueKind,
    pub ref_count: usize,
}

#[derive(Debug, Clone)]
pub struct ValuePool {
    pub values: HashMap<ValueId, ValueInfo>,
    pub scalar_cache: HashMap<String, ValueId>,
    next_id: u64,
}

impl ValuePool {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            scalar_cache: HashMap::new(),
            next_id: 1,
        }
    }

    fn allocate_id(&mut self) -> ValueId {
        let id = self.next_id;
        self.next_id += 1;
        ValueId::new(id)
    }

    pub fn new_allocated(&mut self, ty: Option<WindResolvedType>) -> ValueId {
        let id = self.allocate_id();
        self.values.insert(
            id,
            ValueInfo {
                id,
                ty,
                kind: ValueKind::Allocated,
                ref_count: 1,
            },
        );
        id
    }

    pub fn new_copy_of(&mut self, _source: ValueId, ty: Option<WindResolvedType>) -> ValueId {
        let id = self.allocate_id();
        self.values.insert(
            id,
            ValueInfo {
                id,
                ty,
                kind: ValueKind::Copied,
                ref_count: 1,
            },
        );
        id
    }

    pub fn new_reference(&mut self, target: ValueId, ty: Option<WindResolvedType>) -> ValueId {
        let id = self.allocate_id();
        self.values.insert(
            id,
            ValueInfo {
                id,
                ty,
                kind: ValueKind::Reference { target },
                ref_count: 0,
            },
        );
        if let Some(info) = self.values.get_mut(&target) {
            info.ref_count += 1;
        }
        id
    }

    pub fn scalar(&mut self, literal_repr: &str, ty: Option<WindResolvedType>) -> ValueId {
        if let Some(&cached) = self.scalar_cache.get(literal_repr) {
            if let Some(info) = self.values.get_mut(&cached) {
                info.ref_count += 1;
            }
            return cached;
        }
        let id = self.allocate_id();
        self.values.insert(
            id,
            ValueInfo {
                id,
                ty,
                kind: ValueKind::ScalarConst,
                ref_count: 1,
            },
        );
        self.scalar_cache.insert(literal_repr.to_string(), id);
        id
    }

    pub fn inc_ref(&mut self, id: ValueId) {
        if let Some(info) = self.values.get_mut(&id) {
            info.ref_count += 1;
        }
    }

    pub fn dec_ref(&mut self, id: ValueId) -> bool {
        if let Some(info) = self.values.get_mut(&id) {
            if info.ref_count > 0 {
                info.ref_count -= 1;
            }
            return info.ref_count == 0;
        }
        true
    }

    pub fn get(&self, id: ValueId) -> Option<&ValueInfo> {
        self.values.get(&id)
    }

    pub fn get_mut(&mut self, id: ValueId) -> Option<&mut ValueInfo> {
        self.values.get_mut(&id)
    }
}

#[derive(Debug)]
pub struct Bindings {
    pub name_to_value: HashMap<MangledName, ValueId>,
    pub value_to_names: HashMap<ValueId, Vec<MangledName>>,
}

impl Bindings {
    pub fn new() -> Self {
        Self {
            name_to_value: HashMap::new(),
            value_to_names: HashMap::new(),
        }
    }

    pub fn bind(&mut self, name: MangledName, value: ValueId, pool: &mut ValuePool) {
        if let Some(&old_value) = self.name_to_value.get(&name) {
            let _dead = pool.dec_ref(old_value);
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
    }

    pub fn unbind(&mut self, name: &MangledName, pool: &mut ValuePool) -> Option<ValueId> {
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

    pub fn lookup(&self, name: &MangledName) -> Option<ValueId> {
        self.name_to_value.get(name).copied()
    }

    pub fn get_names_for_value(&self, value: ValueId) -> &[MangledName] {
        self.value_to_names
            .get(&value)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn unbind_all_in_scope(&mut self, scope_mangled_names: &[MangledName], pool: &mut ValuePool) -> Vec<(MangledName, ValueId)> {
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

#[derive(Debug, Clone, PartialEq)]
pub enum StorageClass {
    Let,
    Const,
    Constatic,
    Explain,
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub public: bool,
    pub is_static: bool,
    pub name: String,
    pub ty: WindTypeRef,
    pub which: Option<Vec<WindWhichClauseRef>>,
    pub conditions: Option<Box<WindExprRef>>,
    pub default_value: Option<Box<WindExprRef>>,
}

#[derive(Debug, Clone)]
pub struct FnSignatureInfo {
    pub id: FnSignatureId,
    pub public: bool,
    pub name: String,
    pub params: Vec<(String, WindTypeRef)>,
    pub return_type: Option<WindTypeRef>,
    pub which: Option<Vec<WindWhichClauseRef>>,
}

#[derive(Debug, Clone)]
pub enum Symbol {
    Variable {
        name: String,
        mangled_name: MangledName,
        ty: Option<WindTypeRef>,
        mutable: bool,
        storage_class: StorageClass,
    },
    Function {
        name: String,
        public: bool,
        signature: FnSignatureId,
        which: Option<Vec<WindWhichClauseRef>>,
        scope_id: ScopeId,
    },
    Struct {
        name: String,
        public: bool,
        fields: Vec<FieldInfo>,
    },
    Trait {
        name: String,
        public: bool,
        methods: Vec<FnSignatureId>,
    },
    Enum {
        name: String,
        public: bool,
        variants: Vec<(String, Option<WindTypeRef>)>,
    },
    TypeAlias {
        name: String,
        public: bool,
        base_type: WindTypeRef,
        conditions: Vec<WindExprRef>,
    },
    Extra {
        name: Option<String>,
        target_struct: String,
        functions: Vec<FnSignatureId>,
    },
    Impl {
        trait_name: String,
        target_struct: String,
        functions: Vec<FnSignatureId>,
    },
    Group {
        name: String,
        public: bool,
        target_struct: Option<String>,
        rules: Vec<GroupRuleInfo>,
    },
}

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

#[derive(Debug, Clone)]
pub struct GroupRuleInfo {
    pub kind: GroupRuleKind,
    pub ty: WindTypeRef,
}

#[derive(Debug, Clone)]
pub enum GroupRuleKind {
    Simple { param: String },
    SelfField { field: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScopeKind {
    Global,
    Function,
    Block,
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub id: ScopeId,
    pub kind: ScopeKind,
    pub parent: Option<ScopeId>,
    pub symbols: HashMap<String, Symbol>,
    pub local_mangled_names: Vec<MangledName>,
}

impl Scope {
    pub fn new(id: ScopeId, kind: ScopeKind, parent: Option<ScopeId>) -> Self {
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

#[derive(Debug)]
pub struct ScopeTree {
    pub scopes: HashMap<ScopeId, Scope>,
    pub current: ScopeId,
    next_id: u64,
}

impl ScopeTree {
    pub fn new() -> Self {
        let global_id = ScopeId::new(1);
        let global = Scope::new(global_id, ScopeKind::Global, None);
        let mut scopes = HashMap::new();
        scopes.insert(global_id, global);
        Self {
            scopes,
            current: global_id,
            next_id: 2,
        }
    }

    pub fn push_scope(&mut self, kind: ScopeKind) -> ScopeId {
        let id = ScopeId::new(self.next_id);
        self.next_id += 1;
        let parent = self.current;
        let scope = Scope::new(id, kind, Some(parent));
        self.scopes.insert(id, scope);
        self.current = id;
        id
    }

    pub fn pop_scope(&mut self) -> Option<ScopeId> {
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

    pub fn get_scope(&self, id: ScopeId) -> Option<&Scope> {
        self.scopes.get(&id)
    }

    pub fn get_scope_mut(&mut self, id: ScopeId) -> Option<&mut Scope> {
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

    pub fn lookup_symbol_in_scope(&self, scope_id: ScopeId, name: &str) -> Option<&Symbol> {
        self.scopes.get(&scope_id)?.lookup(name)
    }

    pub fn insert_symbol(&mut self, name: &str, symbol: Symbol) {
        self.current_scope_mut().insert_symbol(name, symbol);
    }
}

#[derive(Debug, Clone)]
pub enum WindTypeRef {
    Named(String),
    Generic { base: String, args: Vec<WindTypeRef> },
    Fn { params: Vec<WindTypeRef>, ret: Box<WindTypeRef> },
    SelfType,
}

impl WindTypeRef {
    pub fn from_ast(ty: &wind_frontend::ast_node::WindType) -> Self {
        match ty {
            wind_frontend::ast_node::WindType::Named(n) => WindTypeRef::Named(n.clone()),
            wind_frontend::ast_node::WindType::Generic { base, args } => WindTypeRef::Generic {
                base: base.clone(),
                args: args.iter().map(Self::from_ast).collect(),
            },
            wind_frontend::ast_node::WindType::Fn { params, ret } => WindTypeRef::Fn {
                params: params.iter().map(Self::from_ast).collect(),
                ret: Box::new(Self::from_ast(ret)),
            },
            wind_frontend::ast_node::WindType::SelfType => WindTypeRef::SelfType,
        }
    }

    pub fn display_name(&self) -> String {
        match self {
            WindTypeRef::Named(n) => n.clone(),
            WindTypeRef::Generic { base, args } => {
                let args_str: Vec<String> = args.iter().map(|a| a.display_name()).collect();
                format!("{}<{}>", base, args_str.join(", "))
            }
            WindTypeRef::Fn { params, ret } => {
                let p: Vec<String> = params.iter().map(|a| a.display_name()).collect();
                format!("fn({}) -> {}", p.join(", "), ret.display_name())
            }
            WindTypeRef::SelfType => "Self".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WindExprRef;

#[derive(Debug, Clone)]
pub struct WindWhichClauseRef {
    pub method: String,
    pub after: Vec<String>,
}

impl WindWhichClauseRef {
    pub fn from_ast(clause: &wind_frontend::ast_node::WindWhichClause) -> Self {
        Self {
            method: clause.method.clone(),
            after: clause.after.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum WindResolvedType {
    Int,
    Float,
    String,
    Char,
    Bool,
    None,
    Byte,
    Tuple(Vec<WindResolvedType>),
    Vec(Box<WindResolvedType>),
    Map(Box<WindResolvedType>, Box<WindResolvedType>),
    Set(Box<WindResolvedType>),
    Struct(String),
    Enum(String),
    Tag,
    Function {
        params: Vec<WindResolvedType>,
        ret: Box<WindResolvedType>,
    },
    SelfType(String),
    Unknown,
    Error,
}

impl WindResolvedType {
    pub fn from_builtin_name(name: &str) -> Option<Self> {
        match name {
            "int" => Some(WindResolvedType::Int),
            "float" => Some(WindResolvedType::Float),
            "string" => Some(WindResolvedType::String),
            "char" => Some(WindResolvedType::Char),
            "bool" => Some(WindResolvedType::Bool),
            "None" => Some(WindResolvedType::None),
            "byte" => Some(WindResolvedType::Byte),
            "tag" => Some(WindResolvedType::Tag),
            _ => None,
        }
    }

    pub fn display_name(&self) -> String {
        match self {
            WindResolvedType::Int => "int".to_string(),
            WindResolvedType::Float => "float".to_string(),
            WindResolvedType::String => "string".to_string(),
            WindResolvedType::Char => "char".to_string(),
            WindResolvedType::Bool => "bool".to_string(),
            WindResolvedType::None => "None".to_string(),
            WindResolvedType::Byte => "byte".to_string(),
            WindResolvedType::Tuple(elems) => {
                let names: Vec<String> = elems.iter().map(|e| e.display_name()).collect();
                format!("({})", names.join(", "))
            }
            WindResolvedType::Vec(elem) => format!("vec<{}>", elem.display_name()),
            WindResolvedType::Map(k, v) => format!("map<{}, {}>", k.display_name(), v.display_name()),
            WindResolvedType::Set(elem) => format!("set<{}>", elem.display_name()),
            WindResolvedType::Struct(name) => name.clone(),
            WindResolvedType::Enum(name) => name.clone(),
            WindResolvedType::Tag => "tag".to_string(),
            WindResolvedType::Function { params, ret } => {
                let p: Vec<String> = params.iter().map(|t| t.display_name()).collect();
                format!("fn({}) -> {}", p.join(", "), ret.display_name())
            }
            WindResolvedType::SelfType(s) => s.clone(),
            WindResolvedType::Unknown => "<unknown>".to_string(),
            WindResolvedType::Error => "<error>".to_string(),
        }
    }

    pub fn is_builtin(&self) -> bool {
        matches!(
            self,
            WindResolvedType::Int
                | WindResolvedType::Float
                | WindResolvedType::String
                | WindResolvedType::Char
                | WindResolvedType::Bool
                | WindResolvedType::None
                | WindResolvedType::Byte
                | WindResolvedType::Tag
        )
    }

    pub fn is_container(&self) -> bool {
        matches!(
            self,
            WindResolvedType::Vec(_) | WindResolvedType::Map(_, _) | WindResolvedType::Set(_)
        )
    }

    pub fn is_value_type(&self) -> bool {
        matches!(
            self,
            WindResolvedType::Int
                | WindResolvedType::Float
                | WindResolvedType::Char
                | WindResolvedType::Bool
                | WindResolvedType::None
                | WindResolvedType::Byte
        )
    }
}

#[derive(Debug, Clone)]
pub struct LiveRange {
    pub value: ValueId,
    pub born_at: usize,
    pub last_use: usize,
    pub drop_at: Option<usize>,
    pub dropped_by_scope_exit: bool,
}

#[derive(Debug, Clone)]
pub struct DropPoint {
    pub value: ValueId,
    pub at_position: usize,
}

#[derive(Debug, Clone)]
pub struct SemanticError {
    pub message: String,
    pub span: Option<(usize, usize)>,
}

impl SemanticError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
        }
    }

    pub fn with_span(message: impl Into<String>, start: usize, end: usize) -> Self {
        Self {
            message: message.into(),
            span: Some((start, end)),
        }
    }
}
