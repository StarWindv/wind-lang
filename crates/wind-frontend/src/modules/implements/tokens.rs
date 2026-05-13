use logos::Logos;
use crate::modules::types::LexError;
use crate::modules::types::tokens::{WindSpannedToken, WindToken};



impl WindToken {
    pub fn as_keyword_str(&self) -> Option<&'static str> {
        match self {
            WindToken::Fn => Some("fn"),
            WindToken::Struct => Some("struct"),
            WindToken::Enum => Some("enum"),
            WindToken::Extra => Some("extra"),
            WindToken::Impl => Some("impl"),
            WindToken::Trait => Some("trait"),
            WindToken::Const => Some("const"),
            WindToken::Constatic => Some("constatic"),
            WindToken::Which => Some("which"),
            WindToken::Where => Some("where"),
            WindToken::Type => Some("type"),
            WindToken::Group => Some("group"),
            WindToken::In => Some("in"),
            WindToken::Pub => Some("pub"),
            WindToken::Public => Some("public"),
            WindToken::Return => Some("return"),
            WindToken::For => Some("for"),
            WindToken::While => Some("while"),
            WindToken::If => Some("if"),
            WindToken::Elif => Some("elif"),
            WindToken::Else => Some("else"),
            WindToken::To => Some("to"),
            WindToken::Tag => Some("tag"),
            WindToken::SelfKw => Some("self"),
            WindToken::ThisKw => Some("this"),
            WindToken::ItKw => Some("it"),
            WindToken::TrueKw => Some("true"),
            WindToken::FalseKw => Some("false"),
            WindToken::NoneKw => Some("None"),
            _ => None,
        }
    }

    pub fn lex(source: &str) -> Result<Vec<WindSpannedToken>, Vec<LexError>> {
        log::debug!("开始词法分析, 源长度: {}", source.len());

        let lexer = WindToken::lexer(source);
        let mut tokens = Vec::new();
        let mut errors = Vec::new();

        for (result, span) in lexer.spanned() {
            match result {
                Ok(tok) => {
                    if tok == WindToken::LineComment {
                        continue;
                    }
                    log::debug!("词法: {:?} @ {:?}", tok, span);
                    tokens.push((tok, span));
                }
                Err(()) => {
                    let sliced = &source[span.clone()];
                    let (line, col) = crate::modules::types::tokens::byte_to_line_col(source, span.start);
                    let msg = format!("词法错误 [{line}:{col}]: 无法识别的字符序列 `{sliced}`");
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
}
