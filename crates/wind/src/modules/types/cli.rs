use crate::modules::types::log_level::LogLevel;
use clap::{ArgGroup, Parser as ClapParser};
use std::path::PathBuf;

#[derive(ClapParser, Debug)]
#[command(name = "wind", about = "Wind 语言工具链")]
#[command(group(
    ArgGroup::new("mode")
        .args(["lex", "parse"])
        .multiple(false)
))]
pub struct Cli {
    /// 源代码文件路径 (与 -c 二选一)
    pub(crate) path: Option<PathBuf>,

    /// 直接处理字符串形式的程序
    #[arg(short, long)]
    pub(crate) cmd: Option<String>,

    /// 仅执行词法分析并输出标记
    #[arg(short, long)]
    pub lex: bool,

    /// 执行词法 + 语法分析并输出 AST
    #[arg(short, long)]
    pub parse: bool,

    /// 日志级别 (debug/info/warn/error), 默认 info
    #[arg(short = 'L', long, value_enum, default_value = "info")]
    pub level: LogLevel,
}
