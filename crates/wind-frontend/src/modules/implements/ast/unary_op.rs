use crate::modules::types::ast::WindUnaryOp;
use crate::modules::types::tokens::WindToken;

impl WindUnaryOp {
    pub fn from_token(tok: &WindToken) -> Option<Self> {
        match tok {
            WindToken::Minus => Some(WindUnaryOp::Neg),
            WindToken::Bang => Some(WindUnaryOp::Not),
            _ => None,
        }
    }
}
