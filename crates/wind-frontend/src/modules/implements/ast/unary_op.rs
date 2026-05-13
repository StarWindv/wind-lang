use crate::modules::types::ast::UnaryOp;
use crate::modules::types::tokens::Token;

impl UnaryOp {
    pub fn from_token(tok: &Token) -> Option<Self> {
        match tok {
            Token::Minus => Some(UnaryOp::Neg),
            Token::Bang => Some(UnaryOp::Not),
            _ => None,
        }
    }
}
