use std::path::PathBuf;

use clap::{ArgGroup, Parser as ClapParser, ValueEnum};
use log::LevelFilter;
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
};

use wind_frontend::lexer::WindToken;
use wind_frontend::parser::WindParser;

#[derive(Debug, Clone, ValueEnum)]
enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Error => LevelFilter::Error,
        }
    }
}

#[derive(ClapParser, Debug)]
#[command(name = "wind", about = "Wind 语言工具链")]
#[command(group(
    ArgGroup::new("mode")
        .args(["lex", "parse"])
        .multiple(false)
))]
struct Cli {
    /// 源代码文件路径 (与 -c 二选一)
    path: Option<PathBuf>,

    /// 直接处理字符串形式的程序
    #[arg(short, long)]
    cmd: Option<String>,

    /// 仅执行词法分析并输出标记
    #[arg(short, long)]
    lex: bool,

    /// 执行词法 + 语法分析并输出 AST
    #[arg(short, long)]
    parse: bool,

    /// 日志级别 (debug/info/warn/error), 默认 info
    #[arg(short = 'L', long, value_enum, default_value = "info")]
    level: LogLevel,
}

fn init_log(level: LevelFilter) {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "[{d(%H:%M:%S)}] [{l:<5}] {m}{n}",
        )))
        .build();

    let config = log4rs::Config::builder()
        .appender(Appender::builder().build("stdout", Box::from(stdout)))
        .build(Root::builder().appender("stdout").build(level))
        .unwrap();

    log4rs::init_config(config).unwrap();
}

fn resolve_source(cli: &Cli) -> String {
    if let Some(ref code) = cli.cmd {
        code.clone()
    } else if let Some(ref path) = cli.path {
        std::fs::read_to_string(path).unwrap_or_else(|e| {
            log::error!("无法读取文件 {:?}: {}", path, e);
            std::process::exit(1);
        })
    } else {
        log::error!("必须提供源文件路径或使用 -c 传入代码");
        std::process::exit(1);
    }
}

fn run_lex(source: &str) {
    match WindToken::lex(source) {
        Ok(tokens) => {
            log::info!("词法分析成功, 共 {} 个标记:", tokens.len());
            for (tok, span) in &tokens {
                log::info!("  {:?} @ {:?}", tok, span);
            }
        }
        Err(errors) => {
            for e in &errors {
                log::error!("{}", e.message);
            }
            std::process::exit(1);
        }
    }
}

fn run_parse(source: &str) {
    let tokens = match WindToken::lex(source) {
        Ok(t) => t,
        Err(errors) => {
            for e in &errors {
                log::error!("{}", e.message);
            }
            std::process::exit(1);
        }
    };

    match WindParser::parse(source, &tokens) {
        Ok(program) => {
            log::info!("语法分析成功, 共 {} 个顶层条目:", program.items.len());
            for (i, item) in program.items.iter().enumerate() {
                log::info!("  [{i}] {:#?}", item);
            }
        }
        Err(errors) => {
            for e in &errors {
                log::error!("{}", e.message);
            }
            std::process::exit(1);
        }
    }
}

fn main() {
    let cli = Cli::parse();
    init_log(cli.level.clone().into());

    let source = resolve_source(&cli);

    if cli.lex {
        run_lex(&source);
    } else if cli.parse {
        run_parse(&source);
    } else {
        log::info!("wind: 暂无后续管线, 仅支持 -l/--lex 和 -p/--parse");
    }
}
