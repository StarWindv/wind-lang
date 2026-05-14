use crate::modules::types::ast::{
    WindAssignOp, WindBinaryOp, WindExpr, WindFnParam, WindFnSignature, WindGroupRule,
    WindParseError, WindParser, WindProgram, WindStmt, WindStructField, WindType, WindUnaryOp,
    WindWhichClause,
};
use crate::modules::types::tokens::{WindSpan, WindSpannedToken, WindToken};
use chumsky::IterParser;
use chumsky::Parser;
use chumsky::error::Rich;
use chumsky::error::RichPattern;
use chumsky::extra;
use chumsky::input::{Input, Stream, ValueInput};
use chumsky::pratt;
use chumsky::prelude::{choice, end, just, recursive};
use crate::lexer::byte_to_line_col;

type WindRichErr<'a> = extra::Err<Rich<'a, WindToken, WindSpan>>;

impl WindParser {
    fn select_token<'a, I>(tok: WindToken) -> impl Parser<'a, I, WindToken, WindRichErr<'a>> + Clone
    where
        I: ValueInput<'a, Token = WindToken, Span = WindSpan>,
    {
        let t = tok.clone();
        just(tok).labelled(format!("{t:?}"))
    }

    fn identifier<'a, I>() -> impl Parser<'a, I, String, WindRichErr<'a>> + Clone
    where
        I: ValueInput<'a, Token = WindToken, Span = WindSpan>,
    {
        chumsky::select! {
            WindToken::Identifier(s) => s,
        }
        .labelled("identifier")
    }

    fn self_keyword<'a, I>() -> impl Parser<'a, I, WindExpr, WindRichErr<'a>> + Clone
    where
        I: ValueInput<'a, Token = WindToken, Span = WindSpan>,
    {
        chumsky::select! {
            WindToken::SelfKw => WindExpr::Identifier("self".to_string()),
            WindToken::ThisKw => WindExpr::Identifier("this".to_string()),
            WindToken::ItKw => WindExpr::Identifier("it".to_string()),
        }
        .labelled("self|this|it")
    }

    fn int_literal<'a, I>() -> impl Parser<'a, I, WindExpr, WindRichErr<'a>> + Clone
    where
        I: ValueInput<'a, Token = WindToken, Span = WindSpan>,
    {
        chumsky::select! {
            WindToken::IntLiteral(n) => WindExpr::IntLiteral(n),
        }
        .labelled("integer")
    }

    fn float_literal<'a, I>() -> impl Parser<'a, I, WindExpr, WindRichErr<'a>> + Clone
    where
        I: ValueInput<'a, Token = WindToken, Span = WindSpan>,
    {
        chumsky::select! {
            WindToken::FloatLiteral(s) => WindExpr::FloatLiteral(s.parse::<f64>().unwrap()),
        }
        .labelled("float")
    }

    fn string_literal<'a, I>() -> impl Parser<'a, I, WindExpr, WindRichErr<'a>> + Clone
    where
        I: ValueInput<'a, Token = WindToken, Span = WindSpan>,
    {
        chumsky::select! {
            WindToken::StringLiteral(s) => WindExpr::StringLiteral(s),
        }
        .labelled("string")
    }

    fn char_literal<'a, I>() -> impl Parser<'a, I, WindExpr, WindRichErr<'a>> + Clone
    where
        I: ValueInput<'a, Token = WindToken, Span = WindSpan>,
    {
        chumsky::select! {
            WindToken::CharLiteral(s) => WindExpr::CharLiteral(s),
        }
        .labelled("char")
    }

    fn bool_literal<'a, I>() -> impl Parser<'a, I, WindExpr, WindRichErr<'a>> + Clone
    where
        I: ValueInput<'a, Token = WindToken, Span = WindSpan>,
    {
        chumsky::select! {
            WindToken::TrueKw => WindExpr::BoolLiteral(true),
            WindToken::FalseKw => WindExpr::BoolLiteral(false),
        }
        .labelled("bool")
    }

    fn none_literal<'a, I>() -> impl Parser<'a, I, WindExpr, WindRichErr<'a>> + Clone
    where
        I: ValueInput<'a, Token = WindToken, Span = WindSpan>,
    {
        just(WindToken::NoneKw)
            .to(WindExpr::NoneLiteral)
            .labelled("None")
    }

    fn ident_expr<'a, I>() -> impl Parser<'a, I, WindExpr, WindRichErr<'a>> + Clone
    where
        I: ValueInput<'a, Token = WindToken, Span = WindSpan>,
    {
        Self::identifier().map(WindExpr::Identifier)
    }

    fn type_expr<'a, I>() -> impl Parser<'a, I, WindType, WindRichErr<'a>> + Clone
    where
        I: ValueInput<'a, Token = WindToken, Span = WindSpan>,
    {
        recursive(|type_rec| {
            let generic_args = Self::select_token(WindToken::Less)
                .ignore_then(
                    type_rec
                        .clone()
                        .separated_by(Self::select_token(WindToken::Comma))
                        .collect(),
                )
                .then_ignore(Self::select_token(WindToken::Greater));

            let self_type = Self::select_token(WindToken::SelfKw).to(WindType::SelfType);
            let none_type =
                Self::select_token(WindToken::NoneKw).to(WindType::Named("None".to_string()));

            choice((
                self_type,
                none_type,
                Self::identifier()
                    .then(generic_args.or_not())
                    .map(|(base, args)| {
                        if let Some(args) = args {
                            WindType::Generic { base, args }
                        } else {
                            WindType::Named(base)
                        }
                    }),
            ))
        })
    }

