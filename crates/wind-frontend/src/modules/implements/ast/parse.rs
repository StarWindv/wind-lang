use chumsky::IterParser;
use chumsky::Parser;
use chumsky::prelude::{any, choice, end, recursive};
use crate::modules::types::ast::{WindAssignOp, WindBinaryOp, WindExpr, WindFnParam, WindFnSignature, WindGroupRule, WindTokenSlice, WindParseError, WindProgram, WindStmt, WindStructField, WindType, WindUnaryOp, WindWhichClause, WindParser};
use crate::modules::types::tokens::{WindSpannedToken, WindToken};

impl WindParser {
    fn select_token<'a>(tok: WindToken) -> impl Parser<'a, WindTokenSlice<'a>, WindToken> + Clone {
        any().filter_map(move |(t, _): WindSpannedToken| if t == tok { Some(tok.clone()) } else { None })
    }

    fn identifier<'a>() -> impl Parser<'a, WindTokenSlice<'a>, String> + Clone {
        any().filter_map(|(t, _): WindSpannedToken| if let WindToken::Identifier(s) = t { Some(s) } else { None })
    }

    fn self_keyword<'a>() -> impl Parser<'a, WindTokenSlice<'a>, WindExpr> + Clone {
        any().filter_map(|(t, _): WindSpannedToken| {
            match t {
                WindToken::SelfKw => Some(WindExpr::Identifier("self".to_string())),
                WindToken::ThisKw => Some(WindExpr::Identifier("this".to_string())),
                WindToken::ItKw => Some(WindExpr::Identifier("it".to_string())),
                _ => None,
            }
        })
    }

    fn int_literal<'a>() -> impl Parser<'a, WindTokenSlice<'a>, WindExpr> + Clone {
        any().filter_map(|(t, _): WindSpannedToken| if let WindToken::IntLiteral(n) = t { Some(WindExpr::IntLiteral(n)) } else { None })
    }

    fn float_literal<'a>() -> impl Parser<'a, WindTokenSlice<'a>, WindExpr> + Clone {
        any().filter_map(|(t, _): WindSpannedToken| {
            if let WindToken::FloatLiteral(s) = t { s.parse::<f64>().ok().map(WindExpr::FloatLiteral) } else { None }
        })
    }

    fn string_literal<'a>() -> impl Parser<'a, WindTokenSlice<'a>, WindExpr> + Clone {
        any().filter_map(|(t, _): WindSpannedToken| if let WindToken::StringLiteral(s) = t { Some(WindExpr::StringLiteral(s)) } else { None })
    }

    fn char_literal<'a>() -> impl Parser<'a, WindTokenSlice<'a>, WindExpr> + Clone {
        any().filter_map(|(t, _): WindSpannedToken| if let WindToken::CharLiteral(s) = t { Some(WindExpr::CharLiteral(s)) } else { None })
    }

    fn bool_literal<'a>() -> impl Parser<'a, WindTokenSlice<'a>, WindExpr> + Clone {
        any().filter_map(|(t, _): WindSpannedToken| match &t {
            WindToken::TrueKw => Some(WindExpr::BoolLiteral(true)),
            WindToken::FalseKw => Some(WindExpr::BoolLiteral(false)),
            _ => None,
        })
    }

    fn none_literal<'a>() -> impl Parser<'a, WindTokenSlice<'a>, WindExpr> + Clone {
        Self::select_token(WindToken::NoneKw).to(WindExpr::NoneLiteral)
    }

    fn ident_expr<'a>() -> impl Parser<'a, WindTokenSlice<'a>, WindExpr> + Clone {
        Self::identifier().map(WindExpr::Identifier)
    }

    fn type_expr<'a>() -> impl Parser<'a, WindTokenSlice<'a>, WindType> + Clone {
        recursive(|type_rec| {
            let generic_args = Self::select_token(WindToken::Less)
                .ignore_then(type_rec.clone().separated_by(Self::select_token(WindToken::Comma)).collect())
                .then_ignore(Self::select_token(WindToken::Greater));

            Self::identifier()
                .then(generic_args.or_not())
                .map(|(base, args)| {
                    if let Some(args) = args {
                        WindType::Generic { base, args }
                    } else {
                        WindType::Named(base)
                    }
                })
        })
    }

    fn type_annotation<'a>() -> impl Parser<'a, WindTokenSlice<'a>, WindType> + Clone {
        Self::select_token(WindToken::Colon).ignore_then(Self::type_expr())
    }

    fn fn_param<'a>() -> impl Parser<'a, WindTokenSlice<'a>, WindFnParam> + Clone {
        Self::identifier().then(Self::type_annotation().or_not()).map(|(name, ty)| WindFnParam { name, ty })
    }

    fn assign_op<'a>() -> impl Parser<'a, WindTokenSlice<'a>, WindAssignOp> + Clone {
        choice((
            Self::select_token(WindToken::LeftAssign).to(WindAssignOp::LeftAbs),
            Self::select_token(WindToken::RightAssign).to(WindAssignOp::RightAbs),
            Self::select_token(WindToken::Equal).to(WindAssignOp::Direct),
        ))
    }

    fn which_clause_parser<'a>() -> impl Parser<'a, WindTokenSlice<'a>, WindWhichClause> + Clone {
        let method_ref = Self::select_token(WindToken::DoubleColon).ignore_then(Self::identifier())
            .map(|id| format!("::{id}"))
            .or(Self::identifier());

        Self::select_token(WindToken::Which)
            .ignore_then(method_ref.separated_by(Self::select_token(WindToken::Comma)).collect())
            .map(|after| WindWhichClause { method: String::new(), after })
    }

    fn make_fn_which_clause<'a>() -> impl Parser<'a, WindTokenSlice<'a>, Option<Vec<WindWhichClause>>> + Clone {
        Self::select_token(WindToken::Comma)
            .ignore_then(Self::which_clause_parser())
            .map(|w| vec![w])
            .or_not()
    }

    fn expr_parser<'a>() -> impl Parser<'a, WindTokenSlice<'a>, WindExpr> + Clone {
        recursive(|full_expr| {
            let literal = choice((
                Self::float_literal(), Self::int_literal(), Self::string_literal(),
                Self::char_literal(), Self::bool_literal(), Self::none_literal(),
            ));

            let group = Self::select_token(WindToken::OpenParen)
                .ignore_then(full_expr.clone())
                .then_ignore(Self::select_token(WindToken::CloseParen))
                .map(|e| WindExpr::Group(Box::new(e)));

            let if_expr = Self::select_token(WindToken::If)
                .ignore_then(full_expr.clone())
                .then(
                    Self::select_token(WindToken::OpenBrace)
                        .ignore_then(full_expr.clone())
                        .then_ignore(Self::select_token(WindToken::CloseBrace))
                )
                .then(
                    Self::select_token(WindToken::Else).ignore_then(
                        Self::select_token(WindToken::OpenBrace)
                            .ignore_then(full_expr.clone())
                            .then_ignore(Self::select_token(WindToken::CloseBrace))
                    ).or_not()
                )
                .map(|((condition, then_branch), else_branch)| {
                    WindExpr::IfExpr {
                        condition: Box::new(condition),
                        then_branch: Box::new(then_branch),
                        else_branch: else_branch.map(Box::new),
                    }
                });

            let atom = choice((literal, group, if_expr, Self::self_keyword(), Self::ident_expr()));

            let call_args = Self::select_token(WindToken::OpenParen)
                .ignore_then(full_expr.clone().separated_by(Self::select_token(WindToken::Comma)).collect())
                .then_ignore(Self::select_token(WindToken::CloseParen));

            let call_or_access = {
                let call = call_args
                    .map(|args: Vec<WindExpr>| Box::new(move |e: WindExpr| WindExpr::Call { callee: Box::new(e), args: args.clone() }) as Box<dyn Fn(WindExpr) -> WindExpr>);

                let dot_access = Self::select_token(WindToken::Dot).ignore_then(Self::identifier())
                    .map(|field| Box::new(move |e: WindExpr| WindExpr::FieldAccess { object: Box::new(e), field: field.clone() }) as Box<dyn Fn(WindExpr) -> WindExpr>);

                let scope_ref = Self::select_token(WindToken::DoubleColon).ignore_then(Self::identifier())
                    .map(|member| Box::new(move |e: WindExpr| WindExpr::ScopeRef { object: Box::new(e), member: member.clone() }) as Box<dyn Fn(WindExpr) -> WindExpr>);

                let index_access = Self::select_token(WindToken::OpenBracket).ignore_then(full_expr.clone()).then_ignore(Self::select_token(WindToken::CloseBracket))
                    .map(|idx| Box::new(move |e: WindExpr| WindExpr::Index { expr: Box::new(e), index: Box::new(idx.clone()) }) as Box<dyn Fn(WindExpr) -> WindExpr>);

                atom.foldl(
                    choice((call, dot_access, scope_ref, index_access)).repeated(),
                    |base, apply: Box<dyn Fn(WindExpr) -> WindExpr>| apply(base),
                )
            };

            let unary = {
                let unary_op = any().filter_map(|(t, _): WindSpannedToken| WindUnaryOp::from_token(&t));
                unary_op.then(call_or_access.clone())
                    .map(|(op, e)| WindExpr::Unary { op, expr: Box::new(e) })
                    .or(call_or_access)
            };

            let bin_op = any().filter_map(|(t, _): WindSpannedToken| WindBinaryOp::from_token(&t));

            let map_literal = {
                let pair = full_expr.clone().then_ignore(Self::select_token(WindToken::Colon)).then(full_expr.clone());
                Self::select_token(WindToken::OpenBrace)
                    .ignore_then(pair.separated_by(Self::select_token(WindToken::Comma)).collect())
                    .then_ignore(Self::select_token(WindToken::CloseBrace))
                    .map(WindExpr::MapLiteral)
            };

            let array_literal = Self::select_token(WindToken::OpenBracket)
                .ignore_then(full_expr.clone().separated_by(Self::select_token(WindToken::Comma)).collect())
                .then_ignore(Self::select_token(WindToken::CloseBracket))
                .map(WindExpr::ArrayLiteral);

            let primary = choice((map_literal, array_literal, unary));

            primary.clone().foldl(
                bin_op.then(primary).repeated(),
                |left, (op, right)| WindExpr::Binary { left: Box::new(left), op, right: Box::new(right) },
            )
        })
    }

    fn expr_or_type_expr<'a>() -> impl Parser<'a, WindTokenSlice<'a>, WindExpr> + Clone {
        Self::expr_parser().then(Self::type_annotation().or_not())
            .map(|(e, ty)| if let Some(t) = ty { WindExpr::TypeExpr { expr: Box::new(e), ty: t } } else { e })
    }

    fn visibility<'a>() -> impl Parser<'a, WindTokenSlice<'a>, bool> + Clone {
        choice((Self::select_token(WindToken::Pub), Self::select_token(WindToken::Public)))
            .or_not().map(|p| p.is_some())
    }

    fn program_parser<'a>() -> impl Parser<'a, WindTokenSlice<'a>, WindProgram> + Clone {
        recursive(|stmt| {
            let block = Self::select_token(WindToken::OpenBrace)
                .ignore_then(stmt.clone().repeated().collect())
                .then_ignore(Self::select_token(WindToken::CloseBrace))
                .map(WindStmt::Block);

            let let_stmt = Self::identifier().then(Self::type_annotation().or_not())
                .then(Self::select_token(WindToken::Equal).ignore_then(Self::expr_or_type_expr()))
                .then_ignore(Self::select_token(WindToken::Semicolon))
                .map(|((name, ty), value)| WindStmt::Let { name, ty, value: Box::new(value) });

            let assign_stmt = Self::expr_parser().then(Self::assign_op()).then(Self::expr_or_type_expr())
                .then_ignore(Self::select_token(WindToken::Semicolon))
                .map(|((target, op), value)| WindStmt::Assignment { target: Box::new(target), op, value: Box::new(value) });

            let expr_stmt = Self::expr_or_type_expr().then_ignore(Self::select_token(WindToken::Semicolon))
                .map(|e| WindStmt::Expr(Box::new(e)));

            let if_stmt = {
                let elif = Self::select_token(WindToken::Elif).ignore_then(Self::expr_parser()).then(block.clone())
                    .map(|(cond, body)| (cond, body));
                Self::select_token(WindToken::If).ignore_then(Self::expr_parser()).then(block.clone())
                    .then(elif.repeated().collect())
                    .then(Self::select_token(WindToken::Else).ignore_then(block.clone()).or_not())
                    .map(|(((condition, then_branch), elif_branches), else_branch)| {
                        WindStmt::If { condition: Box::new(condition), then_branch: Box::new(then_branch), elif_branches, else_branch: else_branch.map(Box::new) }
                    })
            };

            let for_stmt = Self::select_token(WindToken::For).ignore_then(Self::select_token(WindToken::OpenParen))
                .ignore_then(Self::expr_parser().or_not())
                .then(Self::select_token(WindToken::Semicolon).ignore_then(Self::expr_parser().or_not()))
                .then(Self::select_token(WindToken::Semicolon).ignore_then(Self::expr_parser().or_not()))
                .then_ignore(Self::select_token(WindToken::CloseParen))
                .then(block.clone())
                .map(|(((init, cond), update), body)| {
                    WindStmt::For { init: init.map(Box::new), condition: cond.map(Box::new), update: update.map(Box::new), body: Box::new(body) }
                });

            let while_stmt = Self::select_token(WindToken::While).ignore_then(Self::expr_parser()).then(block.clone())
                .map(|(condition, body)| WindStmt::While { condition: Box::new(condition), body: Box::new(body) });

            let return_stmt = Self::select_token(WindToken::Return).ignore_then(Self::expr_or_type_expr().or_not())
                .then_ignore(Self::select_token(WindToken::Semicolon))
                .map(|e| WindStmt::Return(e.map(Box::new)));

            let fn_def = Self::select_token(WindToken::Fn).ignore_then(Self::identifier())
                .then(
                    Self::select_token(WindToken::OpenParen)
                        .ignore_then(Self::fn_param().separated_by(Self::select_token(WindToken::Comma)).collect())
                        .then_ignore(Self::select_token(WindToken::CloseParen)),
                )
                    .then(Self::select_token(WindToken::Arrow).ignore_then(Self::type_expr()).or_not())
                    .then(Self::make_fn_which_clause())
                    .then(block.clone())
                    .map(|((((name, params), return_type), which), body)| {
                        WindStmt::FnDef { name, params, return_type, which, body: Box::new(body) }
                    });

            let struct_field = {
                let where_body = Self::select_token(WindToken::Where)
                    .ignore_then(Self::select_token(WindToken::OpenBrace)
                        .ignore_then(stmt.clone().repeated().collect())
                        .then_ignore(Self::select_token(WindToken::CloseBrace))
                    );
                let arrow_body = Self::select_token(WindToken::Arrow)
                    .ignore_then(Self::select_token(WindToken::OpenBrace)
                        .ignore_then(stmt.clone().repeated().collect())
                        .then_ignore(Self::select_token(WindToken::CloseBrace))
                    );
                let conditions = where_body.or(arrow_body).or_not();

                Self::visibility().then(Self::identifier())
                    .then(Self::select_token(WindToken::Colon).ignore_then(Self::type_expr()))
                    .then(conditions)
                    .then_ignore(Self::select_token(WindToken::Semicolon))
                    .map(|(((public, name), ty), conditions)| {
                        let cond_expr = conditions.and_then(|stmts: Vec<WindStmt>| {
                            stmts.into_iter().find_map(|s| if let WindStmt::Expr(e) = s { Some(*e) } else { None })
                        });
                        WindStructField { public, name, ty, which: None, conditions: cond_expr }
                    })
            };

            let struct_def = Self::select_token(WindToken::Struct).ignore_then(Self::identifier())
                .then(
                    Self::select_token(WindToken::OpenBrace)
                        .ignore_then(struct_field.repeated().collect())
                        .then_ignore(Self::select_token(WindToken::CloseBrace)),
                )
                .map(|(name, fields)| WindStmt::StructDef { name, fields });

            let enum_def = {
                let variant = Self::identifier().then(Self::type_annotation().or_not());
                Self::select_token(WindToken::Enum).ignore_then(Self::identifier())
                    .then(
                        Self::select_token(WindToken::OpenBrace)
                            .ignore_then(variant.separated_by(Self::select_token(WindToken::Comma)).collect())
                            .then_ignore(Self::select_token(WindToken::CloseBrace)),
                    )
                    .map(|(name, variants)| WindStmt::EnumDef { name, variants })
            };

            let type_def = {
                let where_block = Self::select_token(WindToken::Where)
                    .ignore_then(Self::select_token(WindToken::OpenBrace)
                        .ignore_then(stmt.clone().repeated().collect())
                        .then_ignore(Self::select_token(WindToken::CloseBrace))
                    )
                    .map(|stmts: Vec<WindStmt>| stmts.into_iter().filter_map(|s| if let WindStmt::Expr(e) = s { Some(*e) } else { None }).collect::<Vec<_>>())
                    .or_not().map(|c| c.unwrap_or_default());
                Self::select_token(WindToken::Type).ignore_then(Self::identifier())
                    .then(Self::select_token(WindToken::Equal).ignore_then(Self::type_expr()))
                    .then(where_block)
                    .map(|((name, base_type), conditions)| WindStmt::TypeDef { name, base_type, conditions })
            };

            let const_def = Self::select_token(WindToken::Const).ignore_then(Self::identifier())
                .then(Self::select_token(WindToken::Equal).ignore_then(Self::expr_or_type_expr()))
                .then_ignore(Self::select_token(WindToken::Semicolon))
                .map(|(name, value)| WindStmt::ConstDef { name, value: Box::new(value) });

            let constatic_def = Self::select_token(WindToken::Constatic).ignore_then(Self::identifier())
                .then(Self::select_token(WindToken::Equal).ignore_then(Self::expr_or_type_expr()))
                .then_ignore(Self::select_token(WindToken::Semicolon))
                .map(|(name, value)| WindStmt::ConstaticDef { name, value: Box::new(value) });

            let tag_def = {
                let tag_body = Self::select_token(WindToken::OpenBrace)
                    .ignore_then(stmt.clone().repeated().collect())
                    .then_ignore(Self::select_token(WindToken::CloseBrace));
                Self::select_token(WindToken::Tag)
                    .ignore_then(Self::identifier())
                    .then(Self::select_token(WindToken::Equal).ignore_then(tag_body))
                    .then_ignore(Self::select_token(WindToken::Semicolon))
                    .map(|(name, body)| {
                        WindStmt::Let {
                            name: name.clone(),
                            ty: None,
                            value: Box::new(WindExpr::TagExpr { name, body }),
                        }
                    })
            };

            let to_stmt = Self::select_token(WindToken::To).ignore_then(Self::identifier())
                .then_ignore(Self::select_token(WindToken::Semicolon))
                .map(|tag| WindStmt::ToStmt { tag });

            let trait_def = {
                let sig = Self::visibility()
                    .then(Self::select_token(WindToken::Fn).ignore_then(Self::identifier()))
                    .then(
                        Self::select_token(WindToken::OpenParen)
                            .ignore_then(Self::fn_param().separated_by(Self::select_token(WindToken::Comma)).collect())
                            .then_ignore(Self::select_token(WindToken::CloseParen)),
                    )
                    .then(Self::select_token(WindToken::Arrow).ignore_then(Self::type_expr()).or_not())
                    .then_ignore(Self::select_token(WindToken::Semicolon))
                    .map(|(((public, name), params), return_type)| {
                        WindFnSignature { public, name, params, return_type, which: None }
                    });
                Self::select_token(WindToken::Trait).ignore_then(Self::identifier())
                    .then(
                        Self::select_token(WindToken::OpenBrace)
                            .ignore_then(sig.repeated().collect())
                            .then_ignore(Self::select_token(WindToken::CloseBrace)),
                    )
                    .map(|(name, functions)| WindStmt::TraitDef { name, functions })
            };

            let extra_def = {
                let extra_fn = Self::visibility()
                    .then(Self::select_token(WindToken::Fn).ignore_then(Self::identifier()))
                    .then(
                        Self::select_token(WindToken::OpenParen)
                            .ignore_then(Self::fn_param().separated_by(Self::select_token(WindToken::Comma)).collect())
                            .then_ignore(Self::select_token(WindToken::CloseParen)),
                    )
                    .then(Self::select_token(WindToken::Arrow).ignore_then(Self::type_expr()).or_not())
                    .then(Self::make_fn_which_clause())
                    .then(block.clone())
                    .map(|(((((_public, name), params), return_type), which), body)| {
                        WindStmt::FnDef { name, params, return_type, which, body: Box::new(body) }
                    });

                let extra_target = Self::identifier()
                    .then(Self::select_token(WindToken::Colon).ignore_then(Self::identifier()).or_not())
                    .map(|(first, second)| {
                        if let Some(target) = second {
                            (Some(first), target)
                        } else {
                            (None, first)
                        }
                    });

                Self::select_token(WindToken::Extra)
                    .ignore_then(extra_target)
                    .then(
                        Self::select_token(WindToken::OpenBrace)
                            .ignore_then(extra_fn.repeated().collect())
                            .then_ignore(Self::select_token(WindToken::CloseBrace)),
                    )
                    .map(|((name, target), functions)| WindStmt::ExtraDef { name, target, functions })
            };

            let impl_def = {
                let impl_fn = Self::visibility()
                    .then(Self::select_token(WindToken::Fn).ignore_then(Self::identifier()))
                    .then(
                        Self::select_token(WindToken::OpenParen)
                            .ignore_then(Self::fn_param().separated_by(Self::select_token(WindToken::Comma)).collect())
                            .then_ignore(Self::select_token(WindToken::CloseParen)),
                    )
                    .then(Self::select_token(WindToken::Arrow).ignore_then(Self::type_expr()).or_not())
                    .then(Self::make_fn_which_clause())
                    .then(block.clone())
                    .map(|(((((_public, name), params), return_type), which), body)| {
                        WindStmt::FnDef { name, params, return_type, which, body: Box::new(body) }
                    });

                Self::select_token(WindToken::Impl)
                    .ignore_then(Self::identifier())
                    .then(Self::select_token(WindToken::For).ignore_then(Self::identifier()))
                    .then(
                        Self::select_token(WindToken::OpenBrace)
                            .ignore_then(impl_fn.repeated().collect())
                            .then_ignore(Self::select_token(WindToken::CloseBrace)),
                    )
                    .map(|((trait_name, target), functions)| WindStmt::ImplDef { trait_name, target, functions })
            };

            let group_def = {
                let group_rule = {
                    let self_field_rule = Self::select_token(WindToken::SelfKw)
                        .ignore_then(Self::select_token(WindToken::Dot).ignore_then(Self::identifier()))
                        .then(Self::select_token(WindToken::Arrow).ignore_then(Self::type_expr()))
                        .then_ignore(Self::select_token(WindToken::Semicolon))
                        .map(|(field, ty)| WindGroupRule::SelfField { field, ty });

                    let simple_rule = Self::identifier()
                        .then(Self::select_token(WindToken::Arrow).ignore_then(Self::type_expr()))
                        .then_ignore(Self::select_token(WindToken::Semicolon))
                        .map(|(field, ty)| WindGroupRule::Simple { field, ty });

                    choice((self_field_rule, simple_rule))
                };

                let group_params = Self::select_token(WindToken::OpenParen)
                    .ignore_then(Self::fn_param().separated_by(Self::select_token(WindToken::Comma)).collect())
                    .then_ignore(Self::select_token(WindToken::CloseParen));

                let group_header = Self::identifier()
                    .then(
                        Self::select_token(WindToken::Colon).ignore_then(Self::identifier())
                            .map(|t| (Some(t), None))
                            .or(group_params.map(|p| (None, Some(p))))
                            .or_not()
                    )
                    .map(|(name, opt)| {
                        let (target, params) = opt.unwrap_or((None, None));
                        (name, target, params)
                    });

                Self::select_token(WindToken::Group)
                    .ignore_then(group_header)
                    .then(
                        Self::select_token(WindToken::OpenBrace)
                            .ignore_then(group_rule.repeated().collect())
                            .then_ignore(Self::select_token(WindToken::CloseBrace)),
                    )
                    .map(|((name, target, params), rules)| WindStmt::GroupDef { name, target, params, rules })
            };

            let apply_stmt = {
                let fields = Self::select_token(WindToken::OpenBrace)
                    .ignore_then(Self::identifier().separated_by(Self::select_token(WindToken::Comma)).collect())
                    .then_ignore(Self::select_token(WindToken::CloseBrace));

                Self::identifier()
                    .then(Self::select_token(WindToken::At).ignore_then(Self::identifier()))
                    .then(Self::select_token(WindToken::Arrow).ignore_then(fields))
                    .then_ignore(Self::select_token(WindToken::Semicolon))
                    .map(|((group, target), fields)| WindStmt::Apply { group, target, fields })
            };

            choice((
                fn_def, struct_def, enum_def, type_def, trait_def,
                extra_def, impl_def, group_def, apply_stmt, tag_def,
                const_def, constatic_def, to_stmt,
                if_stmt, for_stmt, while_stmt, return_stmt,
                let_stmt, assign_stmt, expr_stmt,
            ))
        })
            .repeated().collect().map(|items| WindProgram { items }).then_ignore(end())
    }

    pub fn parse(tokens: &[WindSpannedToken]) -> Result<WindProgram, Vec<WindParseError>> {
        log::debug!("开始语法分析, 输入 {} 个标记", tokens.len());

        match Self::program_parser().parse(tokens).into_result() {
            Ok(program) => {
                log::debug!("语法分析成功: {} 个顶层条目", program.items.len());
                Ok(program)
            }
            Err(errors) => {
                let parse_errors: Vec<WindParseError> = errors
                    .into_iter()
                    .enumerate()
                    .map(|(i, e)| {
                        let msg = format!("语法错误 #{i}: {e}");
                        log::error!("{msg}");
                        WindParseError { message: msg, span: 0..0, found: None, expected: vec![] }
                    })
                    .collect();
                Err(parse_errors)
            }
        }
    }
}
