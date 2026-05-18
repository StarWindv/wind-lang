use crate::modules::types::cli::Cli;
use crate::modules::types::utils::Utils;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use wind_frontend::lexer::WindToken;
use wind_frontend::parser::WindParser;
use wind_sa::SemanticAnalyzer;

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

    pub fn resolve_file_name(cli: &Cli) -> String {
        cli.path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<cmd>".to_string())
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

    pub fn run_check(source: &str, file_name: &str) {
        let tokens = match WindToken::lex(source) {
            Ok(t) => t,
            Err(errors) => {
                for e in &errors {
                    log::error!("[Lexer] {}", e.message);
                }
                std::process::exit(1);
            }
        };

        let program = match WindParser::parse(source, &tokens) {
            Ok(p) => p,
            Err(errors) => {
                for e in &errors {
                    log::error!("[Parser] {}", e.message);
                }
                std::process::exit(1);
            }
        };

        log::info!("语法分析成功, 共 {} 个顶层条目", program.items.len());

        let result = SemanticAnalyzer::new()
            .with_source(source.to_string())
            .with_file_name(file_name)
            .analyze(&program);

        if result.has_errors() {
            result.emit_diagnostics();
            log::info!("语义分析未通过. {} 个错误", result.all_errors.len());
        } else {
            log::info!("语义分析通过.");
        }

        debug_summary(&result);
    }
}

fn debug_summary(_result: &SemanticAnalyzer) {
    print_symbol_table(_result);
    print_value_pool(_result);
    print_liveness(_result);
}

fn print_symbol_table(result: &SemanticAnalyzer) {
    use log::{debug, info};
    let scopes = &result.ctx.scope_tree.scopes;

    let sym_count: usize = scopes.values().map(|s| s.symbols.len()).sum();
    info!("符号表: {} 个作用域, {} 个符号", scopes.len(), sym_count);

    let mut scope_ids: Vec<_> = scopes.keys().collect();
    scope_ids.sort_by_key(|k| k.get());

    for sid in scope_ids {
        let scope = &scopes[sid];
        let kind = match scope.kind {
            wind_sa::ScopeKind::Global => "Global",
            wind_sa::ScopeKind::Function => "Function",
            wind_sa::ScopeKind::Block => "Block",
        };
        if scope.symbols.is_empty() {
            continue;
        }
        debug!(
            "  [{} scope_id={}] {} 个符号:",
            kind,
            sid.get(),
            scope.symbols.len()
        );
        for (name, sym) in &scope.symbols {
            let sym_kind = match sym {
                wind_sa::Symbol::Variable { storage_class, .. } => {
                    format!("var ({:?})", storage_class)
                }
                wind_sa::Symbol::Function { .. } => "fn".to_string(),
                wind_sa::Symbol::Struct { .. } => "struct".to_string(),
                wind_sa::Symbol::Trait { .. } => "trait".to_string(),
                wind_sa::Symbol::Enum { .. } => "enum".to_string(),
                wind_sa::Symbol::TypeAlias { .. } => "type".to_string(),
                wind_sa::Symbol::Extra { .. } => "extra".to_string(),
                wind_sa::Symbol::Impl { .. } => "impl".to_string(),
                wind_sa::Symbol::Group { .. } => "group".to_string(),
            };
            debug!("    - {} ({})", name, sym_kind);
        }
    }
}

fn print_value_pool(result: &SemanticAnalyzer) {
    use log::debug;
    let pool = &result.ctx.value_pool;
    debug!("Value Pool: {} 个值", pool.values.len());
    for (id, info) in &pool.values {
        let name = result
            .ctx
            .value_names
            .get(id)
            .cloned()
            .unwrap_or_else(|| "<anon>".to_string());
        let kind = format!("{:?}", info.kind);
        let ty = info
            .ty
            .as_ref()
            .map(|t| t.display_name())
            .unwrap_or_else(|| "?".to_string());
        debug!(
            "  - id={} '{}' {} ::{}  refs={}",
            id.get(),
            name,
            kind,
            ty,
            info.ref_count,
        );
    }
}

fn print_liveness(result: &SemanticAnalyzer) {
    use log::{debug, info};
    let drops = &result.drop_points;
    let ranges = &result.live_ranges;

    let scope_drops: Vec<_> = drops
        .iter()
        .filter(|d| {
            ranges
                .iter()
                .any(|r| r.value == d.value && r.dropped_by_scope_exit)
        })
        .collect();
    let early_drops: Vec<_> = drops
        .iter()
        .filter(|d| {
            !ranges
                .iter()
                .any(|r| r.value == d.value && r.dropped_by_scope_exit)
        })
        .collect();

    info!(
        "存活分析: {} 区间, {} Drop ({} scope-exit, {} early)",
        ranges.len(),
        drops.len(),
        scope_drops.len(),
        early_drops.len(),
    );

    if !early_drops.is_empty() {
        debug!("  [提前 Drop (ref_count→0)]:");
        for dp in &early_drops {
            debug!("    - {}", dp.description);
        }
    }
    if !scope_drops.is_empty() {
        debug!("  [作用域退出 Drop]:");
        for dp in &scope_drops {
            debug!("    - {}", dp.description);
        }
    }
}
