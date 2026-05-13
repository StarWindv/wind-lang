use crate::modules::types::tokens::WindSpan;

pub mod ast;
pub mod tokens;

#[derive(Debug, Clone)]
pub struct LexError {
    pub message: String,
    pub span: WindSpan,
}
