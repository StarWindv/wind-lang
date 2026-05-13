use crate::modules::implements::tokens::Span;

pub mod tokens;
pub mod ast;

#[derive(Debug, Clone)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}
