#[derive(Debug, Clone)]
pub struct SemanticError {
    pub message: String,
    pub span: Option<(usize, usize)>,
}
