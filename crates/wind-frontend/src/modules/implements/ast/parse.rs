use chumsky::IterParser;
use chumsky::Parser;
use chumsky::prelude::{any, choice, end, recursive};
use crate::modules::types::ast::{WindAssignOp, WindBinaryOp, WindExpr, WindFnParam, WindFnSignature, WindTokenSlice, WindParseError, WindProgram, WindStmt, WindStructField, WindType, WindUnaryOp, WindParser};
use crate::modules::types::tokens::{WindSpannedToken, WindToken};


impl WindParser {
    fn select_token<'a>(tok: WindToken) -> impl Parser<'a, WindTokenSlice<'a>, WindToken> + Clone {
        any().filter_map(move |(t, _): WindSpannedToken| if t == tok { Some(tok.clone()) } else { None })
    }

    fn identifier<'a>() -> impl Parser<'a, WindTokenSlice<'a>, String> + Clone {
        any().filter_map(|(t, _): WindSpannedToken| if let WindToken::Identifier(s) = t { Some(s) } else { None })
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
        Self::identifier().map(WindType::Named)
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

    fn expr_parser<'a>() -> impl Parser<'a, WindTokenSlice<'a>, WindExpr> + Clone {
        recursive(|expr_rec| {
            let literal = choice((
                Self::float_literal(), Self::int_literal(), Self::string_literal(),
                Self::char_literal(), Self::bool_literal(), Self::none_literal(),
            ));

            let group = Self::select_token(WindToken::OpenParen)
                .ignore_then(expr_rec.clone())
                .then_ignore(Self::select_token(WindToken::CloseParen))
                .map(|e| WindExpr::Group(Box::new(e)));

            let atom = choice((literal, group, Self::ident_expr()));

            let call_args = Self::select_token(WindToken::OpenParen)
                .ignore_then(expr_rec.clone().separated_by(Self::select_token(WindToken::Comma)).collect())
                .then_ignore(Self::select_token(WindToken::CloseParen));

            let call_or_access = {
                let call = call_args
                    .map(|args: Vec<WindExpr>| Box::new(move |e: WindExpr| WindExpr::Call { callee: Box::new(e), args: args.clone() }) as Box<dyn Fn(WindExpr) -> WindExpr>);

                let dot_access = Self::select_token(WindToken::Dot).ignore_then(Self::identifier())
                    .map(|field| Box::new(move |e: WindExpr| WindExpr::FieldAccess { object: Box::new(e), field: field.clone() }) as Box<dyn Fn(WindExpr) -> WindExpr>);

                let scope_ref = Self::select_token(WindToken::DoubleColon).ignore_then(Self::identifier())
                    .map(|member| Box::new(move |e: WindExpr| WindExpr::ScopeRef { object: Box::new(e), member: member.clone() }) as Box<dyn Fn(WindExpr) -> WindExpr>);

                let index_access = Self::select_token(WindToken::OpenBracket).ignore_then(expr_rec.clone()).then_ignore(Self::select_token(WindToken::CloseBracket))
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
                let pair = expr_rec.clone().then_ignore(Self::select_token(WindToken::Colon)).then(expr_rec.clone());
                Self::select_token(WindToken::OpenBrace)
                    .ignore_then(pair.separated_by(Self::select_token(WindToken::Comma)).collect())
                    .then_ignore(Self::select_token(WindToken::CloseBrace))
                    .map(WindExpr::MapLiteral)
            };

            let array_literal = Self::select_token(WindToken::OpenBracket)
                .ignore_then(expr_rec.clone().separated_by(Self::select_token(WindToken::Comma)).collect())
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
                .then(block.clone())
                .map(|(((name, params), return_type), body)| {
                    WindStmt::FnDef { name, params, return_type, body: Box::new(body) }
                });

            let struct_field = {
                let public = choice((Self::select_token(WindToken::Pub), Self::select_token(WindToken::Public)))
                    .or_not().map(|p| p.is_some());
                let where_body = Self::select_token(WindToken::Where)
                    .ignore_then(Self::select_token(WindToken::OpenBrace)
                        .ignore_then(stmt.clone().repeated().collect())
                        .then_ignore(Self::select_token(WindToken::CloseBrace))
                    );
                public.then(Self::identifier())
                    .then(Self::select_token(WindToken::Colon).ignore_then(Self::type_expr()))
                    .then(where_body.or_not())
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

            let to_stmt = Self::select_token(WindToken::To).ignore_then(Self::identifier())
                .then_ignore(Self::select_token(WindToken::Semicolon))
                .map(|tag| WindStmt::ToStmt { tag });

            let trait_def = {
                let sig = choice((Self::select_token(WindToken::Pub), Self::select_token(WindToken::Public)))
                    .or_not().map(|p| p.is_some())
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

            choice((
                fn_def, struct_def, enum_def, type_def, trait_def,
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