    fn type_annotation<'a, I>() -> impl Parser<'a, I, WindType, WindRichErr<'a>> + Clone
    where
        I: ValueInput<'a, Token = WindToken, Span = WindSpan>,
    {
        Self::select_token(WindToken::Colon).ignore_then(Self::type_expr())
    }

    fn fn_param<'a, I>() -> impl Parser<'a, I, WindFnParam, WindRichErr<'a>> + Clone
    where
        I: ValueInput<'a, Token = WindToken, Span = WindSpan>,
    {
        let self_param = Self::self_keyword().map(|e| {
            let name = if let WindExpr::Identifier(n) = e {
                n
            } else {
                String::new()
            };
            WindFnParam { name, ty: None }
        });
        let normal_param = Self::identifier()
            .then(Self::type_annotation().or_not())
            .map(|(name, ty)| WindFnParam { name, ty });

        choice((self_param, normal_param))
    }

    fn assign_op<'a, I>() -> impl Parser<'a, I, WindAssignOp, WindRichErr<'a>> + Clone
    where
        I: ValueInput<'a, Token = WindToken, Span = WindSpan>,
    {
        choice((
            Self::select_token(WindToken::LeftAssign).to(WindAssignOp::LeftAbs),
            Self::select_token(WindToken::RightAssign).to(WindAssignOp::RightAbs),
            Self::select_token(WindToken::PlusEqual).to(WindAssignOp::SumEq),
            Self::select_token(WindToken::MinusEqual).to(WindAssignOp::DiffEq),
            Self::select_token(WindToken::StarEqual).to(WindAssignOp::ProdEq),
            Self::select_token(WindToken::SlashEqual).to(WindAssignOp::QuotEq),
            Self::select_token(WindToken::Equal).to(WindAssignOp::Direct),
        ))
    }

    fn which_clause_parser<'a, I>() -> impl Parser<'a, I, WindWhichClause, WindRichErr<'a>> + Clone
    where
        I: ValueInput<'a, Token = WindToken, Span = WindSpan>,
    {
        let method_ref = Self::select_token(WindToken::DoubleColon)
            .ignore_then(Self::identifier())
            .map(|id| format!("::{id}"))
            .or(Self::identifier());

        Self::select_token(WindToken::Which)
            .ignore_then(
                method_ref
                    .separated_by(Self::select_token(WindToken::Comma))
                    .collect(),
            )
            .map(|after| WindWhichClause {
                method: String::new(),
                after,
            })
    }

    fn make_fn_which_clause<'a, I>()
    -> impl Parser<'a, I, Option<Vec<WindWhichClause>>, WindRichErr<'a>> + Clone
    where
        I: ValueInput<'a, Token = WindToken, Span = WindSpan>,
    {
        Self::select_token(WindToken::Comma)
            .ignore_then(Self::which_clause_parser())
            .map(|w| vec![w])
            .or_not()
    }

