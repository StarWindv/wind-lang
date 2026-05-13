use logos::Logos;


pub type WindSpan = std::ops::Range<usize>;

pub type WindSpannedToken = (WindToken, WindSpan);

pub fn byte_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let offset = offset.min(source.len());
    let line = source[..offset].matches('\n').count() + 1;
    let last_nl = source[..offset].rfind('\n').map(|p| p + 1).unwrap_or(0);
    let col = offset - last_nl + 1;
    (line, col)
}


#[derive(Logos, Debug, Clone, PartialEq, Eq, Hash)]
#[logos(
    skip r"[ \t\n\r\f]+",
    skip r"\\[ \t]*\n",
    skip r"```[\s\S]*?```"
)]
pub enum WindToken {
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
    #[token("explain")]
    Explain,
    #[token("static")]
    Static,
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

    #[token("lambda")]
    Lambda,
    #[token("from")]
    From,
    #[token("import")]
    Import,
    #[token("use")]
    Use,
    #[token("as")]
    As,
    #[token("when")]
    When,
    #[token("define")]
    Define,
    #[token("guard")]
    Guard,
    #[token("protect")]
    Protect,

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
    #[token("===")]
    AddrEqual,
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
    #[token("+=")]
    PlusEqual,
    #[token("-=")]
    MinusEqual,
    #[token("*=")]
    StarEqual,
    #[token("/=")]
    SlashEqual,
    #[token("++")]
    PlusPlus,
    #[token("--")]
    MinusMinus,
    #[token("..")]
    DotDot,

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
    #[regex(r#""([^"\\]|\\"|\\[^"])*""#, |lex| {
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

