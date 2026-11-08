use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "lacie", about = "Package manager for developer tools", disable_version_flag = true)]
pub struct Cli {
    #[arg(short = 'v', short_alias = 'V', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    #[command(name = "init-lacie", about = "Initialize lacie working environment")]
    InitLacie,
    #[command(name = "explain", about = "Explain error or warning code")]
    Explain { code: String, },
    #[command(name = "gen-toml", about = "Generate lacie.toml template")]
    GenToml,
    #[command(name = "validate", about = "Validate manifest")]
    Validate {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "doctor", about = "Check environment health")]
    Doctor {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "completions", about = "Generate shell completions")]
    Completions {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "suicide", about = "Uninstall lacie completely")]
    Suicide {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "export", about = "Export state")]
    Export {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "import", about = "Import state")]
    Import {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "config", about = "Manage configuration")]
    Config {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "add", about = "Install package")]
    Add {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "update", about = "Update packages")]
    Update {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "remove", about = "Remove package")]
    Remove {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "switch", about = "Switch package version")]
    Switch {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "freeze", about = "Freeze package version")]
    Freeze {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "unfreeze", about = "Unfreeze package")]
    Unfreeze {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "frozen", about = "List frozen packages")]
    Frozen {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "rename", about = "Rename command")]
    Rename {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "source", about = "Manage sources")]
    Source {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "list", about = "List installed packages")]
    List {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "info", about = "Show package information")]
    Info {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

pub fn is_exempt(cmd: Option<&Commands>) -> bool {
    matches!(
        cmd,
        None | Some(Commands::InitLacie)
            | Some(Commands::Explain { .. })
            | Some(Commands::GenToml)
            | Some(Commands::Validate { .. })
            | Some(Commands::Doctor { .. })
            | Some(Commands::Completions { .. })
            | Some(Commands::Suicide { .. })
    )
}
