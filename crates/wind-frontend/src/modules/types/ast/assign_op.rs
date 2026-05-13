#[derive(Debug, Clone, PartialEq)]
pub enum WindAssignOp {
    Direct,
    LeftAbs,
    RightAbs,
    SumEq,
    DiffEq,
    ProdEq,
    QuotEq,
}
