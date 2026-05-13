use crate::modules::types::tokens::{WindSpan, WindToken};

#[derive(Debug, Clone)]
pub struct WindParseError {
    pub message: String,
    pub span: WindSpan,
    pub found: Option<WindToken>,
    pub expected: Vec<String>,
}
