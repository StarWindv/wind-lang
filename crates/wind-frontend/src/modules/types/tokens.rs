use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq, Eq, Hash)]
#[logos(
    skip r"[ \t\n\r\f]+",
    skip r"```[\s\S]*?```"
)]
pub enum Token {
    #[token("fn")]
    Fn,
    #[token("struct")]
    Struct,
    #[token("enum")]
    Enum,
    #[token("extra")]
    Extra,
    #[token("impl")]
    Impl,
    #[token("trait")]
    Trait,
    #[token("const")]
    Const,
    #[token("constatic")]
    Constatic,
    #[token("which")]
    Which,
    #[token("where")]
    Where,
    #[token("type")]
    Type,
    #[token("group")]
    Group,
    #[token("in")]
    In,
    #[token("pub")]
    Pub,
    #[token("public")]
    Public,
    #[token("return")]
    Return,
    #[token("for")]
    For,
    #[token("while")]
    While,
    #[token("if")]
    If,
    #[token("elif")]
    Elif,
    #[token("else")]
    Else,
    #[token("to")]
    To,
    #[token("tag")]
    Tag,
    #[token("self")]
    SelfKw,
    #[token("this")]
    ThisKw,
    #[token("it")]
    ItKw,
    #[token("true")]
    TrueKw,
    #[token("false")]
    FalseKw,
    #[token("None")]
    NoneKw,

    #[token("<=")]
    LessEqual,
    #[token(">=")]
    GreaterEqual,
    #[token("!=")]
    NotEqual,
    #[token("!<")]
    NotLess,
    #[token("!>")]
    NotGreater,
    #[token("==")]
    EqualEqual,
    #[token("<:")]
    LeftAssign,
    #[token(":>")]
    RightAssign,
    #[token("//")]
    DoubleSlash,
    #[token("<<")]
    LeftShift,
    #[token(">>")]
    RightShift,
    #[token("::")]
    DoubleColon,
    #[token("->")]
    Arrow,
    #[token("&&")]
    AndAnd,
    #[token("||")]
    OrOr,

    #[token(".")]
    Dot,
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[token("[")]
    OpenBracket,
    #[token("]")]
    CloseBracket,
    #[token("{")]
    OpenBrace,
    #[token("}")]
    CloseBrace,
    #[token("@")]
    At,
    #[token("!")]
    Bang,
    #[token("-")]
    Minus,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("*")]
    Star,
    #[token("+")]
    Plus,
    #[token("&")]
    Amp,
    #[token("|")]
    Pipe,
    #[token("^")]
    Caret,
    #[token("=")]
    Equal,
    #[token("<")]
    Less,
    #[token(">")]
    Greater,

    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().to_string())]
    FloatLiteral(String),
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().unwrap())]
    IntLiteral(i64),
    #[regex(r#""([^"\\]|\\[^"])*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    StringLiteral(String),
    #[regex(r"'[^'\\]*(?:\\.[^'\\]*)*'", |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    CharLiteral(String),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    #[regex(r"`[^\r\n]*", allow_greedy = true)]
    LineComment,
}

