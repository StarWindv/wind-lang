use super::expr::WindExpr;
use super::assign_op::WindAssignOp;
use super::ty::WindType;
use super::fn_param::WindFnParam;
use super::struct_field::WindStructField;
use super::fn_signature::WindFnSignature;
use super::group_rule::WindGroupRule;
use super::which_clause::WindWhichClause;

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
        public: bool,
        name: String,
        params: Vec<WindFnParam>,
        return_type: Option<WindType>,
        which: Option<Vec<WindWhichClause>>,
        body: Box<WindStmt>,
    },
    StructDef {
        public: bool,
        name: String,
        fields: Vec<WindStructField>,
    },
    EnumDef {
        public: bool,
        name: String,
        variants: Vec<(String, Option<WindType>)>,
    },
    TypeDef {
        public: bool,
        name: String,
        base_type: WindType,
        conditions: Vec<WindExpr>,
    },
    ExtraDef {
        public: bool,
        name: Option<String>,
        target: String,
        functions: Vec<WindStmt>,
    },
    ImplDef {
        public: bool,
        trait_name: String,
        target: String,
        functions: Vec<WindStmt>,
    },
    TraitDef {
        public: bool,
        name: String,
        functions: Vec<WindFnSignature>,
    },
    GroupDef {
        public: bool,
        name: String,
        target: Option<String>,
        params: Option<Vec<WindFnParam>>,
        rules: Vec<WindGroupRule>,
    },
    ConstDef {
        name: String,
        ty: WindType,
        value: Box<WindExpr>,
    },
    ConstaticDef {
        name: String,
        ty: WindType,
        value: Box<WindExpr>,
    },
    ToStmt {
        tag: String,
    },
    Apply {
        group: String,
        target: String,
        fields: Vec<String>,
    },
    ExplainDef {
        name: String,
        ty: WindType,
        value: Box<WindExpr>,
    },
    ForIn {
        var: String,
        iterable: Box<WindExpr>,
        body: Box<WindStmt>,
    },
}
