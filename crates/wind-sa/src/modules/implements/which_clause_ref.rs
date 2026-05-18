use crate::modules::types::WindWhichClauseRef;

impl WindWhichClauseRef {
    pub fn from_ast(clause: &wind_frontend::ast_node::WindWhichClause) -> Self {
        Self {
            method: clause.method.clone(),
            after: clause.after.clone(),
        }
    }
}
