use crate::modules::implements::tokens::Span;
use crate::modules::types::tokens::Token;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
    pub found: Option<Token>,
    pub expected: Vec<String>,
}
