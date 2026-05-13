use chumsky::prelude::*;
use crate::modules::implements::tokens::SpannedToken;
use crate::modules::types::ast::*;
use crate::modules::types::tokens::Token;


fn select_token<'a>(tok: Token) -> impl Parser<'a, PInput<'a>, Token> + Clone {
    any().filter_map(move |(t, _): SpannedToken| if t == tok { Some(tok.clone()) } else { None })
}

fn identifier<'a>() -> impl Parser<'a, PInput<'a>, String> + Clone {
    any().filter_map(|(t, _): SpannedToken| if let Token::Identifier(s) = t { Some(s) } else { None })
}

fn int_literal<'a>() -> impl Parser<'a, PInput<'a>, Expr> + Clone {
    any().filter_map(|(t, _): SpannedToken| if let Token::IntLiteral(n) = t { Some(Expr::IntLiteral(n)) } else { None })
}

fn float_literal<'a>() -> impl Parser<'a, PInput<'a>, Expr> + Clone {
    any().filter_map(|(t, _): SpannedToken| {
        if let Token::FloatLiteral(s) = t { s.parse::<f64>().ok().map(Expr::FloatLiteral) } else { None }
    })
}

fn string_literal<'a>() -> impl Parser<'a, PInput<'a>, Expr> + Clone {
    any().filter_map(|(t, _): SpannedToken| if let Token::StringLiteral(s) = t { Some(Expr::StringLiteral(s)) } else { None })
}

fn char_literal<'a>() -> impl Parser<'a, PInput<'a>, Expr> + Clone {
    any().filter_map(|(t, _): SpannedToken| if let Token::CharLiteral(s) = t { Some(Expr::CharLiteral(s)) } else { None })
}

fn bool_literal<'a>() -> impl Parser<'a, PInput<'a>, Expr> + Clone {
    any().filter_map(|(t, _): SpannedToken| match &t {
        Token::TrueKw => Some(Expr::BoolLiteral(true)),
        Token::FalseKw => Some(Expr::BoolLiteral(false)),
        _ => None,
    })
}

fn none_literal<'a>() -> impl Parser<'a, PInput<'a>, Expr> + Clone {
    select_token(Token::NoneKw).to(Expr::NoneLiteral)
}

fn ident_expr<'a>() -> impl Parser<'a, PInput<'a>, Expr> + Clone {
    identifier().map(Expr::Identifier)
}

fn type_expr<'a>() -> impl Parser<'a, PInput<'a>, Type> + Clone {
    identifier().map(Type::Named)
}

fn type_annotation<'a>() -> impl Parser<'a, PInput<'a>, Type> + Clone {
    select_token(Token::Colon).ignore_then(type_expr())
}

fn fn_param<'a>() -> impl Parser<'a, PInput<'a>, FnParam> + Clone {
    identifier().then(type_annotation().or_not()).map(|(name, ty)| FnParam { name, ty })
}

fn assign_op<'a>() -> impl Parser<'a, PInput<'a>, AssignOp> + Clone {
    choice((
        select_token(Token::LeftAssign).to(AssignOp::LeftAbs),
        select_token(Token::RightAssign).to(AssignOp::RightAbs),
        select_token(Token::Equal).to(AssignOp::Direct),
    ))
}

fn expr_parser<'a>() -> impl Parser<'a, PInput<'a>, Expr> + Clone {
    recursive(|expr_rec| {
        let literal = choice((
            float_literal(), int_literal(), string_literal(),
            char_literal(), bool_literal(), none_literal(),
        ));

        let group = select_token(Token::OpenParen)
            .ignore_then(expr_rec.clone())
            .then_ignore(select_token(Token::CloseParen))
            .map(|e| Expr::Group(Box::new(e)));

        let atom = choice((literal, group, ident_expr()));

        let call_args = select_token(Token::OpenParen)
            .ignore_then(expr_rec.clone().separated_by(select_token(Token::Comma)).collect())
            .then_ignore(select_token(Token::CloseParen));

        let call_or_access = {
            let call = call_args
                .map(|args: Vec<Expr>| Box::new(move |e: Expr| Expr::Call { callee: Box::new(e), args: args.clone() }) as Box<dyn Fn(Expr) -> Expr>);

            let dot_access = select_token(Token::Dot).ignore_then(identifier())
                .map(|field| Box::new(move |e: Expr| Expr::FieldAccess { object: Box::new(e), field: field.clone() }) as Box<dyn Fn(Expr) -> Expr>);

            let scope_ref = select_token(Token::DoubleColon).ignore_then(identifier())
                .map(|member| Box::new(move |e: Expr| Expr::ScopeRef { object: Box::new(e), member: member.clone() }) as Box<dyn Fn(Expr) -> Expr>);

            let index_access = select_token(Token::OpenBracket).ignore_then(expr_rec.clone()).then_ignore(select_token(Token::CloseBracket))
                .map(|idx| Box::new(move |e: Expr| Expr::Index { expr: Box::new(e), index: Box::new(idx.clone()) }) as Box<dyn Fn(Expr) -> Expr>);

            atom.foldl(
                choice((call, dot_access, scope_ref, index_access)).repeated(),
                |base, apply: Box<dyn Fn(Expr) -> Expr>| apply(base),
            )
        };

        let unary = {
            let unary_op = any().filter_map(|(t, _): SpannedToken| UnaryOp::from_token(&t));
            unary_op.then(call_or_access.clone())
                .map(|(op, e)| Expr::Unary { op, expr: Box::new(e) })
                .or(call_or_access)
        };

        let bin_op = any().filter_map(|(t, _): SpannedToken| BinaryOp::from_token(&t));

        let map_literal = {
            let pair = expr_rec.clone().then_ignore(select_token(Token::Colon)).then(expr_rec.clone());
            select_token(Token::OpenBrace)
                .ignore_then(pair.separated_by(select_token(Token::Comma)).collect())
                .then_ignore(select_token(Token::CloseBrace))
                .map(Expr::MapLiteral)
        };

        let array_literal = select_token(Token::OpenBracket)
            .ignore_then(expr_rec.clone().separated_by(select_token(Token::Comma)).collect())
            .then_ignore(select_token(Token::CloseBracket))
            .map(Expr::ArrayLiteral);

        let primary = choice((map_literal, array_literal, unary));

        primary.clone().foldl(
            bin_op.then(primary).repeated(),
            |left, (op, right)| Expr::Binary { left: Box::new(left), op, right: Box::new(right) },
        )
    })
}

