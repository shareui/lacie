pub mod cli;
pub mod cmd;
pub mod config;
pub mod diag;
pub mod filter;
pub mod locale;
pub mod model;
pub mod net;
pub mod platform;
pub mod ruby;
pub mod state;
pub mod ui;

use clap::Parser;
use cli::{Cli, Commands};
use config::paths::LaciePaths;
use config::{check_init, load_config};
use diag::{exit_error, DiagError};

fn dispatch_command(cmd: Commands, paths: &LaciePaths, cfg: &config::Config) -> Result<(), DiagError> {
    match cmd {
        Commands::InitLacie => cmd::init::execute_init(paths),
        Commands::Explain { code } => {
            diag::explain::explain_command(&code)
                .map_err(|msg| DiagError::new("E017", msg))
        }
        Commands::Add { args } => cmd::add::execute_add(paths, cfg, &args),
        Commands::Update { args } => cmd::update::execute_update(paths, cfg, &args),
        Commands::Switch { args } => cmd::switch::execute_switch(paths, cfg, &args),
        Commands::Remove { args } => cmd::remove::execute_remove(paths, cfg, &args),
        Commands::Freeze { args } => cmd::freeze::execute_freeze(paths, cfg, &args),
        Commands::Unfreeze { args } => cmd::freeze::execute_unfreeze(paths, cfg, &args),
        Commands::Frozen { args } => cmd::freeze::execute_frozen(paths, cfg, &args),
        Commands::Rename { args } => cmd::rename::execute_rename(paths, cfg, &args),
        Commands::Source { args } => cmd::source::execute_source(paths, cfg, &args),
        Commands::List { args } => cmd::list::execute_list(paths, cfg, &args),
        Commands::Info { args } => cmd::info::execute_info(paths, cfg, &args),
        Commands::Doctor { args } => cmd::doctor::execute_doctor(paths, cfg, &args),
        Commands::Validate { args } => cmd::validate::execute_validate(&args),
        Commands::GenToml => cmd::gen_toml::execute_gen_toml(),
        Commands::Export { args } => cmd::export_import::execute_export(paths, cfg, &args),
        Commands::Import { args } => cmd::export_import::execute_import(paths, cfg, &args),
        Commands::Config { args } => cmd::config::execute_config(paths, cfg, &args),
        Commands::Completions { args } => cmd::completions::execute_completions(&args),
        Commands::Suicide { args } => cmd::suicide::execute_suicide(paths, cfg, &args),
    }
}

fn print_root_info(styler: &ui::color::Styler) {
    let pkg_ver = env!("CARGO_PKG_VERSION");
    println!("{} v{}", styler.bold_cyan("Lacie package manager"), pkg_ver);
    println!("{}: MIT 2026", styler.cyan("License"));
    println!("{}: {}", styler.cyan("Source code"), styler.underline("https://github.com/shareui/lacie"));
}

fn print_version_info(styler: &ui::color::Styler) {
    let pkg_ver = env!("CARGO_PKG_VERSION");
    println!("{} v{}", styler.bold_cyan("Lacie package manager"), pkg_ver);
    println!("Use {} or {} for more", styler.cyan("lacie"), styler.cyan("lacie --help"));
}

fn run_app() -> Result<(), DiagError> {
    let cli = Cli::parse();
    let paths = LaciePaths::resolve();
    let mut cfg = config::Config::default();
    if let Ok(loaded) = load_config(&paths) {
        locale::set_language(&loaded.language);
        cfg = loaded;
    }
    let mode = cfg.colors.parse::<ui::color::ColorMode>().unwrap_or(ui::color::ColorMode::Terminal);
    let styler = ui::color::Styler::new(mode);

    if cli.version {
        print_version_info(&styler);
        return Ok(());
    }

    if cli.command.is_none() {
        print_root_info(&styler);
        return Ok(());
    }

    if !cli::is_exempt(cli.command.as_ref()) && !check_init(&paths) {
        return Err(DiagError::from_code("E001", &[]));
    }

    match cli.command {
        Some(cmd) => dispatch_command(cmd, &paths, &cfg),
        None => Ok(()),
    }
}

fn main() { if let Err(err) = run_app() { exit_error(&err); } }
