use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log::LevelFilter;

use wind_frontend::modules::implements::tokens::{lex, SpannedToken};
use wind_frontend::modules::implements::ast::parse::parse;

use std::thread;
use wind_frontend::modules::types::ast::Program;

const STACK_SIZE: usize = 8 * 1024 * 1024;

fn init_log() {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "[{d(%H:%M:%S)}] [{l:<5}] {m}{n}",
        )))
        .build();

    let config = log4rs::Config::builder()
        .appender(Appender::builder().build("stdout", Box::from(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Debug))
        .unwrap();

    log4rs::init_config(config).unwrap();
}

fn run_with_stack<F: FnOnce() + Send + 'static>(f: F) {
    thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(f)
        .unwrap()
        .join()
        .unwrap();
}

fn test_lex_simple() {
    let source = "fn main() { return 42; }";
    let result = lex(source);
    log::debug!("lex result is_ok: expected=true, got={}", result.is_ok());
    assert!(result.is_ok(), "lex failed: {:?}", result.err());
    let tokens = result.unwrap();
    log::debug!("tokens.len() > 0: expected=true, got={} (len={})", tokens.len() > 0, tokens.len());
    assert!(tokens.len() > 0, "expected some tokens");
    for (tok, span) in &tokens {
        eprintln!("  {:?} @ {:?}", tok, span);
    }
}

fn test_parse_simple_fn() {
    run_with_stack(|| {
        let source = "fn main() { return 42; }";
        let tokens = lex(source).unwrap();
        let result = parse(&tokens);

        link(source, &tokens, &result.clone().unwrap());
        assert!(result.is_ok(), "parse failed: {:?}", result.err());
        let program = result.unwrap();
        log::debug!("program.items.len() == 1: expected=1, got={}", program.items.len());
        assert_eq!(program.items.len(), 1);
        eprintln!("program: {:#?}", program);
    });
}

fn test_parse_struct() {
    run_with_stack(|| {
        let source = "struct Point { pub x: int; pub y: int; }";
        let tokens = lex(source).unwrap();
        let result = parse(&tokens);
        link(source, &tokens, &result.clone().unwrap());
        assert!(result.is_ok(), "parse failed: {:?}", result.err());
    });
}

fn test_lex_comments() {
    let source = "fn main() { ` this is a comment\n    return 1;\n}";
    let result = lex(source);
    log::debug!("lex result is_ok: expected=true, got={}", result.is_ok());
    assert!(result.is_ok(), "lex failed: {:?}", result.err());
}

fn link(src: &str, tokens: &Vec<SpannedToken>, result: &Program) {
    log::debug!("Inputs: {}", src);
    log::debug!("Tokens: {:#?}", tokens);
    log::debug!("Result: {:#?}", result);
}

fn main() {
    init_log();
    log::debug!("Running tests...");

    test_lex_simple();
    test_parse_simple_fn();
    test_parse_struct();
    test_lex_comments();

    log::info!("All tests passed!");
}
