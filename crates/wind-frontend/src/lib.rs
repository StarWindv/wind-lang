mod modules;

pub mod lexer {
    use crate::modules;
    pub use modules::types::tokens::{WindSpan, WindSpannedToken, WindToken};
    pub use modules::implements::byte_to_line_col;
}

pub mod parser {
    use crate::modules;
    pub use modules::types::ast::{WindParser, WindTokenSlice};
}

pub mod errors {
    use crate::modules;
    pub use modules::types::{LexError, ast::WindParseError};
}

pub mod ast_node {
    use crate::modules;
    pub use modules::types::ast::{
        WindAssignOp, WindBinaryOp, WindExpr, WindFnParam, WindFnSignature, WindGroupRule,
        WindProgram, WindStmt, WindStructField, WindType, WindUnaryOp, WindWhichClause,
    };
}