    fn expr_parser<'a, I>() -> impl Parser<'a, I, WindExpr, WindRichErr<'a>> + Clone
    where
        I: ValueInput<'a, Token = WindToken, Span = WindSpan>,
    {
        recursive(|full_expr| {
            let literal = choice((
                Self::float_literal(),
                Self::int_literal(),
                Self::string_literal(),
                Self::char_literal(),
                Self::bool_literal(),
                Self::none_literal(),
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
                        .then_ignore(Self::select_token(WindToken::CloseBrace)),
                )
                .then(
                    Self::select_token(WindToken::Else)
                        .ignore_then(
                            Self::select_token(WindToken::OpenBrace)
                                .ignore_then(full_expr.clone())
                                .then_ignore(Self::select_token(WindToken::CloseBrace)),
                        )
                        .or_not(),
                )
                .map(|((condition, then_branch), else_branch)| WindExpr::IfExpr {
                    condition: Box::new(condition),
                    then_branch: Box::new(then_branch),
                    else_branch: else_branch.map(Box::new),
                });

            let map_literal = {
                let pair = full_expr
                    .clone()
                    .then_ignore(Self::select_token(WindToken::Colon))
                    .then(full_expr.clone());
                Self::select_token(WindToken::OpenBrace)
                    .ignore_then(
                        pair.separated_by(Self::select_token(WindToken::Comma))
                            .allow_trailing()
                            .collect(),
                    )
                    .then_ignore(Self::select_token(WindToken::CloseBrace))
                    .map(WindExpr::MapLiteral)
            };

            let array_literal = Self::select_token(WindToken::OpenBracket)
                .ignore_then(
                    full_expr
                        .clone()
                        .separated_by(Self::select_token(WindToken::Comma))
                        .allow_trailing()
                        .collect(),
                )
                .then_ignore(Self::select_token(WindToken::CloseBracket))
                .map(WindExpr::ArrayLiteral);

            let struct_literal = {
                let explicit_field = Self::identifier()
                    .then_ignore(Self::select_token(WindToken::Colon))
                    .then(full_expr.clone())
                    .map(|(name, val)| (name, val));
                let shorthand_or_expr = full_expr.clone().map(|expr| {
                    if let WindExpr::Identifier(ref name) = expr {
                        (name.clone(), expr)
                    } else {
                        (String::new(), expr)
                    }
                });
                let field = choice((explicit_field, shorthand_or_expr));
                Self::ident_expr()
                    .then(
                        Self::select_token(WindToken::OpenBrace)
                            .ignore_then(
                                field
                                    .separated_by(Self::select_token(WindToken::Comma))
                                    .allow_trailing()
                                    .collect(),
                            )
                            .then_ignore(Self::select_token(WindToken::CloseBrace)),
                    )
                    .map(|(name, fields)| {
                        if let WindExpr::Identifier(n) = name {
                            WindExpr::StructLiteral { name: n, fields }
                        } else {
                            WindExpr::MapLiteral(
                                fields
                                    .into_iter()
                                    .map(|(k, v)| (WindExpr::Identifier(k), v))
                                    .collect(),
                            )
                        }
                    })
            };

            let atom = choice((
                literal,
                group,
                if_expr,
                map_literal,
                array_literal,
                struct_literal,
                Self::self_keyword(),
                Self::ident_expr(),
            ));

            let call_args = Self::select_token(WindToken::OpenParen)
                .ignore_then(
                    full_expr
                        .clone()
                        .separated_by(Self::select_token(WindToken::Comma))
                        .allow_trailing()
                        .collect(),
                )
                .then_ignore(Self::select_token(WindToken::CloseParen));

            let postfix_pref_infix_lo = (
                pratt::postfix(12, call_args, |lhs, args, _span| WindExpr::Call {
                    callee: Box::new(lhs),
                    args,
                }),
                pratt::postfix(
                    12,
                    Self::select_token(WindToken::Dot).ignore_then(Self::identifier()),
                    |lhs, field, _span| WindExpr::FieldAccess {
                        object: Box::new(lhs),
                        field,
                    },
                ),
                pratt::postfix(
                    12,
                    Self::select_token(WindToken::DoubleColon).ignore_then(Self::identifier()),
                    |lhs, member, _span| WindExpr::ScopeRef {
                        object: Box::new(lhs),
                        member,
                    },
                ),
                pratt::postfix(
                    12,
                    Self::select_token(WindToken::OpenBracket)
                        .ignore_then(full_expr.clone())
                        .then_ignore(Self::select_token(WindToken::CloseBracket)),
                    |lhs, idx, _span| WindExpr::Index {
                        expr: Box::new(lhs),
                        index: Box::new(idx),
                    },
                ),
                pratt::postfix(
                    13,
                    Self::select_token(WindToken::PlusPlus),
                    |lhs, _op, _span| WindExpr::Unary {
                        op: WindUnaryOp::IncPost,
                        expr: Box::new(lhs),
                    },
                ),
                pratt::postfix(
                    13,
                    Self::select_token(WindToken::MinusMinus),
                    |lhs, _op, _span| WindExpr::Unary {
                        op: WindUnaryOp::DecPost,
                        expr: Box::new(lhs),
                    },
                ),
                pratt::infix(
                    pratt::left(10),
                    Self::select_token(WindToken::Star).to(WindBinaryOp::Mul),
                    |l, op, r, _| WindExpr::Binary {
                        left: Box::new(l),
                        op,
                        right: Box::new(r),
                    },
                ),
                pratt::infix(
                    pratt::left(10),
                    Self::select_token(WindToken::Slash).to(WindBinaryOp::Div),
                    |l, op, r, _| WindExpr::Binary {
                        left: Box::new(l),
                        op,
                        right: Box::new(r),
                    },
                ),
                pratt::infix(
                    pratt::left(10),
                    Self::select_token(WindToken::DoubleSlash).to(WindBinaryOp::IntDiv),
                    |l, op, r, _| WindExpr::Binary {
                        left: Box::new(l),
                        op,
                        right: Box::new(r),
                    },
                ),
                pratt::infix(
                    pratt::left(10),
                    Self::select_token(WindToken::Percent).to(WindBinaryOp::Mod),
                    |l, op, r, _| WindExpr::Binary {
                        left: Box::new(l),
                        op,
                        right: Box::new(r),
                    },
                ),
                pratt::infix(
                    pratt::left(9),
                    Self::select_token(WindToken::Plus).to(WindBinaryOp::Add),
                    |l, op, r, _| WindExpr::Binary {
                        left: Box::new(l),
                        op,
                        right: Box::new(r),
                    },
                ),
                pratt::infix(
                    pratt::left(9),
                    Self::select_token(WindToken::Minus).to(WindBinaryOp::Sub),
                    |l, op, r, _| WindExpr::Binary {
                        left: Box::new(l),
                        op,
                        right: Box::new(r),
                    },
                ),
            );

            let prefix_lo = (
                pratt::prefix(
                    11,
                    Self::select_token(WindToken::PlusPlus).to(WindUnaryOp::Inc),
                    |_op, rhs, _span| WindExpr::Unary {
                        op: WindUnaryOp::Inc,
                        expr: Box::new(rhs),
                    },
                ),
                pratt::prefix(
                    11,
                    Self::select_token(WindToken::MinusMinus).to(WindUnaryOp::Dec),
                    |_op, rhs, _span| WindExpr::Unary {
                        op: WindUnaryOp::Dec,
                        expr: Box::new(rhs),
                    },
                ),
                pratt::prefix(
                    11,
                    Self::select_token(WindToken::Minus).to(WindUnaryOp::Neg),
                    |_op, rhs, _span| WindExpr::Unary {
                        op: WindUnaryOp::Neg,
                        expr: Box::new(rhs),
                    },
                ),
                pratt::prefix(
                    11,
                    Self::select_token(WindToken::Bang).to(WindUnaryOp::Not),
                    |_op, rhs, _span| WindExpr::Unary {
                        op: WindUnaryOp::Not,
                        expr: Box::new(rhs),
                    },
                ),
                pratt::prefix(
                    11,
                    Self::select_token(WindToken::DotDot),
                    |_op, rhs, _span| WindExpr::Unpack(Box::new(rhs)),
                ),
            );

            let infix_lo = (
                pratt::infix(
                    pratt::left(8),
                    Self::select_token(WindToken::LeftShift).to(WindBinaryOp::Shl),
                    |l, op, r, _| WindExpr::Binary {
                        left: Box::new(l),
                        op,
                        right: Box::new(r),
                    },
                ),
                pratt::infix(
                    pratt::left(8),
                    Self::select_token(WindToken::RightShift).to(WindBinaryOp::Shr),
                    |l, op, r, _| WindExpr::Binary {
                        left: Box::new(l),
                        op,
                        right: Box::new(r),
                    },
                ),
                pratt::infix(
                    pratt::left(7),
                    Self::select_token(WindToken::Less).to(WindBinaryOp::Lt),
                    |l, op, r, _| WindExpr::Binary {
                        left: Box::new(l),
                        op,
                        right: Box::new(r),
                    },
                ),
                pratt::infix(
                    pratt::left(7),
                    Self::select_token(WindToken::Greater).to(WindBinaryOp::Gt),
                    |l, op, r, _| WindExpr::Binary {
                        left: Box::new(l),
                        op,
                        right: Box::new(r),
                    },
                ),
                pratt::infix(
                    pratt::left(7),
                    Self::select_token(WindToken::LessEqual).to(WindBinaryOp::Le),
                    |l, op, r, _| WindExpr::Binary {
                        left: Box::new(l),
                        op,
                        right: Box::new(r),
                    },
                ),
                pratt::infix(
                    pratt::left(7),
                    Self::select_token(WindToken::GreaterEqual).to(WindBinaryOp::Ge),
                    |l, op, r, _| WindExpr::Binary {
                        left: Box::new(l),
                        op,
                        right: Box::new(r),
                    },
                ),
                pratt::infix(
                    pratt::left(7),
                    Self::select_token(WindToken::NotLess).to(WindBinaryOp::NotLt),
                    |l, op, r, _| WindExpr::Binary {
                        left: Box::new(l),
                        op,
                        right: Box::new(r),
                    },
                ),
                pratt::infix(
                    pratt::left(7),
                    Self::select_token(WindToken::NotGreater).to(WindBinaryOp::NotGt),
                    |l, op, r, _| WindExpr::Binary {
                        left: Box::new(l),
                        op,
                        right: Box::new(r),
                    },
                ),
                pratt::infix(
                    pratt::left(7),
                    Self::select_token(WindToken::In).to(WindBinaryOp::In),
                    |l, op, r, _| WindExpr::Binary {
                        left: Box::new(l),
                        op,
                        right: Box::new(r),
                    },
                ),
                pratt::infix(
                    pratt::left(6),
                    Self::select_token(WindToken::EqualEqual).to(WindBinaryOp::Eq),
                    |l, op, r, _| WindExpr::Binary {
                        left: Box::new(l),
                        op,
                        right: Box::new(r),
                    },
                ),
                pratt::infix(
                    pratt::left(6),
                    Self::select_token(WindToken::AddrEqual).to(WindBinaryOp::AddrEq),
                    |l, op, r, _| WindExpr::Binary {
                        left: Box::new(l),
                        op,
                        right: Box::new(r),
                    },
                ),
                pratt::infix(
                    pratt::left(6),
                    Self::select_token(WindToken::NotEqual).to(WindBinaryOp::Neq),
                    |l, op, r, _| WindExpr::Binary {
                        left: Box::new(l),
                        op,
                        right: Box::new(r),
                    },
                ),
                pratt::infix(
                    pratt::left(5),
                    Self::select_token(WindToken::Amp).to(WindBinaryOp::BitAnd),
                    |l, op, r, _| WindExpr::Binary {
                        left: Box::new(l),
                        op,
                        right: Box::new(r),
                    },
                ),
                pratt::infix(
                    pratt::left(4),
                    Self::select_token(WindToken::Caret).to(WindBinaryOp::BitXor),
                    |l, op, r, _| WindExpr::Binary {
                        left: Box::new(l),
                        op,
                        right: Box::new(r),
                    },
                ),
                pratt::infix(
                    pratt::left(3),
                    Self::select_token(WindToken::Pipe).to(WindBinaryOp::BitOr),
                    |l, op, r, _| WindExpr::Binary {
                        left: Box::new(l),
                        op,
                        right: Box::new(r),
                    },
                ),
            );

            let infix_hi = (
                pratt::infix(
                    pratt::left(2),
                    Self::select_token(WindToken::AndAnd).to(WindBinaryOp::And),
                    |l, op, r, _| WindExpr::Binary {
                        left: Box::new(l),
                        op,
                        right: Box::new(r),
                    },
                ),
                pratt::infix(
                    pratt::left(1),
                    Self::select_token(WindToken::OrOr).to(WindBinaryOp::Or),
                    |l, op, r, _| WindExpr::Binary {
                        left: Box::new(l),
                        op,
                        right: Box::new(r),
                    },
                ),
            );

            atom.pratt(postfix_pref_infix_lo)
                .pratt(prefix_lo)
                .pratt(infix_lo)
                .pratt(infix_hi)
        })
    }

    fn expr_or_type_expr<'a, I>() -> impl Parser<'a, I, WindExpr, WindRichErr<'a>> + Clone
    where
        I: ValueInput<'a, Token = WindToken, Span = WindSpan>,
    {
        Self::expr_parser()
            .then(Self::type_annotation().or_not())
            .map(|(e, ty)| {
                if let Some(t) = ty {
                    WindExpr::TypeExpr {
                        expr: Box::new(e),
                        ty: t,
                    }
                } else {
                    e
                }
            })
    }

    fn visibility<'a, I>() -> impl Parser<'a, I, bool, WindRichErr<'a>> + Clone
    where
        I: ValueInput<'a, Token = WindToken, Span = WindSpan>,
    {
        choice((
            Self::select_token(WindToken::Pub),
            Self::select_token(WindToken::Public),
        ))
        .or_not()
        .map(|p| p.is_some())
    }

    fn program_parser<'a, I>() -> impl Parser<'a, I, WindProgram, WindRichErr<'a>> + Clone
    where
        I: ValueInput<'a, Token = WindToken, Span = WindSpan>,
    {
        recursive(|stmt| {
            let block = Self::select_token(WindToken::OpenBrace)
                .ignore_then(stmt.clone().repeated().collect())
                .then_ignore(Self::select_token(WindToken::CloseBrace))
                .map(WindStmt::Block);

            let let_stmt = Self::identifier()
                .then(Self::type_annotation().or_not())
                .then(Self::select_token(WindToken::Equal).ignore_then(Self::expr_or_type_expr()))
                .then_ignore(Self::select_token(WindToken::Semicolon).or_not())
                .map(|((name, ty), value)| WindStmt::Let {
                    name,
                    ty,
                    value: Box::new(value),
                });

            let assign_stmt = Self::expr_parser()
                .then(Self::assign_op())
                .then(Self::expr_or_type_expr())
                .then_ignore(Self::select_token(WindToken::Semicolon).or_not())
                .map(|((target, op), value)| WindStmt::Assignment {
                    target: Box::new(target),
                    op,
                    value: Box::new(value),
                });

            let expr_stmt = Self::expr_or_type_expr()
                .then_ignore(Self::select_token(WindToken::Semicolon).or_not())
                .map(|e| WindStmt::Expr(Box::new(e)));

            let if_stmt = {
                let elif = Self::select_token(WindToken::Elif)
                    .ignore_then(Self::expr_parser())
                    .then(block.clone())
                    .map(|(cond, body)| (cond, body));
                Self::select_token(WindToken::If)
                    .ignore_then(Self::expr_parser())
                    .then(block.clone())
                    .then(elif.repeated().collect())
                    .then(
                        Self::select_token(WindToken::Else)
                            .ignore_then(block.clone())
                            .or_not(),
                    )
                    .map(
                        |(((condition, then_branch), elif_branches), else_branch)| WindStmt::If {
                            condition: Box::new(condition),
                            then_branch: Box::new(then_branch),
                            elif_branches,
                            else_branch: else_branch.map(Box::new),
                        },
                    )
            };

            let for_stmt = {
                let for_head = Self::select_token(WindToken::For)
                    .ignore_then(Self::select_token(WindToken::OpenParen));

                let c_style = Self::identifier()
                    .then(Self::type_annotation().or_not())
                    .then(
                        Self::select_token(WindToken::Equal)
                            .ignore_then(Self::expr_or_type_expr())
                            .or_not(),
                    )
                    .then(
                        Self::select_token(WindToken::Semicolon)
                            .ignore_then(Self::expr_parser().or_not()),
                    )
                    .then(
                        Self::select_token(WindToken::Semicolon)
                            .ignore_then(Self::expr_parser().or_not()),
                    )
                    .then_ignore(Self::select_token(WindToken::CloseParen))
                    .then(block.clone())
                    .map(|((((var, init), cond), update), body)| {
                        let (name, ty) = var;
                        let name_expr = match ty {
                            Some(t) => WindExpr::TypeExpr {
                                expr: Box::new(WindExpr::Identifier(name)),
                                ty: t,
                            },
                            None => WindExpr::Identifier(name),
                        };
                        WindStmt::For {
                            init: init.or(Some(name_expr)).map(Box::new),
                            condition: cond.map(Box::new),
                            update: update.map(Box::new),
                            body: Box::new(body),
                        }
                    });

                let for_in = Self::identifier()
                    .then_ignore(Self::select_token(WindToken::Colon))
                    .then(Self::expr_parser())
                    .then_ignore(Self::select_token(WindToken::CloseParen))
                    .then(block.clone())
                    .map(|((var, iterable), body)| WindStmt::ForIn {
                        var,
                        iterable: Box::new(iterable),
                        body: Box::new(body),
                    });

                for_head.ignore_then(choice((c_style, for_in)))
            };

            let while_stmt = Self::select_token(WindToken::While)
                .ignore_then(Self::expr_parser())
                .then(block.clone())
                .map(|(condition, body)| WindStmt::While {
                    condition: Box::new(condition),
                    body: Box::new(body),
                });

            let return_stmt = Self::select_token(WindToken::Return)
                .ignore_then(Self::expr_or_type_expr().or_not())
                .then_ignore(Self::select_token(WindToken::Semicolon).or_not())
                .map(|e| WindStmt::Return(e.map(Box::new)));

            let fn_def = Self::visibility()
                .then(Self::select_token(WindToken::Fn).ignore_then(Self::identifier()))
                .then(
                    Self::select_token(WindToken::OpenParen)
                        .ignore_then(
                            Self::fn_param()
                                .separated_by(Self::select_token(WindToken::Comma))
                                .allow_trailing()
                                .collect(),
                        )
                        .then_ignore(Self::select_token(WindToken::CloseParen)),
                )
                .then(
                    Self::select_token(WindToken::Arrow)
                        .ignore_then(Self::type_expr())
                        .or_not(),
                )
                .then(Self::make_fn_which_clause())
                .then(block.clone())
                .map(
                    |(((((public, name), params), return_type), which), body)| WindStmt::FnDef {
                        public,
                        name,
                        params,
                        return_type,
                        which,
                        body: Box::new(body),
                    },
                );

            let struct_field = {
                let where_body = Self::select_token(WindToken::Where).ignore_then(
                    Self::select_token(WindToken::OpenBrace)
                        .ignore_then(
                            Self::expr_or_type_expr()
                                .separated_by(Self::select_token(WindToken::Semicolon))
                                .allow_trailing()
                                .collect(),
                        )
                        .then_ignore(Self::select_token(WindToken::CloseBrace)),
                );
                let arrow_body = Self::select_token(WindToken::Arrow).ignore_then(
                    Self::select_token(WindToken::OpenBrace)
                        .ignore_then(
                            Self::expr_or_type_expr()
                                .separated_by(Self::select_token(WindToken::Semicolon))
                                .allow_trailing()
                                .collect(),
                        )
                        .then_ignore(Self::select_token(WindToken::CloseBrace)),
                );
                let conditions = where_body.or(arrow_body).or_not();

                let default_value = Self::select_token(WindToken::Equal)
                    .ignore_then(Self::expr_or_type_expr())
                    .or_not();

                Self::select_token(WindToken::Static)
                    .or_not()
                    .then(Self::visibility())
                    .then(Self::identifier())
                    .then(Self::select_token(WindToken::Colon).ignore_then(Self::type_expr()))
                    .then(conditions)
                    .then(default_value)
                    .then_ignore(Self::select_token(WindToken::Semicolon))
                    .map(
                        |(((((static_tok, vis), name), ty), conditions), default_val)| {
                            let is_static = static_tok.is_some();
                            let cond_expr = conditions
                                .and_then(|exprs: Vec<WindExpr>| exprs.into_iter().next());
                            WindStructField {
                                public: vis,
                                is_static,
                                name,
                                ty,
                                which: None,
                                conditions: cond_expr,
                                default_value: default_val.map(Box::new),
                            }
                        },
                    )
            };

            let struct_def = Self::visibility()
                .then(Self::select_token(WindToken::Struct).ignore_then(Self::identifier()))
                .then(
                    Self::select_token(WindToken::OpenBrace)
                        .ignore_then(struct_field.repeated().collect())
                        .then_ignore(Self::select_token(WindToken::CloseBrace)),
                )
                .map(|((public, name), fields)| WindStmt::StructDef {
                    public,
                    name,
                    fields,
                });

            let enum_def = {
                let variant = Self::identifier().then(Self::type_annotation().or_not());
                Self::visibility()
                    .then(Self::select_token(WindToken::Enum).ignore_then(Self::identifier()))
                    .then(
                        Self::select_token(WindToken::OpenBrace)
                            .ignore_then(
                                variant
                                    .separated_by(Self::select_token(WindToken::Comma))
                                    .allow_trailing()
                                    .collect(),
                            )
                            .then_ignore(Self::select_token(WindToken::CloseBrace)),
                    )
                    .map(|((public, name), variants)| WindStmt::EnumDef {
                        public,
                        name,
                        variants,
                    })
            };

            let type_def = {
                let where_block = Self::select_token(WindToken::Where)
                    .ignore_then(
                        Self::select_token(WindToken::OpenBrace)
                            .ignore_then(stmt.clone().repeated().collect())
                            .then_ignore(Self::select_token(WindToken::CloseBrace)),
                    )
                    .map(|stmts: Vec<WindStmt>| {
                        stmts
                            .into_iter()
                            .filter_map(|s| {
                                if let WindStmt::Expr(e) = s {
                                    Some(*e)
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                    .or_not()
                    .map(|c| c.unwrap_or_default());
                Self::visibility()
                    .then(Self::select_token(WindToken::Type).ignore_then(Self::identifier()))
                    .then(Self::select_token(WindToken::Equal).ignore_then(Self::type_expr()))
                    .then(where_block)
                    .map(
                        |(((public, name), base_type), conditions)| WindStmt::TypeDef {
                            public,
                            name,
                            base_type,
                            conditions,
                        },
                    )
            };

            let const_def = Self::select_token(WindToken::Const)
                .ignore_then(Self::identifier())
                .then(Self::type_annotation())
                .then(Self::select_token(WindToken::Equal).ignore_then(Self::expr_or_type_expr()))
                .then_ignore(Self::select_token(WindToken::Semicolon))
                .map(|((name, ty), value)| WindStmt::ConstDef {
                    name,
                    ty,
                    value: Box::new(value),
                });

            let constatic_def = Self::select_token(WindToken::Constatic)
                .ignore_then(Self::identifier())
                .then(Self::type_annotation())
                .then(Self::select_token(WindToken::Equal).ignore_then(Self::expr_or_type_expr()))
                .then_ignore(Self::select_token(WindToken::Semicolon))
                .map(|((name, ty), value)| WindStmt::ConstaticDef {
                    name,
                    ty,
                    value: Box::new(value),
                });

            let explain_def = Self::select_token(WindToken::Explain)
                .ignore_then(Self::identifier())
                .then(Self::type_annotation())
                .then(Self::select_token(WindToken::Equal).ignore_then(Self::expr_or_type_expr()))
                .then_ignore(Self::select_token(WindToken::Semicolon))
                .map(|((name, ty), value)| WindStmt::ExplainDef {
                    name,
                    ty,
                    value: Box::new(value),
                });

            let tag_def = {
                let tag_body = Self::select_token(WindToken::OpenBrace)
                    .ignore_then(stmt.clone().repeated().collect())
                    .then_ignore(Self::select_token(WindToken::CloseBrace));
                Self::select_token(WindToken::Tag)
                    .ignore_then(Self::identifier())
                    .then(Self::select_token(WindToken::Equal).ignore_then(tag_body))
                    .then_ignore(Self::select_token(WindToken::Semicolon))
                    .map(|(name, body)| WindStmt::Let {
                        name: name.clone(),
                        ty: None,
                        value: Box::new(WindExpr::TagExpr { name, body }),
                    })
            };


            let trait_def = {
                let sig = Self::visibility()
                    .then(Self::select_token(WindToken::Fn).ignore_then(Self::identifier()))
                    .then(
                        Self::select_token(WindToken::OpenParen)
                            .ignore_then(
                                Self::fn_param()
                                    .separated_by(Self::select_token(WindToken::Comma))
                                    .allow_trailing()
                                    .collect(),
                            )
                            .then_ignore(Self::select_token(WindToken::CloseParen)),
                    )
                    .then(
                        Self::select_token(WindToken::Arrow)
                            .ignore_then(Self::type_expr())
                            .or_not(),
                    )
                    .then_ignore(Self::select_token(WindToken::Semicolon))
                    .map(|(((public, name), params), return_type)| WindFnSignature {
                        public,
                        name,
                        params,
                        return_type,
                        which: None,
                    });
                Self::visibility()
                    .then(Self::select_token(WindToken::Trait).ignore_then(Self::identifier()))
                    .then(
                        Self::select_token(WindToken::OpenBrace)
                            .ignore_then(sig.repeated().collect())
                            .then_ignore(Self::select_token(WindToken::CloseBrace)),
                    )
                    .map(|((public, name), functions)| WindStmt::TraitDef {
                        public,
                        name,
                        functions,
                    })
            };

            let extra_def = {
                let extra_fn = Self::visibility()
                    .then(Self::select_token(WindToken::Fn).ignore_then(Self::identifier()))
                    .then(
                        Self::select_token(WindToken::OpenParen)
                            .ignore_then(
                                Self::fn_param()
                                    .separated_by(Self::select_token(WindToken::Comma))
                                    .allow_trailing()
                                    .collect(),
                            )
                            .then_ignore(Self::select_token(WindToken::CloseParen)),
                    )
                    .then(
                        Self::select_token(WindToken::Arrow)
                            .ignore_then(Self::type_expr())
                            .or_not(),
                    )
                    .then(Self::make_fn_which_clause())
                    .then(block.clone())
                    .map(|(((((public, name), params), return_type), which), body)| {
                        WindStmt::FnDef {
                            public,
                            name,
                            params,
                            return_type,
                            which,
                            body: Box::new(body),
                        }
                    });

                let extra_target = Self::identifier()
                    .then(
                        Self::select_token(WindToken::Colon)
                            .ignore_then(Self::identifier())
                            .or_not(),
                    )
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
                    .map(|((name, target), functions)| WindStmt::ExtraDef {
                        public: false,
                        name,
                        target,
                        functions,
                    })
            };

            let impl_def = {
                let impl_fn = Self::visibility()
                    .then(Self::select_token(WindToken::Fn).ignore_then(Self::identifier()))
                    .then(
                        Self::select_token(WindToken::OpenParen)
                            .ignore_then(
                                Self::fn_param()
                                    .separated_by(Self::select_token(WindToken::Comma))
                                    .allow_trailing()
                                    .collect(),
                            )
                            .then_ignore(Self::select_token(WindToken::CloseParen)),
                    )
                    .then(
                        Self::select_token(WindToken::Arrow)
                            .ignore_then(Self::type_expr())
                            .or_not(),
                    )
                    .then(Self::make_fn_which_clause())
                    .then(block.clone())
                    .map(|(((((public, name), params), return_type), which), body)| {
                        WindStmt::FnDef {
                            public,
                            name,
                            params,
                            return_type,
                            which,
                            body: Box::new(body),
                        }
                    });

                Self::select_token(WindToken::Impl)
                    .ignore_then(Self::identifier())
                    .then(Self::select_token(WindToken::For).ignore_then(Self::identifier()))
                    .then(
                        Self::select_token(WindToken::OpenBrace)
                            .ignore_then(impl_fn.repeated().collect())
                            .then_ignore(Self::select_token(WindToken::CloseBrace)),
                    )
                    .map(|((trait_name, target), functions)| WindStmt::ImplDef {
                        public: false,
                        trait_name,
                        target,
                        functions,
                    })
            };

            let group_def = {
                let group_rule = {
                    let self_field_rule = Self::select_token(WindToken::SelfKw)
                        .ignore_then(
                            Self::select_token(WindToken::Dot).ignore_then(Self::identifier()),
                        )
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
                    .ignore_then(
                        Self::fn_param()
                            .separated_by(Self::select_token(WindToken::Comma))
                            .collect(),
                    )
                    .then_ignore(Self::select_token(WindToken::CloseParen));

                let group_header = Self::identifier()
                    .then(
                        Self::select_token(WindToken::Colon)
                            .ignore_then(Self::identifier())
                            .map(|t| (Some(t), None))
                            .or(group_params.map(|p| (None, Some(p))))
                            .or_not(),
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
                    .map(|((name, target, params), rules)| WindStmt::GroupDef {
                        public: false,
                        name,
                        target,
                        params,
                        rules,
                    })
            };

            let apply_stmt = {
                let fields = Self::select_token(WindToken::OpenBrace)
                    .ignore_then(
                        Self::identifier()
                            .separated_by(Self::select_token(WindToken::Comma))
                            .collect(),
                    )
                    .then_ignore(Self::select_token(WindToken::CloseBrace));

                Self::identifier()
                    .then(Self::select_token(WindToken::At).ignore_then(Self::identifier()))
                    .then(Self::select_token(WindToken::Arrow).ignore_then(fields))
                    .then_ignore(Self::select_token(WindToken::Semicolon).or_not())
                    .map(|((group, target), fields)| WindStmt::Apply {
                        group,
                        target,
                        fields,
                    })
            };

            choice((
                fn_def,
                struct_def,
                enum_def,
                type_def,
                trait_def,
                extra_def,
                impl_def,
                group_def,
                apply_stmt,
                tag_def,
                const_def,
                constatic_def,
                explain_def,

                if_stmt,
                for_stmt,
                while_stmt,
                return_stmt,
                let_stmt,
                assign_stmt,
                expr_stmt,
            ))
        })
        .repeated()
        .collect()
        .map(|items| WindProgram { items })
        .then_ignore(end())
    }

    pub fn parse(
        source: &str,
        tokens: &[WindSpannedToken],
    ) -> Result<WindProgram, Vec<WindParseError>> {
        log::debug!("开始语法分析, 输入 {} 个标记", tokens.len());

        let eoi = tokens.last().map(|(_, s)| s.end..s.end).unwrap_or(0..0);
        let input = Stream::from_iter(tokens.iter().cloned())
            .map(eoi.into(), |(t, s): (WindToken, WindSpan)| (t, s));

        match Self::program_parser().parse(input).into_result() {
            Ok(program) => {
                log::debug!("语法分析成功: {} 个顶层条目", program.items.len());
                Ok(program)
            }
            Err(errors) => {
                let parse_errors: Vec<WindParseError> = errors
                    .into_iter()
                    .enumerate()
                    .map(|(i, e)| {
                        let span = e.span().clone();
                        let found = e.found().cloned();
                        let expected: Vec<String> = e
                            .expected()
                            .map(|p| match p {
                                RichPattern::Token(t) => format!("{t:?}"),
                                RichPattern::Label(label) => label.to_string(),
                                RichPattern::Identifier(id) => id.clone(),
                                RichPattern::Any => "<any>".to_string(),
                                RichPattern::SomethingElse => "<something>".to_string(),
                                &_ => "<unknown>".to_string(),
                            })
                            .collect();
                        let (line, col) =
                            byte_to_line_col(source, span.start);
                        let msg = format!(
                            "语法错误 #{i} [{}:{}]: 发现 {}, 期望 {}",
                            line,
                            col,
                            found
                                .as_ref()
                                .map(|t| format!("{t:?}"))
                                .as_deref()
                                .unwrap_or("<无>"),
                            if expected.is_empty() {
                                "<未知>".to_string()
                            } else {
                                expected.join(", ")
                            },
                        );
                        log::error!("{msg}");
                        WindParseError {
                            message: msg,
                            span,
                            found,
                            expected,
                        }
                    })
                    .collect();
                Err(parse_errors)
            }
        }
    }
}
