use crate::modules::types::cli::Cli;
use crate::modules::types::utils::Utils;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use wind_frontend::lexer::WindToken;
use wind_frontend::parser::WindParser;

impl Utils {
    pub fn init_log(level: LevelFilter) {
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

    pub fn resolve_source(cli: &Cli) -> String {
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

    pub fn run_lex(source: &str) {
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

    pub fn run_parse(source: &str) {
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
}
