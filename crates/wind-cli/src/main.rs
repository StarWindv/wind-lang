use clap::Parser;
use wind_cli::{Cli, Utils};


fn main() {
    let cli = Cli::parse();
    Utils::init_log(cli.level.clone().into());

    let source = Utils::resolve_source(&cli);

    if cli.lex {
        Utils::run_lex(&source);
    } else if cli.parse {
        Utils::run_parse(&source);
    } else {
        log::info!("Not Implemented");
    }
}
