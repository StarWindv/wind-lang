use super::expr::Expr;
use super::assign_op::AssignOp;
use super::ty::Type;
use super::fn_param::FnParam;
use super::struct_field::StructField;
use super::fn_signature::FnSignature;
use super::group_rule::GroupRule;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        ty: Option<Type>,
        value: Box<Expr>,
    },
    Assignment {
        target: Box<Expr>,
        op: AssignOp,
        value: Box<Expr>,
    },
    Expr(Box<Expr>),
    Block(Vec<Stmt>),
    If {
        condition: Box<Expr>,
        then_branch: Box<Stmt>,
        elif_branches: Vec<(Expr, Stmt)>,
        else_branch: Option<Box<Stmt>>,
    },
    For {
        init: Option<Box<Expr>>,
        condition: Option<Box<Expr>>,
        update: Option<Box<Expr>>,
        body: Box<Stmt>,
    },
    While {
        condition: Box<Expr>,
        body: Box<Stmt>,
    },
    Return(Option<Box<Expr>>),
    FnDef {
        name: String,
        params: Vec<FnParam>,
        return_type: Option<Type>,
        body: Box<Stmt>,
    },
    StructDef {
        name: String,
        fields: Vec<StructField>,
    },
    EnumDef {
        name: String,
        variants: Vec<(String, Option<Type>)>,
    },
    TypeDef {
        name: String,
        base_type: Type,
        conditions: Vec<Expr>,
    },
    ExtraDef {
        name: String,
        target: String,
        functions: Vec<Stmt>,
    },
    ImplDef {
        trait_name: String,
        target: String,
        functions: Vec<Stmt>,
    },
    TraitDef {
        name: String,
        functions: Vec<FnSignature>,
    },
    GroupDef {
        name: String,
        target: Option<String>,
        params: Option<Vec<FnParam>>,
        rules: Vec<GroupRule>,
    },
    ConstDef {
        name: String,
        value: Box<Expr>,
    },
    ConstaticDef {
        name: String,
        value: Box<Expr>,
    },
    ToStmt {
        tag: String,
    },
}
