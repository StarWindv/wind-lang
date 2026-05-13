use super::expr::WindExpr;
use super::assign_op::WindAssignOp;
use super::ty::WindType;
use super::fn_param::WindFnParam;
use super::struct_field::WindStructField;
use super::fn_signature::WindFnSignature;
use super::group_rule::WindGroupRule;

#[derive(Debug, Clone, PartialEq)]
pub enum WindStmt {
    Let {
        name: String,
        ty: Option<WindType>,
        value: Box<WindExpr>,
    },
    Assignment {
        target: Box<WindExpr>,
        op: WindAssignOp,
        value: Box<WindExpr>,
    },
    Expr(Box<WindExpr>),
    Block(Vec<WindStmt>),
    If {
        condition: Box<WindExpr>,
        then_branch: Box<WindStmt>,
        elif_branches: Vec<(WindExpr, WindStmt)>,
        else_branch: Option<Box<WindStmt>>,
    },
    For {
        init: Option<Box<WindExpr>>,
        condition: Option<Box<WindExpr>>,
        update: Option<Box<WindExpr>>,
        body: Box<WindStmt>,
    },
    While {
        condition: Box<WindExpr>,
        body: Box<WindStmt>,
    },
    Return(Option<Box<WindExpr>>),
    FnDef {
        name: String,
        params: Vec<WindFnParam>,
        return_type: Option<WindType>,
        body: Box<WindStmt>,
    },
    StructDef {
        name: String,
        fields: Vec<WindStructField>,
    },
    EnumDef {
        name: String,
        variants: Vec<(String, Option<WindType>)>,
    },
    TypeDef {
        name: String,
        base_type: WindType,
        conditions: Vec<WindExpr>,
    },
    ExtraDef {
        name: String,
        target: String,
        functions: Vec<WindStmt>,
    },
    ImplDef {
        trait_name: String,
        target: String,
        functions: Vec<WindStmt>,
    },
    TraitDef {
        name: String,
        functions: Vec<WindFnSignature>,
    },
    GroupDef {
        name: String,
        target: Option<String>,
        params: Option<Vec<WindFnParam>>,
        rules: Vec<WindGroupRule>,
    },
    ConstDef {
        name: String,
        value: Box<WindExpr>,
    },
    ConstaticDef {
        name: String,
        value: Box<WindExpr>,
    },
    ToStmt {
        tag: String,
    },
}
