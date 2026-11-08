use crate::diag::DiagError;

fn detect_shell() -> &'static str {
    if let Ok(shell_path) = std::env::var("SHELL") {
        if shell_path.ends_with("/fish") || shell_path == "fish" {
            return "fish";
        }
        if shell_path.ends_with("/zsh") || shell_path == "zsh" {
            return "zsh";
        }
        if shell_path.ends_with("/bash") || shell_path == "bash" {
            return "bash";
        }
    }
    if std::env::var("FISH_VERSION").is_ok() {
        return "fish";
    }
    if std::env::var("ZSH_VERSION").is_ok() {
        return "zsh";
    }
    if std::env::var("PSModulePath").is_ok() {
        return "powershell";
    }
    "bash"
}

fn print_bash_completion() {
    println!(r#"# Bash completion for lacie
# To install, add to ~/.bashrc (or ~/.bash_profile):
#   eval "$(lacie completions bash)"

_lacie_completions() {{
    local cur prev opts
    COMPREPLY=()
    cur="${{COMP_WORDS[COMP_CWORD]}}"
    prev="${{COMP_WORDS[COMP_CWORD-1]}}"
    opts="add update remove switch freeze unfreeze frozen rename source list info doctor validate gen-toml export import config completions suicide explain init-lacie"
    if [[ ${{COMP_CWORD}} -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "${{opts}}" -- "${{cur}}") )
        return 0
    fi
    case "${{prev}}" in
        add)
            COMPREPLY=( $(compgen -W "--show --list --info --max --force --ignore-hash --set-hash --iwannasee --no-cache" -- "${{cur}}") )
            return 0
            ;;
        update)
            COMPREPLY=( $(compgen -W "--ignore --pre --force" -- "${{cur}}") )
            return 0
            ;;
        config)
            COMPREPLY=( $(compgen -W "list get set reset" -- "${{cur}}") )
            return 0
            ;;
        list|info)
            COMPREPLY=( $(compgen -W "--json" -- "${{cur}}") )
            return 0
            ;;
        completions)
            COMPREPLY=( $(compgen -W "bash zsh fish powershell" -- "${{cur}}") )
            return 0
            ;;
    esac
}}
complete -F _lacie_completions lacie"#);
}

fn print_zsh_completion() {
    println!(r#"#compdef lacie
# Zsh completion for lacie
# To install, save as _lacie in a directory in $fpath (e.g. ~/.zfunc/_lacie):
#   lacie completions zsh > ~/.zfunc/_lacie
# and ensure ~/.zshrc contains: fpath=(~/.zfunc $fpath); compinit

_lacie() {{
    local -a commands
    commands=(
        'add:Install package'
        'update:Update packages'
        'remove:Remove package'
        'switch:Switch package version'
        'freeze:Freeze package version'
        'unfreeze:Unfreeze package'
        'frozen:List frozen packages'
        'rename:Rename command'
        'source:Manage sources'
        'list:List installed packages'
        'info:Show package information'
        'doctor:Check environment health'
        'validate:Validate manifest'
        'gen-toml:Generate lacie.toml template'
        'export:Export state'
        'import:Import state'
        'config:Manage configuration'
        'completions:Generate shell completions'
        'suicide:Uninstall lacie completely'
        'explain:Explain error or warning code'
        'init-lacie:Initialize lacie working environment'
    )
    _describe 'commands' commands
}}
_lacie"#);
}

fn print_fish_completion() {
    println!(r#"# Fish completion for lacie
# To install:
#   lacie completions fish > ~/.config/fish/completions/lacie.fish

set -l commands add update remove switch freeze unfreeze frozen rename source list info doctor validate gen-toml export import config completions suicide explain init-lacie
complete -c lacie -f
for cmd in $commands
    complete -c lacie -n "not __fish_seen_subcommand_from $commands" -a $cmd
end

complete -c lacie -s h -l help -d 'Print help'
complete -c lacie -s V -l version -d 'Print version'
complete -c lacie -n '__fish_seen_subcommand_from add' -l show -d 'Show lacie.toml without installing'
complete -c lacie -n '__fish_seen_subcommand_from add' -l list -d 'List available releases'
complete -c lacie -n '__fish_seen_subcommand_from add' -l force -d 'Force reinstall even if hash matches'
complete -c lacie -n '__fish_seen_subcommand_from add' -l iwannasee -d 'Show known files again'
complete -c lacie -n '__fish_seen_subcommand_from update' -l force -d 'Force update'
complete -c lacie -n '__fish_seen_subcommand_from list info' -l json -d 'Output in JSON format'
complete -c lacie -n '__fish_seen_subcommand_from config' -a 'list get set reset'"#);
}

fn print_powershell_completion() {
    println!(r#"# PowerShell completion for lacie
# To install, add to $PROFILE:
#   lacie completions powershell | Out-String | Invoke-Expression

Register-ArgumentCompleter -Native -CommandName lacie -ScriptBlock {{
    param($wordToComplete, $commandAst, $cursorPosition)
    $commands = @('add', 'update', 'remove', 'switch', 'freeze', 'unfreeze', 'frozen', 'rename', 'source', 'list', 'info', 'doctor', 'validate', 'gen-toml', 'export', 'import', 'config', 'completions', 'suicide', 'explain', 'init-lacie')
    $commands | Where-Object {{ $_ -like "$wordToComplete*" }} | ForEach-Object {{
        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
    }}
}}"#);
}

pub fn execute_completions(args: &[String]) -> Result<(), DiagError> {
    let detected = detect_shell();
    let shell = args.first().map(|s| s.as_str()).unwrap_or(detected);
    match shell.to_lowercase().as_str() {
        "bash" => print_bash_completion(),
        "zsh" => print_zsh_completion(),
        "fish" => print_fish_completion(),
        "powershell" => print_powershell_completion(),
        _ => return Err(DiagError::from_code("E004", &[("key", "shell"), ("value", shell), ("allowed", "bash, zsh, fish, powershell")])),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completions_shells() {
        assert!(execute_completions(&["bash".to_string()]).is_ok());
        assert!(execute_completions(&["zsh".to_string()]).is_ok());
        assert!(execute_completions(&["fish".to_string()]).is_ok());
        assert!(execute_completions(&["powershell".to_string()]).is_ok());
        assert!(execute_completions(&["unknown".to_string()]).is_err());
    }
}
