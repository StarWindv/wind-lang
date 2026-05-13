use crate::modules::types::ast::WindBinaryOp;
use crate::modules::types::tokens::WindToken;

impl WindBinaryOp {
    pub fn from_token(tok: &WindToken) -> Option<Self> {
        match tok {
            WindToken::Plus => Some(WindBinaryOp::Add),
            WindToken::Minus => Some(WindBinaryOp::Sub),
            WindToken::Star => Some(WindBinaryOp::Mul),
            WindToken::Slash => Some(WindBinaryOp::Div),
            WindToken::DoubleSlash => Some(WindBinaryOp::IntDiv),
            WindToken::Percent => Some(WindBinaryOp::Mod),
            WindToken::AndAnd => Some(WindBinaryOp::And),
            WindToken::OrOr => Some(WindBinaryOp::Or),
            WindToken::Amp => Some(WindBinaryOp::BitAnd),
            WindToken::Pipe => Some(WindBinaryOp::BitOr),
            WindToken::Caret => Some(WindBinaryOp::BitXor),
            WindToken::LeftShift => Some(WindBinaryOp::Shl),
            WindToken::RightShift => Some(WindBinaryOp::Shr),
            WindToken::EqualEqual => Some(WindBinaryOp::Eq),
            WindToken::NotEqual => Some(WindBinaryOp::Neq),
            WindToken::Less => Some(WindBinaryOp::Lt),
            WindToken::Greater => Some(WindBinaryOp::Gt),
            WindToken::LessEqual => Some(WindBinaryOp::Le),
            WindToken::GreaterEqual => Some(WindBinaryOp::Ge),
            WindToken::NotLess => Some(WindBinaryOp::NotLt),
            WindToken::NotGreater => Some(WindBinaryOp::NotGt),
            WindToken::In => Some(WindBinaryOp::In),
            _ => None,
        }
    }
}
