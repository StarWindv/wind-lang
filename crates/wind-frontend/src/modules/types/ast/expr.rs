use super::binary_op::BinaryOp;
use super::unary_op::UnaryOp;
use super::ty::Type;
use super::stmt::Stmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    CharLiteral(String),
    BoolLiteral(bool),
    NoneLiteral,
    Identifier(String),
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },
    Index {
        expr: Box<Expr>,
        index: Box<Expr>,
    },
    ScopeRef {
        object: Box<Expr>,
        member: String,
    },
    TypeExpr {
        expr: Box<Expr>,
        ty: Type,
    },
    Group(Box<Expr>),
    MapLiteral(Vec<(Expr, Expr)>),
    ArrayLiteral(Vec<Expr>),
    IfExpr {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },
    TagExpr {
        name: String,
        body: Vec<Stmt>,
    },
}
