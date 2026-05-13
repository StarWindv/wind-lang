use super::binary_op::WindBinaryOp;
use super::unary_op::WindUnaryOp;
use super::ty::WindType;
use super::stmt::WindStmt;

#[derive(Debug, Clone, PartialEq)]
pub enum WindExpr {
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    CharLiteral(String),
    BoolLiteral(bool),
    NoneLiteral,
    Identifier(String),
    Binary {
        left: Box<WindExpr>,
        op: WindBinaryOp,
        right: Box<WindExpr>,
    },
    Unary {
        op: WindUnaryOp,
        expr: Box<WindExpr>,
    },
    Call {
        callee: Box<WindExpr>,
        args: Vec<WindExpr>,
    },
    FieldAccess {
        object: Box<WindExpr>,
        field: String,
    },
    Index {
        expr: Box<WindExpr>,
        index: Box<WindExpr>,
    },
    ScopeRef {
        object: Box<WindExpr>,
        member: String,
    },
    TypeExpr {
        expr: Box<WindExpr>,
        ty: WindType,
    },
    Group(Box<WindExpr>),
    MapLiteral(Vec<(WindExpr, WindExpr)>),
    ArrayLiteral(Vec<WindExpr>),
    IfExpr {
        condition: Box<WindExpr>,
        then_branch: Box<WindExpr>,
        else_branch: Option<Box<WindExpr>>,
    },
    TagExpr {
        name: String,
        body: Vec<WindStmt>,
    },
}
