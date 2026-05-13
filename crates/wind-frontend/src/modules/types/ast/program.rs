use super::stmt::WindStmt;

#[derive(Debug, Clone, PartialEq)]
pub struct WindProgram {
    pub items: Vec<WindStmt>,
}
