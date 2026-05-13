#[derive(Debug, Clone, PartialEq)]
pub struct WhichClause {
    pub method: String,
    pub after: Vec<String>,
}
