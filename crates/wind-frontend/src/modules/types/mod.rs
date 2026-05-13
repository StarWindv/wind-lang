use crate::modules::types::tokens::WindSpan;

pub mod tokens;
pub mod ast;

#[derive(Debug, Clone)]
pub struct LexError {
    pub message: String,
    pub span: WindSpan,
}
