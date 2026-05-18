use crate::modules::types::SemanticError;

impl SemanticError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
        }
    }

    pub fn with_span(message: impl Into<String>, start: usize, end: usize) -> Self {
        Self {
            message: message.into(),
            span: Some((start, end)),
        }
    }
}
