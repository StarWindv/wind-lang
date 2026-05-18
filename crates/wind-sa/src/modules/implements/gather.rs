use crate::modules::types::*;
use crate::modules::types::types::*;
use wind_frontend::ast_node::*;
use log::debug;

pub struct GatherContext {
    pub scope_tree: ScopeTree,
    pub value_pool: WindValuePool,
    pub bindings: Bindings,
    pub errors: Vec<SemanticError>,
    pub dead_values: Vec<(MangledName, WindValueID)>,
    pub value_names: std::collections::HashMap<WindValueID, String>,
}

impl GatherContext {
    pub fn new() -> Self {
        Self {
            scope_tree: ScopeTree::new(),
            value_pool: WindValuePool::new(),
            bindings: Bindings::new(),
            errors: Vec::new(),
            dead_values: Vec::new(),
            value_names: std::collections::HashMap::new(),
        }
    }

    pub fn gather(&mut self, program: &WindProgram) {
        debug!("[Gather] Starting symbol gathering pass");

        for stmt in &program.items {
            self.gather_top_level(stmt);
        }
    }

    fn gather_top_level(&mut self, stmt: &WindStmt) {
        match stmt {
            WindStmt::FnDef {
                public,
                name,
                params,
                return_type,
                which,
                body: _,
            } => self.gather_fn_def(*public, name, params, return_type, which),

            WindStmt::StructDef {
                public,
                name,
                fields,
            } => self.gather_struct_def(*public, name, fields),

            WindStmt::EnumDef {
                public,
                name,
                variants,
            } => self.gather_enum_def(*public, name, variants),

            WindStmt::TypeDef {
                public,
                name,
                base_type,
                conditions,
            } => self.gather_type_def(*public, name, base_type, conditions),

            WindStmt::TraitDef {
                public,
                name,
                functions,
            } => self.gather_trait_def(*public, name, functions),

            WindStmt::ExtraDef {
                public,
                name: extra_name,
                target,
                functions,
            } => self.gather_extra_def(*public, extra_name, target, functions),

            WindStmt::ImplDef {
                public,
                trait_name,
                target,
                functions,
            } => self.gather_impl_def(*public, trait_name, target, functions),

            WindStmt::GroupDef {
                public,
                name,
                target,
                params: _,
                rules,
            } => self.gather_group_def(*public, name, target, rules),

            WindStmt::ConstDef { name, ty, value: _ } => {
                self.gather_var_def(name, ty, StorageClass::Const);
            }

            WindStmt::ConstaticDef { name, ty, value: _ } => {
                self.gather_var_def(name, ty, StorageClass::Constatic);
            }

            WindStmt::ExplainDef { name, ty, value: _ } => {
                self.gather_var_def(name, ty, StorageClass::Explain);
            }

            WindStmt::Apply {
                group,
                target,
                fields: _,
            } => {
                debug!("[Gather] Apply: @{} -> {}", group, target);
            }

            WindStmt::Expr(expr) => {
                if let WindExpr::TagExpr { name, body } = expr.as_ref() {
                    self.gather_tag_expr(name, body);
                }
            }

            _ => {
                // Non-top-level statements are handled by the constraint checker later
            }
        }
    }

    fn gather_fn_def(
        &mut self,
        public: bool,
        name: &str,
        params: &[WindFnParam],
        return_type: &Option<WindType>,
        which: &Option<Vec<WindWhichClause>>,
    ) {
        let signature = FnSignatureInfo {
            id: WindFnSignatureId::new(1),
            public,
            name: name.to_string(),
            params: params
                .iter()
                .map(|p| {
                    let ty = p.ty.as_ref().map(WindTypeRef::from_ast).unwrap_or(WindTypeRef::Named("void".to_string()));
                    (p.name.clone(), ty)
                })
                .collect(),
            return_type: return_type.as_ref().map(WindTypeRef::from_ast),
            which: which.as_ref().map(|w| w.iter().map(WindWhichClauseRef::from_ast).collect()),
        };

        let scope_id = self.scope_tree.push_scope(ScopeKind::Function);

        let symbol = Symbol::Function {
            name: name.to_string(),
            public,
            signature: signature.id,
            which: signature.which.clone(),
            scope_id,
        };

        self.scope_tree
            .get_scope_mut(self.scope_tree.current)
            .unwrap()
            .symbols
            .insert(name.to_string(), symbol);

        self.scope_tree.pop_scope();
    }

