#[derive(Debug, Clone, PartialEq)]
pub enum WindUnaryOp {
    Neg,
    Not,
    Inc,
    Dec,
    IncPost,
    DecPost,
}