fn expr_or_type_expr<'a>() -> impl Parser<'a, PInput<'a>, Expr> + Clone {
    expr_parser().then(type_annotation().or_not())
        .map(|(e, ty)| if let Some(t) = ty { Expr::TypeExpr { expr: Box::new(e), ty: t } } else { e })
}

fn program_parser<'a>() -> impl Parser<'a, PInput<'a>, Program> + Clone {
    recursive(|stmt| {
        let block = select_token(Token::OpenBrace)
            .ignore_then(stmt.clone().repeated().collect())
            .then_ignore(select_token(Token::CloseBrace))
            .map(Stmt::Block);

        let let_stmt = identifier().then(type_annotation().or_not())
            .then(select_token(Token::Equal).ignore_then(expr_or_type_expr()))
            .then_ignore(select_token(Token::Semicolon))
            .map(|((name, ty), value)| Stmt::Let { name, ty, value: Box::new(value) });

        let assign_stmt = expr_parser().then(assign_op()).then(expr_or_type_expr())
            .then_ignore(select_token(Token::Semicolon))
            .map(|((target, op), value)| Stmt::Assignment { target: Box::new(target), op, value: Box::new(value) });

        let expr_stmt = expr_or_type_expr().then_ignore(select_token(Token::Semicolon))
            .map(|e| Stmt::Expr(Box::new(e)));

        let if_stmt = {
            let elif = select_token(Token::Elif).ignore_then(expr_parser()).then(block.clone())
                .map(|(cond, body)| (cond, body));
            select_token(Token::If).ignore_then(expr_parser()).then(block.clone())
                .then(elif.repeated().collect())
                .then(select_token(Token::Else).ignore_then(block.clone()).or_not())
                .map(|(((condition, then_branch), elif_branches), else_branch)| {
                    Stmt::If { condition: Box::new(condition), then_branch: Box::new(then_branch), elif_branches, else_branch: else_branch.map(Box::new) }
                })
        };

        let for_stmt = select_token(Token::For).ignore_then(select_token(Token::OpenParen))
            .ignore_then(expr_parser().or_not())
            .then(select_token(Token::Semicolon).ignore_then(expr_parser().or_not()))
            .then(select_token(Token::Semicolon).ignore_then(expr_parser().or_not()))
            .then_ignore(select_token(Token::CloseParen))
            .then(block.clone())
            .map(|(((init, cond), update), body)| {
                Stmt::For { init: init.map(Box::new), condition: cond.map(Box::new), update: update.map(Box::new), body: Box::new(body) }
            });

        let while_stmt = select_token(Token::While).ignore_then(expr_parser()).then(block.clone())
            .map(|(condition, body)| Stmt::While { condition: Box::new(condition), body: Box::new(body) });

        let return_stmt = select_token(Token::Return).ignore_then(expr_or_type_expr().or_not())
            .then_ignore(select_token(Token::Semicolon))
            .map(|e| Stmt::Return(e.map(Box::new)));

        let fn_def = select_token(Token::Fn).ignore_then(identifier())
            .then(
                select_token(Token::OpenParen)
                    .ignore_then(fn_param().separated_by(select_token(Token::Comma)).collect())
                    .then_ignore(select_token(Token::CloseParen)),
            )
            .then(select_token(Token::Arrow).ignore_then(type_expr()).or_not())
            .then(block.clone())
            .map(|(((name, params), return_type), body)| {
                Stmt::FnDef { name, params, return_type, body: Box::new(body) }
            });

        let struct_field = {
            let public = choice((select_token(Token::Pub), select_token(Token::Public)))
                .or_not().map(|p| p.is_some());
            let where_body = select_token(Token::Where)
                .ignore_then(select_token(Token::OpenBrace)
                    .ignore_then(stmt.clone().repeated().collect())
                    .then_ignore(select_token(Token::CloseBrace))
                );
            public.then(identifier())
                .then(select_token(Token::Colon).ignore_then(type_expr()))
                .then(where_body.or_not())
                .then_ignore(select_token(Token::Semicolon))
                .map(|(((public, name), ty), conditions)| {
                    let cond_expr = conditions.and_then(|stmts: Vec<Stmt>| {
                        stmts.into_iter().find_map(|s| if let Stmt::Expr(e) = s { Some(*e) } else { None })
                    });
                    StructField { public, name, ty, which: None, conditions: cond_expr }
                })
        };

        let struct_def = select_token(Token::Struct).ignore_then(identifier())
            .then(
                select_token(Token::OpenBrace)
                    .ignore_then(struct_field.repeated().collect())
                    .then_ignore(select_token(Token::CloseBrace)),
            )
            .map(|(name, fields)| Stmt::StructDef { name, fields });

        let enum_def = {
            let variant = identifier().then(type_annotation().or_not());
            select_token(Token::Enum).ignore_then(identifier())
                .then(
                    select_token(Token::OpenBrace)
                        .ignore_then(variant.separated_by(select_token(Token::Comma)).collect())
                        .then_ignore(select_token(Token::CloseBrace)),
                )
                .map(|(name, variants)| Stmt::EnumDef { name, variants })
        };

        let type_def = {
            let where_block = select_token(Token::Where)
                .ignore_then(select_token(Token::OpenBrace)
                    .ignore_then(stmt.clone().repeated().collect())
                    .then_ignore(select_token(Token::CloseBrace))
                )
                .map(|stmts: Vec<Stmt>| stmts.into_iter().filter_map(|s| if let Stmt::Expr(e) = s { Some(*e) } else { None }).collect::<Vec<_>>())
                .or_not().map(|c| c.unwrap_or_default());
            select_token(Token::Type).ignore_then(identifier())
                .then(select_token(Token::Equal).ignore_then(type_expr()))
                .then(where_block)
                .map(|((name, base_type), conditions)| Stmt::TypeDef { name, base_type, conditions })
        };

        let const_def = select_token(Token::Const).ignore_then(identifier())
            .then(select_token(Token::Equal).ignore_then(expr_or_type_expr()))
            .then_ignore(select_token(Token::Semicolon))
            .map(|(name, value)| Stmt::ConstDef { name, value: Box::new(value) });

        let constatic_def = select_token(Token::Constatic).ignore_then(identifier())
            .then(select_token(Token::Equal).ignore_then(expr_or_type_expr()))
            .then_ignore(select_token(Token::Semicolon))
            .map(|(name, value)| Stmt::ConstaticDef { name, value: Box::new(value) });

        let to_stmt = select_token(Token::To).ignore_then(identifier())
            .then_ignore(select_token(Token::Semicolon))
            .map(|tag| Stmt::ToStmt { tag });

        let trait_def = {
            let sig = choice((select_token(Token::Pub), select_token(Token::Public)))
                .or_not().map(|p| p.is_some())
                .then(select_token(Token::Fn).ignore_then(identifier()))
                .then(
                    select_token(Token::OpenParen)
                        .ignore_then(fn_param().separated_by(select_token(Token::Comma)).collect())
                        .then_ignore(select_token(Token::CloseParen)),
                )
                .then(select_token(Token::Arrow).ignore_then(type_expr()).or_not())
                .then_ignore(select_token(Token::Semicolon))
                .map(|(((public, name), params), return_type)| {
                    FnSignature { public, name, params, return_type, which: None }
                });
            select_token(Token::Trait).ignore_then(identifier())
                .then(
                    select_token(Token::OpenBrace)
                        .ignore_then(sig.repeated().collect())
                        .then_ignore(select_token(Token::CloseBrace)),
                )
                .map(|(name, functions)| Stmt::TraitDef { name, functions })
        };

        choice((
            fn_def, struct_def, enum_def, type_def, trait_def,
            const_def, constatic_def, to_stmt,
            if_stmt, for_stmt, while_stmt, return_stmt,
            let_stmt, assign_stmt, expr_stmt,
        ))
    })
        .repeated().collect().map(|items| Program { items }).then_ignore(end())
}

pub fn parse(tokens: &[SpannedToken]) -> Result<Program, Vec<ParseError>> {
    log::debug!("开始语法分析, 输入 {} 个标记", tokens.len());

    match program_parser().parse(tokens).into_result() {
        Ok(program) => {
            log::debug!("语法分析成功: {} 个顶层条目", program.items.len());
            Ok(program)
        }
        Err(errors) => {
            let parse_errors: Vec<ParseError> = errors
                .into_iter()
                .enumerate()
                .map(|(i, e)| {
                    let msg = format!("语法错误 #{i}: {e}");
                    log::error!("{msg}");
                    ParseError { message: msg, span: 0..0, found: None, expected: vec![] }
                })
                .collect();
            Err(parse_errors)
        }
    }
}