    fn gather_struct_def(
        &mut self,
        public: bool,
        name: &str,
        fields: &[WindStructField],
    ) {
        let field_infos: Vec<FieldInfo> = fields
            .iter()
            .map(|f| FieldInfo {
                public: f.public,
                is_static: f.is_static,
                name: f.name.clone(),
                ty: WindTypeRef::from_ast(&f.ty),
                which: f.which.as_ref().map(|w| w.iter().map(WindWhichClauseRef::from_ast).collect()),
                conditions: f.conditions.as_ref().map(|_c| Box::new(WindExprRef)),
                default_value: f.default_value.as_ref().map(|_d| Box::new(WindExprRef)),
            })
            .collect();

        let symbol = Symbol::Struct {
            name: name.to_string(),
            public,
            fields: field_infos,
        };

        debug!("[Gather] Struct: {}", name);
        self.scope_tree.insert_symbol(name, symbol);
    }

    fn gather_enum_def(
        &mut self,
        public: bool,
        name: &str,
        variants: &[(String, Option<WindType>)],
    ) {
        let variant_list: Vec<(String, Option<WindTypeRef>)> = variants
            .iter()
            .map(|(vname, vty)| {
                (vname.clone(), vty.as_ref().map(WindTypeRef::from_ast))
            })
            .collect();

        let symbol = Symbol::Enum {
            name: name.to_string(),
            public,
            variants: variant_list,
        };

        debug!("[Gather] Enum: {}", name);
        self.scope_tree.insert_symbol(name, symbol);
    }

    fn gather_type_def(
        &mut self,
        public: bool,
        name: &str,
        base_type: &WindType,
        conditions: &[WindExpr],
    ) {
        let symbol = Symbol::TypeAlias {
            name: name.to_string(),
            public,
            base_type: WindTypeRef::from_ast(base_type),
            conditions: conditions.iter().map(|_| WindExprRef).collect(),
        };

        debug!("[Gather] TypeAlias: {}", name);
        self.scope_tree.insert_symbol(name, symbol);
    }

    fn gather_trait_def(
        &mut self,
        public: bool,
        name: &str,
        functions: &[WindFnSignature],
    ) {
        let mut method_ids = Vec::new();
        for sig in functions {
            let info = FnSignatureInfo {
                id: WindFnSignatureId::new(1),
                public: sig.public,
                name: sig.name.clone(),
                params: sig
                    .params
                    .iter()
                    .map(|p| {
                        let ty = p.ty.as_ref().map(WindTypeRef::from_ast).unwrap_or(WindTypeRef::Named("void".to_string()));
                        (p.name.clone(), ty)
                    })
                    .collect(),
                return_type: sig.return_type.as_ref().map(WindTypeRef::from_ast),
                which: sig.which.as_ref().map(|w| w.iter().map(WindWhichClauseRef::from_ast).collect()),
            };
            method_ids.push(info.id);
        }

        let symbol = Symbol::Trait {
            name: name.to_string(),
            public,
            methods: method_ids,
        };

        debug!("[Gather] Trait: {}", name);
        self.scope_tree.insert_symbol(name, symbol);
    }

    fn gather_extra_def(
        &mut self,
        public: bool,
        extra_name: &Option<String>,
        target: &str,
        functions: &[WindStmt],
    ) {
        let mut fn_ids = Vec::new();

        for stmt in functions {
            if let WindStmt::FnDef { name, .. } = stmt {
                let sig_id = WindFnSignatureId::new(1);
                fn_ids.push(sig_id);
                debug!("[Gather] Extra fn: {}", name);
            }
        }

        let symbol = Symbol::Extra {
            name: extra_name.clone(),
            target_struct: target.to_string(),
            functions: fn_ids,
        };

        let key = extra_name.as_deref().unwrap_or(target);
        self.scope_tree.insert_symbol(key, symbol);
        debug!(
            "[Gather] Extra for {} (public: {})",
            target, public
        );
    }

