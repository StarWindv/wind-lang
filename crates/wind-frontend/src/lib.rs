mod modules;


pub mod lexer {
    use crate::modules;
    pub use modules::types::tokens::{
        WindSpan,
        WindSpannedToken,
        WindToken
    };
}

pub mod parser {
    use crate::modules;
    pub use modules::types::ast::{
        WindTokenSlice,
        WindParser
    };
}

pub mod errors {
    use crate::modules;
    pub use modules::types::{
        ast::{
            WindParseError
        },
        LexError
    };
}

pub mod ast_node {
    use crate::modules;
    pub use modules::types::ast::{
        WindAssignOp,
        WindBinaryOp,
        WindExpr,
        WindFnParam,
        WindFnSignature,
        WindGroupRule,
        WindProgram,
        WindStmt,
        WindStructField,
        WindType,
        WindUnaryOp,
        WindWhichClause,
    };
}
