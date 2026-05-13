use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;

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

fn main() {
    init_log();
    log::info!("TEST MAIN");
}