    fn gather_impl_def(
        &mut self,
        public: bool,
        trait_name: &str,
        target: &str,
        functions: &[WindStmt],
    ) {
        let mut fn_ids = Vec::new();

        for stmt in functions {
            if let WindStmt::FnDef { name, .. } = stmt {
                let sig_id = WindFnSignatureId::new(1);
                fn_ids.push(sig_id);
                debug!("[Gather] Impl fn: {}", name);
            }
        }

        let symbol = Symbol::Impl {
            trait_name: trait_name.to_string(),
            target_struct: target.to_string(),
            functions: fn_ids,
        };

        let key = format!("impl_{}_for_{}", trait_name, target);
        self.scope_tree.insert_symbol(&key, symbol);
        debug!(
            "[Gather] Impl {} for {} (public: {})",
            trait_name, target, public
        );
    }

    fn gather_group_def(
        &mut self,
        public: bool,
        name: &str,
        target: &Option<String>,
        rules: &[WindGroupRule],
    ) {
        let rule_infos: Vec<GroupRuleInfo> = rules
            .iter()
            .map(|r| match r {
                WindGroupRule::Simple { field, ty } => GroupRuleInfo {
                    kind: GroupRuleKind::Simple {
                        param: field.clone(),
                    },
                    ty: WindTypeRef::from_ast(ty),
                },
                WindGroupRule::SelfField { field, ty } => GroupRuleInfo {
                    kind: GroupRuleKind::SelfField {
                        field: field.clone(),
                    },
                    ty: WindTypeRef::from_ast(ty),
                },
            })
            .collect();

        let symbol = Symbol::Group {
            name: name.to_string(),
            public,
            target_struct: target.clone(),
            rules: rule_infos,
        };

        debug!("[Gather] Group: {}", name);
        self.scope_tree.insert_symbol(name, symbol);
    }

    fn gather_var_def(&mut self, name: &str, ty: &WindType, storage: StorageClass) {
        debug!("[Gather] Variable: {} ({:?})", name, storage);
        let mangled = MangledName::new(WindScopeId::new(1), "global", 1, name, 1);
        let type_ref = WindTypeRef::from_ast(ty);

        let symbol = Symbol::Variable {
            name: name.to_string(),
            mangled_name: mangled.clone(),
            ty: Some(type_ref),
            mutable: matches!(storage, StorageClass::Explain),
            storage_class: storage,
        };
        self.scope_tree.insert_symbol(name, symbol);
        self.scope_tree
            .current_scope_mut()
            .add_mangled_name(mangled);
    }

    fn gather_tag_expr(&mut self, name: &str, _body: &[WindStmt]) {
        let mangled = MangledName::new(WindScopeId::new(1), "global", 1, name, 1);

        let symbol = Symbol::Variable {
            name: name.to_string(),
            mangled_name: mangled.clone(),
            ty: Some(WindTypeRef::Named("tag".to_string())),
            mutable: false,
            storage_class: StorageClass::Let,
        };

        self.scope_tree.insert_symbol(name, symbol);
        self.scope_tree
            .current_scope_mut()
            .add_mangled_name(mangled);
    }

    #[allow(dead_code)]
    fn stmt_kind_name(&self, stmt: &WindStmt) -> &'static str {
        match stmt {
            WindStmt::Let { .. } => "Let",
            WindStmt::Assignment { .. } => "Assignment",
            WindStmt::Expr(_) => "Expression",
            WindStmt::Block(_) => "Block",
            WindStmt::If { .. } => "If",
            WindStmt::For { .. } => "For",
            WindStmt::ForIn { .. } => "ForIn",
            WindStmt::While { .. } => "While",
            WindStmt::Return(_) => "Return",
            WindStmt::FnDef { .. } => "FnDef",
            WindStmt::StructDef { .. } => "StructDef",
            WindStmt::EnumDef { .. } => "EnumDef",
            WindStmt::TypeDef { .. } => "TypeDef",
            WindStmt::ExtraDef { .. } => "ExtraDef",
            WindStmt::ImplDef { .. } => "ImplDef",
            WindStmt::TraitDef { .. } => "TraitDef",
            WindStmt::GroupDef { .. } => "GroupDef",
            WindStmt::ConstDef { .. } => "ConstDef",
            WindStmt::ConstaticDef { .. } => "ConstaticDef",
            WindStmt::Apply { .. } => "Apply",
            WindStmt::ExplainDef { .. } => "ExplainDef",
        }
    }
}
