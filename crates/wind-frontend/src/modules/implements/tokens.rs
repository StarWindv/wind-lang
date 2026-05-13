use logos::Logos;
use crate::modules::types::LexError;
use crate::modules::types::tokens::Token;

pub type Span = std::ops::Range<usize>;

pub type SpannedToken = (Token, Span);



pub fn lex(source: &str) -> Result<Vec<SpannedToken>, Vec<LexError>> {
    log::debug!("开始词法分析, 源长度: {}", source.len());

    let lexer = Token::lexer(source);
    let mut tokens = Vec::new();
    let mut errors = Vec::new();

    for (result, span) in lexer.spanned() {
        match result {
            Ok(tok) => {
                if tok == Token::LineComment {
                    continue;
                }
                log::debug!("词法: {:?} @ {:?}", tok, span);
                tokens.push((tok, span));
            }
            Err(()) => {
                let sliced = &source[span.clone()];
                let msg = format!("词法错误: 无法识别的字符序列 `{sliced}` @ {span:?}");
                log::error!("{msg}");
                errors.push(LexError {
                    message: msg,
                    span,
                });
            }
        }
    }

    log::debug!(
        "词法分析完成: {} 个标记, {} 个错误",
        tokens.len(),
        errors.len()
    );

    if errors.is_empty() {
        Ok(tokens)
    } else {
        Err(errors)
    }
}

impl Token {
    pub fn as_keyword_str(&self) -> Option<&'static str> {
        match self {
            Token::Fn => Some("fn"),
            Token::Struct => Some("struct"),
            Token::Enum => Some("enum"),
            Token::Extra => Some("extra"),
            Token::Impl => Some("impl"),
            Token::Trait => Some("trait"),
            Token::Const => Some("const"),
            Token::Constatic => Some("constatic"),
            Token::Which => Some("which"),
            Token::Where => Some("where"),
            Token::Type => Some("type"),
            Token::Group => Some("group"),
            Token::In => Some("in"),
            Token::Pub => Some("pub"),
            Token::Public => Some("public"),
            Token::Return => Some("return"),
            Token::For => Some("for"),
            Token::While => Some("while"),
            Token::If => Some("if"),
            Token::Elif => Some("elif"),
            Token::Else => Some("else"),
            Token::To => Some("to"),
            Token::Tag => Some("tag"),
            Token::SelfKw => Some("self"),
            Token::ThisKw => Some("this"),
            Token::ItKw => Some("it"),
            Token::TrueKw => Some("true"),
            Token::FalseKw => Some("false"),
            Token::NoneKw => Some("None"),
            _ => None,
        }
    }
}
