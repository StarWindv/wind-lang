use crate::modules::types::ast::BinaryOp;
use crate::modules::types::tokens::Token;

impl BinaryOp {
    pub fn from_token(tok: &Token) -> Option<Self> {
        match tok {
            Token::Plus => Some(BinaryOp::Add),
            Token::Minus => Some(BinaryOp::Sub),
            Token::Star => Some(BinaryOp::Mul),
            Token::Slash => Some(BinaryOp::Div),
            Token::DoubleSlash => Some(BinaryOp::IntDiv),
            Token::Percent => Some(BinaryOp::Mod),
            Token::AndAnd => Some(BinaryOp::And),
            Token::OrOr => Some(BinaryOp::Or),
            Token::Amp => Some(BinaryOp::BitAnd),
            Token::Pipe => Some(BinaryOp::BitOr),
            Token::Caret => Some(BinaryOp::BitXor),
            Token::LeftShift => Some(BinaryOp::Shl),
            Token::RightShift => Some(BinaryOp::Shr),
            Token::EqualEqual => Some(BinaryOp::Eq),
            Token::NotEqual => Some(BinaryOp::Neq),
            Token::Less => Some(BinaryOp::Lt),
            Token::Greater => Some(BinaryOp::Gt),
            Token::LessEqual => Some(BinaryOp::Le),
            Token::GreaterEqual => Some(BinaryOp::Ge),
            Token::NotLess => Some(BinaryOp::NotLt),
            Token::NotGreater => Some(BinaryOp::NotGt),
            _ => None,
        }
    }
}
