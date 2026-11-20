use crate::diag::DiagError;

const BASH_SCRIPT: &str = r#"# Bash completion for lacie
# To install, add to ~/.bashrc:
#   eval "$(lacie completions bash)"

_lacie_completions() {
    local cur prev words cword
    _init_completion -n : 2>/dev/null || {
        COMPREPLY=()
        cur="${COMP_WORDS[COMP_CWORD]}"
        prev="${COMP_WORDS[COMP_CWORD-1]}"
        words=("${COMP_WORDS[@]}")
        cword=$COMP_CWORD
    }

    local commands="add update remove switch freeze unfreeze frozen rename source list info doctor validate gen-toml export import config completions suicide explain init-lacie"

    local cmd=""
    local i=1
    while [[ $i -lt $cword ]]; do
        local word="${words[i]}"
        for c in $commands; do
            if [[ "$word" == "$c" ]]; then
                cmd="$c"
                break 2
            fi
        done
        ((i++))
    done

    if [[ -z "$cmd" ]]; then
        if [[ "$cur" == -* ]]; then
            COMPREPLY=( $(compgen -W "-h --help -v -V --version" -- "$cur") )
        else
            COMPREPLY=( $(compgen -W "$commands -h --help -v --version" -- "$cur") )
        fi
        return 0
    fi

    case "$cmd" in
        add)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=( $(compgen -W "-y --yes --show --list --info --force --set-hash --ignore-hash --max --iwannasee --no-cache -b --no-bar -h --help" -- "$cur") )
            fi
            ;;
        update)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=( $(compgen -W "-a --all -y --yes -d --dry-run -i --ignore --pre --no-cache -b --no-bar -h --help" -- "$cur") )
            fi
            ;;
        remove)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=( $(compgen -W "-y --yes --unused --symlink-only --active-only --no-scripts -h --help" -- "$cur") )
            fi
            ;;
        switch)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=( $(compgen -W "-h --help" -- "$cur") )
            fi
            ;;
        freeze|unfreeze)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=( $(compgen -W "-h --help" -- "$cur") )
            else
                COMPREPLY=( $(compgen -W "ALL" -- "$cur") )
            fi
            ;;
        frozen|validate)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=( $(compgen -W "-h --help" -- "$cur") )
            fi
            ;;
        rename)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=( $(compgen -W "-h --help" -- "$cur") )
            else
                COMPREPLY=( $(compgen -W "DEFAULT" -- "$cur") )
            fi
            ;;
        source)
            local subcmds="list add remove update enable disable"
            local sub=""
            local j=$((i + 1))
            while [[ $j -lt $cword ]]; do
                for s in $subcmds; do
                    if [[ "${words[j]}" == "$s" ]]; then
                        sub="$s"
                        break 2
                    fi
                done
                ((j++))
            done
            if [[ -z "$sub" ]]; then
                if [[ "$cur" == -* ]]; then
                    COMPREPLY=( $(compgen -W "-h --help" -- "$cur") )
                else
                    COMPREPLY=( $(compgen -W "$subcmds -h --help" -- "$cur") )
                fi
            else
                case "$sub" in
                    add|remove|update)
                        if [[ "$cur" == -* ]]; then
                            COMPREPLY=( $(compgen -W "-y --yes -h --help" -- "$cur") )
                        fi
                        ;;
                    *)
                        if [[ "$cur" == -* ]]; then
                            COMPREPLY=( $(compgen -W "-h --help" -- "$cur") )
                        fi
                        ;;
                esac
            fi
            ;;
        list)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=( $(compgen -W "--json -h --help" -- "$cur") )
            else
                COMPREPLY=( $(compgen -W "name: command: id: author: link: source: storage: version: number: installed: released: hash: active: frozen: unused:" -- "$cur") )
            fi
            ;;
        info)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=( $(compgen -W "-h --help" -- "$cur") )
            fi
            ;;
        doctor)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=( $(compgen -W "--fix -h --help" -- "$cur") )
            fi
            ;;
        config)
            local cfg_subcmds="list get set reset"
            local cfg_keys="language colors verify_sha256 auto_switch confirm always_show cache_ttl default_source windows_priority linux_priority"
            local sub=""
            local j=$((i + 1))
            while [[ $j -lt $cword ]]; do
                for s in $cfg_subcmds; do
                    if [[ "${words[j]}" == "$s" ]]; then
                        sub="$s"
                        break 2
                    fi
                done
                ((j++))
            done
            if [[ -z "$sub" ]]; then
                if [[ "$cur" == -* ]]; then
                    COMPREPLY=( $(compgen -W "-h --help" -- "$cur") )
                else
                    COMPREPLY=( $(compgen -W "$cfg_subcmds -h --help" -- "$cur") )
                fi
            else
                case "$sub" in
                    get)
                        if [[ "$cur" == -* ]]; then
                            COMPREPLY=( $(compgen -W "-h --help" -- "$cur") )
                        else
                            COMPREPLY=( $(compgen -W "$cfg_keys" -- "$cur") )
                        fi
                        ;;
                    set)
                        if [[ "$prev" == "set" ]]; then
                            COMPREPLY=( $(compgen -W "$cfg_keys" -- "$cur") )
                        elif [[ "$prev" == "language" ]]; then
                            COMPREPLY=( $(compgen -W "auto en ru DEFAULT" -- "$cur") )
                        elif [[ "$prev" == "colors" ]]; then
                            COMPREPLY=( $(compgen -W "terminal always never DEFAULT" -- "$cur") )
                        elif [[ "$prev" == "verify_sha256" || "$prev" == "auto_switch" || "$prev" == "confirm" || "$prev" == "always_show" ]]; then
                            COMPREPLY=( $(compgen -W "true false DEFAULT" -- "$cur") )
                        elif [[ "$prev" == "windows_priority" ]]; then
                            COMPREPLY=( $(compgen -W "msvc mingw DEFAULT" -- "$cur") )
                        elif [[ "$prev" == "linux_priority" ]]; then
                            COMPREPLY=( $(compgen -W "glibc musl DEFAULT" -- "$cur") )
                        else
                            COMPREPLY=( $(compgen -W "DEFAULT" -- "$cur") )
                        fi
                        ;;
                    reset)
                        if [[ "$cur" == -* ]]; then
                            COMPREPLY=( $(compgen -W "-y --yes -h --help" -- "$cur") )
                        fi
                        ;;
                    *)
                        if [[ "$cur" == -* ]]; then
                            COMPREPLY=( $(compgen -W "-h --help" -- "$cur") )
                        fi
                        ;;
                esac
            fi
            ;;
        export)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=( $(compgen -W "--all-versions --with-config -h --help" -- "$cur") )
            fi
            ;;
        import)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=( $(compgen -W "-h --help" -- "$cur") )
            fi
            ;;
        gen-toml)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=( $(compgen -W "-y --yes -h --help" -- "$cur") )
            fi
            ;;
        completions)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=( $(compgen -W "-h --help" -- "$cur") )
            else
                COMPREPLY=( $(compgen -W "bash zsh fish powershell" -- "$cur") )
            fi
            ;;
        suicide)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=( $(compgen -W "-y --yes -h --help" -- "$cur") )
            fi
            ;;
        explain)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=( $(compgen -W "-h --help" -- "$cur") )
            fi
            ;;
        init-lacie)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=( $(compgen -W "--recfg -h --help" -- "$cur") )
            else
                COMPREPLY=( $(compgen -W "1 2 3 4 5 lang color verify path switch" -- "$cur") )
            fi
            ;;
        *)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=( $(compgen -W "-h --help" -- "$cur") )
            fi
            ;;
    esac
    return 0
}
complete -F _lacie_completions lacie
"#;

const ZSH_SCRIPT: &str = r#"#compdef lacie
# Zsh completion for lacie
# To install, save as _lacie in a directory in $fpath:
#   lacie completions zsh > ~/.zfunc/_lacie

_lacie() {
    local curcontext="$curcontext" state line
    typeset -A opt_args

    _arguments -C \
        '(-v -V --version)'{-v,-V,--version}'[Print version information]' \
        '(-h --help)'{-h,--help}'[Print help information]' \
        '1: :->command' \
        '*:: :->args'

    case $state in
        command)
            local -a commands
            commands=(
                'add:Install package from remote source, URL or local file'
                'update:Check and apply package updates'
                'remove:Remove package or clean up inactive versions'
                'switch:Switch active package version'
                'freeze:Freeze package version to prevent updates'
                'unfreeze:Unfreeze package to re-enable updates'
                'frozen:List all frozen packages'
                'rename:Rename executable command in active directory'
                'source:Manage package sources and plugins'
                'list:List installed packages'
                'info:Show detailed package metadata'
                'doctor:Check environment and state health'
                'config:Inspect and modify configuration settings'
                'export:Export package state to lockfile'
                'import:Import package state from lockfile'
                'validate:Validate lacie.toml manifest'
                'gen-toml:Generate starter lacie.toml template'
                'completions:Generate shell autocompletions'
                'explain:Explain error or warning code'
                'suicide:Uninstall lacie completely'
                'init-lacie:Initialize lacie working environment'
            )
            _describe -t commands 'lacie command' commands
            ;;
        args)
            case $line[1] in
                add)
                    _arguments \
                        '(-y --yes)'{-y,--yes}'[Skip confirmation prompt]' \
                        '--show[Open interactive file viewer before approving]' \
                        '--list[List available releases]' \
                        '--info[Show package metadata without installing]' \
                        '--force[Force reinstall even if version exists]' \
                        '--set-hash[Explicit expected SHA256 hash]:sha256:' \
                        '--ignore-hash[Skip SHA256 integrity verification]' \
                        '--max[Limit search results]:count:' \
                        '--iwannasee[Show known files again]' \
                        '--no-cache[Bypass release list cache]' \
                        '(-b --no-bar)'{-b,--no-bar}'[Suppress progress bar]' \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '*:package or file:_files'
                    ;;
                update)
                    _arguments \
                        '(-a --all)'{-a,--all}'[Update all installed packages]' \
                        '(-y --yes)'{-y,--yes}'[Apply updates without confirmation]' \
                        '(-d --dry-run)'{-d,--dry-run}'[Check for updates without downloading]' \
                        '(-i --ignore)'{-i,--ignore}'[Exclude package from update]:package:' \
                        '--pre[Include pre-release versions]' \
                        '--no-cache[Bypass local release cache]' \
                        '(-b --no-bar)'{-b,--no-bar}'[Suppress progress bar]' \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '*:package:'
                    ;;
                remove)
                    _arguments \
                        '(-y --yes)'{-y,--yes}'[Skip confirmation prompt]' \
                        '--unused[Remove only inactive cached versions]' \
                        '--symlink-only[Remove active command symlink only]' \
                        '--active-only[Remove currently active version only]' \
                        '--no-scripts[Skip running removal scripts]' \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '*:package:'
                    ;;
                switch)
                    _arguments \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '1:package:' \
                        '2:version:'
                    ;;
                freeze)
                    _arguments \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '1:package:(ALL)'
                    ;;
                unfreeze)
                    _arguments \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '1:package:(ALL)'
                    ;;
                frozen)
                    _arguments \
                        '(-h --help)'{-h,--help}'[Print help]'
                    ;;
                rename)
                    _arguments \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '1:package:' \
                        '2:new command name:(DEFAULT)'
                    ;;
                source)
                    local -a subcommands
                    subcommands=(
                        'list:List registered sources'
                        'add:Add new source plugin'
                        'remove:Remove source plugin'
                        'update:Update source plugins'
                        'enable:Enable source plugin'
                        'disable:Disable source plugin'
                    )
                    _arguments -C \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '1: :->source_cmd' \
                        '*:: :->source_args'
                    case $state in
                        source_cmd)
                            _describe -t subcommands 'source subcommand' subcommands
                            ;;
                        source_args)
                            case $line[1] in
                                add|remove|update)
                                    _arguments \
                                        '(-y --yes)'{-y,--yes}'[Skip confirmation prompt]' \
                                        '(-h --help)'{-h,--help}'[Print help]'
                                    ;;
                                *)
                                    _arguments '(-h --help)'{-h,--help}'[Print help]'
                                    ;;
                            esac
                            ;;
                    esac
                    ;;
                list)
                    _arguments \
                        '--json[Output in JSON format]' \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '*:filter:'
                    ;;
                info)
                    _arguments \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '*:package:'
                    ;;
                doctor)
                    _arguments \
                        '--fix[Automatically repair detected issues]' \
                        '(-h --help)'{-h,--help}'[Print help]'
                    ;;
                config)
                    local -a cfg_subcommands
                    cfg_subcommands=(
                        'list:List all configuration options'
                        'get:Get value of a setting'
                        'set:Set value of a setting'
                        'reset:Reset all settings to defaults'
                    )
                    local -a cfg_keys
                    cfg_keys=(
                        'language:UI language (auto, en, ru)'
                        'colors:Terminal colors (terminal, always, never)'
                        'verify_sha256:Strict checksum verification (true, false)'
                        'auto_switch:Activate newly installed version (true, false)'
                        'confirm:Prompt before operations (true, false)'
                        'always_show:Always inspect manifest before install (true, false)'
                        'cache_ttl:Release cache TTL in minutes'
                        'default_source:Default source plugin (e.g. github)'
                        'windows_priority:Windows ABI preference (msvc, mingw)'
                        'linux_priority:Linux libc preference (glibc, musl)'
                    )
                    _arguments -C \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '1: :->cfg_cmd' \
                        '*:: :->cfg_args'
                    case $state in
                        cfg_cmd)
                            _describe -t cfg_subcommands 'config subcommand' cfg_subcommands
                            ;;
                        cfg_args)
                            case $line[1] in
                                get)
                                    _describe -t cfg_keys 'config key' cfg_keys
                                    ;;
                                set)
                                    _arguments \
                                        '1:config key:->cfg_set_key' \
                                        '2:value:->cfg_set_val'
                                    case $state in
                                        cfg_set_key)
                                            _describe -t cfg_keys 'config key' cfg_keys
                                            ;;
                                        cfg_set_val)
                                            _values 'value' 'DEFAULT'
                                            ;;
                                    esac
                                    ;;
                                reset)
                                    _arguments \
                                        '(-y --yes)'{-y,--yes}'[Skip confirmation prompt]' \
                                        '(-h --help)'{-h,--help}'[Print help]'
                                    ;;
                                *)
                                    _arguments '(-h --help)'{-h,--help}'[Print help]'
                                    ;;
                            esac
                            ;;
                    esac
                    ;;
                export)
                    _arguments \
                        '--all-versions[Include all downloaded versions]' \
                        '--with-config[Include user configuration]' \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '1:file:_files'
                    ;;
                import)
                    _arguments \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '1:file:_files'
                    ;;
                validate)
                    _arguments \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '1:manifest:_files -g "*.toml"'
                    ;;
                gen-toml)
                    _arguments \
                        '(-y --yes)'{-y,--yes}'[Overwrite existing lacie.toml]' \
                        '(-h --help)'{-h,--help}'[Print help]'
                    ;;
                completions)
                    _arguments \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '1:shell:(bash zsh fish powershell)'
                    ;;
                explain)
                    _arguments \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '1:code:'
                    ;;
                suicide)
                    _arguments \
                        '(-y --yes)'{-y,--yes}'[Skip confirmation prompt]' \
                        '(-h --help)'{-h,--help}'[Print help]'
                    ;;
                init-lacie)
                    _arguments \
                        '--recfg[Re-ask already configured questions]' \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '1:question:(1 2 3 4 5 lang color verify path switch)'
                    ;;
            esac
            ;;
    esac
}

_lacie "$@"
"#;

const FISH_SCRIPT: &str = r#"# Fish completion for lacie
# To install:
#   lacie completions fish > ~/.config/fish/completions/lacie.fish

set -l commands add update remove switch freeze unfreeze frozen rename source list info doctor validate gen-toml export import config completions suicide explain init-lacie
complete -c lacie -f

# Global options
complete -c lacie -s v -s V -l version -d 'Print version'
complete -c lacie -s h -l help -d 'Print help'

# Top-level commands
complete -c lacie -n "not __fish_seen_subcommand_from $commands" -a add -d 'Install package from remote source, URL or local file'
complete -c lacie -n "not __fish_seen_subcommand_from $commands" -a update -d 'Check and apply package updates'
complete -c lacie -n "not __fish_seen_subcommand_from $commands" -a remove -d 'Remove package or clean up inactive versions'
complete -c lacie -n "not __fish_seen_subcommand_from $commands" -a switch -d 'Switch active package version'
complete -c lacie -n "not __fish_seen_subcommand_from $commands" -a freeze -d 'Freeze package version to prevent updates'
complete -c lacie -n "not __fish_seen_subcommand_from $commands" -a unfreeze -d 'Unfreeze package to re-enable updates'
complete -c lacie -n "not __fish_seen_subcommand_from $commands" -a frozen -d 'List all frozen packages'
complete -c lacie -n "not __fish_seen_subcommand_from $commands" -a rename -d 'Rename executable command in active directory'
complete -c lacie -n "not __fish_seen_subcommand_from $commands" -a source -d 'Manage package sources and plugins'
complete -c lacie -n "not __fish_seen_subcommand_from $commands" -a list -d 'List installed packages'
complete -c lacie -n "not __fish_seen_subcommand_from $commands" -a info -d 'Show detailed package metadata'
complete -c lacie -n "not __fish_seen_subcommand_from $commands" -a doctor -d 'Check environment and state health'
complete -c lacie -n "not __fish_seen_subcommand_from $commands" -a config -d 'Inspect and modify configuration settings'
complete -c lacie -n "not __fish_seen_subcommand_from $commands" -a export -d 'Export package state to lockfile'
complete -c lacie -n "not __fish_seen_subcommand_from $commands" -a import -d 'Import package state from lockfile'
complete -c lacie -n "not __fish_seen_subcommand_from $commands" -a validate -d 'Validate lacie.toml manifest'
complete -c lacie -n "not __fish_seen_subcommand_from $commands" -a gen-toml -d 'Generate starter lacie.toml template'
complete -c lacie -n "not __fish_seen_subcommand_from $commands" -a completions -d 'Generate shell autocompletions'
complete -c lacie -n "not __fish_seen_subcommand_from $commands" -a explain -d 'Explain error or warning code'
complete -c lacie -n "not __fish_seen_subcommand_from $commands" -a suicide -d 'Uninstall lacie completely'
complete -c lacie -n "not __fish_seen_subcommand_from $commands" -a init-lacie -d 'Initialize lacie working environment'

# add
complete -c lacie -n '__fish_seen_subcommand_from add' -s y -l yes -d 'Skip confirmation prompt'
complete -c lacie -n '__fish_seen_subcommand_from add' -l show -d 'Open interactive file viewer before approving'
complete -c lacie -n '__fish_seen_subcommand_from add' -l list -d 'List available releases'
complete -c lacie -n '__fish_seen_subcommand_from add' -l info -d 'Show package metadata without installing'
complete -c lacie -n '__fish_seen_subcommand_from add' -l force -d 'Force reinstall even if version exists'
complete -c lacie -n '__fish_seen_subcommand_from add' -l set-hash -d 'Explicit expected SHA256 hash' -r
complete -c lacie -n '__fish_seen_subcommand_from add' -l ignore-hash -d 'Skip SHA256 integrity verification'
complete -c lacie -n '__fish_seen_subcommand_from add' -l max -d 'Limit search results' -r
complete -c lacie -n '__fish_seen_subcommand_from add' -l iwannasee -d 'Show known files again'
complete -c lacie -n '__fish_seen_subcommand_from add' -l no-cache -d 'Bypass release list cache'
complete -c lacie -n '__fish_seen_subcommand_from add' -s b -l no-bar -d 'Suppress progress bar'
complete -c lacie -n '__fish_seen_subcommand_from add' -s h -l help -d 'Print help'

# update
complete -c lacie -n '__fish_seen_subcommand_from update' -s a -l all -d 'Update all installed packages'
complete -c lacie -n '__fish_seen_subcommand_from update' -s y -l yes -d 'Apply updates without confirmation'
complete -c lacie -n '__fish_seen_subcommand_from update' -s d -l dry-run -d 'Check for updates without downloading'
complete -c lacie -n '__fish_seen_subcommand_from update' -s i -l ignore -d 'Exclude package from update' -r
complete -c lacie -n '__fish_seen_subcommand_from update' -l pre -d 'Include pre-release versions'
complete -c lacie -n '__fish_seen_subcommand_from update' -l no-cache -d 'Bypass local release cache'
complete -c lacie -n '__fish_seen_subcommand_from update' -s b -l no-bar -d 'Suppress progress bar'
complete -c lacie -n '__fish_seen_subcommand_from update' -s h -l help -d 'Print help'

# remove
complete -c lacie -n '__fish_seen_subcommand_from remove' -s y -l yes -d 'Skip confirmation prompt'
complete -c lacie -n '__fish_seen_subcommand_from remove' -l unused -d 'Remove only inactive cached versions'
complete -c lacie -n '__fish_seen_subcommand_from remove' -l symlink-only -d 'Remove active command symlink only'
complete -c lacie -n '__fish_seen_subcommand_from remove' -l active-only -d 'Remove currently active version only'
complete -c lacie -n '__fish_seen_subcommand_from remove' -l no-scripts -d 'Skip running removal scripts'
complete -c lacie -n '__fish_seen_subcommand_from remove' -s h -l help -d 'Print help'

# switch
complete -c lacie -n '__fish_seen_subcommand_from switch' -s h -l help -d 'Print help'

# freeze / unfreeze / frozen
complete -c lacie -n '__fish_seen_subcommand_from freeze' -a ALL -d 'Freeze all packages'
complete -c lacie -n '__fish_seen_subcommand_from freeze' -s h -l help -d 'Print help'
complete -c lacie -n '__fish_seen_subcommand_from unfreeze' -a ALL -d 'Unfreeze all packages'
complete -c lacie -n '__fish_seen_subcommand_from unfreeze' -s h -l help -d 'Print help'
complete -c lacie -n '__fish_seen_subcommand_from frozen' -s h -l help -d 'Print help'

# rename
complete -c lacie -n '__fish_seen_subcommand_from rename' -a DEFAULT -d 'Reset to default command name'
complete -c lacie -n '__fish_seen_subcommand_from rename' -s h -l help -d 'Print help'

# source
set -l src_cmds list add remove update enable disable
complete -c lacie -n '__fish_seen_subcommand_from source; and not __fish_seen_subcommand_from $src_cmds' -a list -d 'List registered sources'
complete -c lacie -n '__fish_seen_subcommand_from source; and not __fish_seen_subcommand_from $src_cmds' -a add -d 'Add new source plugin'
complete -c lacie -n '__fish_seen_subcommand_from source; and not __fish_seen_subcommand_from $src_cmds' -a remove -d 'Remove source plugin'
complete -c lacie -n '__fish_seen_subcommand_from source; and not __fish_seen_subcommand_from $src_cmds' -a update -d 'Update source plugins'
complete -c lacie -n '__fish_seen_subcommand_from source; and not __fish_seen_subcommand_from $src_cmds' -a enable -d 'Enable source plugin'
complete -c lacie -n '__fish_seen_subcommand_from source; and not __fish_seen_subcommand_from $src_cmds' -a disable -d 'Disable source plugin'
complete -c lacie -n '__fish_seen_subcommand_from source' -s y -l yes -d 'Skip confirmation prompt'
complete -c lacie -n '__fish_seen_subcommand_from source' -s h -l help -d 'Print help'

# list
complete -c lacie -n '__fish_seen_subcommand_from list' -l json -d 'Output in JSON format'
complete -c lacie -n '__fish_seen_subcommand_from list' -s h -l help -d 'Print help'

# info
complete -c lacie -n '__fish_seen_subcommand_from info' -s h -l help -d 'Print help'

# doctor
complete -c lacie -n '__fish_seen_subcommand_from doctor' -l fix -d 'Automatically repair detected issues'
complete -c lacie -n '__fish_seen_subcommand_from doctor' -s h -l help -d 'Print help'

# config
set -l cfg_cmds list get set reset
set -l cfg_keys language colors verify_sha256 auto_switch confirm always_show cache_ttl default_source windows_priority linux_priority
complete -c lacie -n '__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from $cfg_cmds' -a list -d 'List all configuration options'
complete -c lacie -n '__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from $cfg_cmds' -a get -d 'Get value of a setting'
complete -c lacie -n '__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from $cfg_cmds' -a set -d 'Set value of a setting'
complete -c lacie -n '__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from $cfg_cmds' -a reset -d 'Reset all settings to defaults'
complete -c lacie -n '__fish_seen_subcommand_from config; and __fish_seen_subcommand_from get set' -a "$cfg_keys"
complete -c lacie -n '__fish_seen_subcommand_from config' -s y -l yes -d 'Skip confirmation prompt'
complete -c lacie -n '__fish_seen_subcommand_from config' -s h -l help -d 'Print help'

# export / import
complete -c lacie -n '__fish_seen_subcommand_from export' -l all-versions -d 'Include all downloaded versions'
complete -c lacie -n '__fish_seen_subcommand_from export' -l with-config -d 'Include user configuration'
complete -c lacie -n '__fish_seen_subcommand_from export' -s h -l help -d 'Print help'
complete -c lacie -n '__fish_seen_subcommand_from import' -s h -l help -d 'Print help'

# validate / gen-toml
complete -c lacie -n '__fish_seen_subcommand_from validate' -s h -l help -d 'Print help'
complete -c lacie -n '__fish_seen_subcommand_from gen-toml' -s y -l yes -d 'Overwrite existing lacie.toml'
complete -c lacie -n '__fish_seen_subcommand_from gen-toml' -s h -l help -d 'Print help'

# completions
complete -c lacie -n '__fish_seen_subcommand_from completions' -a 'bash zsh fish powershell'
complete -c lacie -n '__fish_seen_subcommand_from completions' -s h -l help -d 'Print help'

# explain / suicide / init-lacie
complete -c lacie -n '__fish_seen_subcommand_from explain' -s h -l help -d 'Print help'
complete -c lacie -n '__fish_seen_subcommand_from suicide' -s y -l yes -d 'Skip confirmation prompt'
complete -c lacie -n '__fish_seen_subcommand_from suicide' -s h -l help -d 'Print help'
complete -c lacie -n '__fish_seen_subcommand_from init-lacie' -l recfg -d 'Re-ask already configured questions'
complete -c lacie -n '__fish_seen_subcommand_from init-lacie' -a '1 2 3 4 5 lang color verify path switch'
complete -c lacie -n '__fish_seen_subcommand_from init-lacie' -s h -l help -d 'Print help'
"#;

const POWERSHELL_SCRIPT: &str = r#"# PowerShell completion for lacie
# To install, add to $PROFILE:
#   lacie completions powershell | Out-String | Invoke-Expression

Register-ArgumentCompleter -Native -CommandName lacie -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $elements = $commandAst.Elements
    $count = $elements.Count
    $subcommand = $null

    $commands = @{
        'add'         = 'Install package from remote source, URL or local file'
        'update'      = 'Check and apply package updates'
        'remove'      = 'Remove package or clean up inactive versions'
        'switch'      = 'Switch active package version'
        'freeze'      = 'Freeze package version to prevent updates'
        'unfreeze'    = 'Unfreeze package to re-enable updates'
        'frozen'      = 'List all frozen packages'
        'rename'      = 'Rename executable command in active directory'
        'source'      = 'Manage package sources and plugins'
        'list'        = 'List installed packages'
        'info'        = 'Show detailed package metadata'
        'doctor'      = 'Check environment and state health'
        'config'      = 'Inspect and modify configuration settings'
        'export'      = 'Export package state to lockfile'
        'import'      = 'Import package state from lockfile'
        'validate'    = 'Validate lacie.toml manifest'
        'gen-toml'    = 'Generate starter lacie.toml template'
        'completions' = 'Generate shell autocompletions'
        'explain'     = 'Explain error or warning code'
        'suicide'     = 'Uninstall lacie completely'
        'init-lacie'  = 'Initialize lacie working environment'
    }

    for ($i = 1; $i -lt $count; $i++) {
        $val = $elements[$i].Value
        if ($commands.ContainsKey($val)) {
            $subcommand = $val
            break
        }
    }

    $candidates = @()

    if (-not $subcommand) {
        if ($wordToComplete -like '-*') {
            $candidates = @('-h', '--help', '-v', '-V', '--version')
        } else {
            $candidates = $commands.Keys
        }
    } else {
        switch ($subcommand) {
            'add' {
                $candidates = @('-y', '--yes', '--show', '--list', '--info', '--force', '--set-hash', '--ignore-hash', '--max', '--iwannasee', '--no-cache', '-b', '--no-bar', '-h', '--help')
            }
            'update' {
                $candidates = @('-a', '--all', '-y', '--yes', '-d', '--dry-run', '-i', '--ignore', '--pre', '--no-cache', '-b', '--no-bar', '-h', '--help')
            }
            'remove' {
                $candidates = @('-y', '--yes', '--unused', '--symlink-only', '--active-only', '--no-scripts', '-h', '--help')
            }
            'switch' {
                $candidates = @('-h', '--help')
            }
            'freeze' {
                $candidates = @('ALL', '-h', '--help')
            }
            'unfreeze' {
                $candidates = @('ALL', '-h', '--help')
            }
            'frozen' {
                $candidates = @('-h', '--help')
            }
            'rename' {
                $candidates = @('DEFAULT', '-h', '--help')
            }
            'source' {
                if ($wordToComplete -like '-*') {
                    $candidates = @('-y', '--yes', '-h', '--help')
                } else {
                    $candidates = @('list', 'add', 'remove', 'update', 'enable', 'disable', '-y', '--yes', '-h', '--help')
                }
            }
            'list' {
                $candidates = @('--json', '-h', '--help')
            }
            'info' {
                $candidates = @('-h', '--help')
            }
            'doctor' {
                $candidates = @('--fix', '-h', '--help')
            }
            'config' {
                $cfgKeys = @('language', 'colors', 'verify_sha256', 'auto_switch', 'confirm', 'always_show', 'cache_ttl', 'default_source', 'windows_priority', 'linux_priority')
                if ($wordToComplete -like '-*') {
                    $candidates = @('-y', '--yes', '-h', '--help')
                } else {
                    $candidates = @('list', 'get', 'set', 'reset', '-y', '--yes', '-h', '--help') + $cfgKeys
                }
            }
            'export' {
                $candidates = @('--all-versions', '--with-config', '-h', '--help')
            }
            'import' {
                $candidates = @('-h', '--help')
            }
            'validate' {
                $candidates = @('-h', '--help')
            }
            'gen-toml' {
                $candidates = @('-y', '--yes', '-h', '--help')
            }
            'completions' {
                $candidates = @('bash', 'zsh', 'fish', 'powershell', '-h', '--help')
            }
            'explain' {
                $candidates = @('-h', '--help')
            }
            'suicide' {
                $candidates = @('-y', '--yes', '-h', '--help')
            }
            'init-lacie' {
                $candidates = @('--recfg', '1', '2', '3', '4', '5', 'lang', 'color', 'verify', 'path', 'switch', '-h', '--help')
            }
        }
    }

    $candidates | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
        $desc = if ($commands.ContainsKey($_)) { $commands[$_] } else { $_ }
        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $desc)
    }
}
"#;

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
    print!("{}", BASH_SCRIPT);
}

fn print_zsh_completion() {
    print!("{}", ZSH_SCRIPT);
}

fn print_fish_completion() {
    print!("{}", FISH_SCRIPT);
}

fn print_powershell_completion() {
    print!("{}", POWERSHELL_SCRIPT);
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

    #[test]
    fn test_all_commands_and_flags_present() {
        let expected_cmds = [
            "add", "update", "remove", "switch", "freeze", "unfreeze", "frozen",
            "rename", "source", "list", "info", "doctor", "validate", "gen-toml",
            "export", "import", "config", "completions", "suicide", "explain", "init-lacie"
        ];
        for cmd in expected_cmds {
            assert!(BASH_SCRIPT.contains(cmd), "Bash missing {}", cmd);
            assert!(ZSH_SCRIPT.contains(cmd), "Zsh missing {}", cmd);
            assert!(FISH_SCRIPT.contains(cmd), "Fish missing {}", cmd);
            assert!(POWERSHELL_SCRIPT.contains(cmd), "PowerShell missing {}", cmd);
        }

        let key_flags = ["recfg", "no-bar", "dry-run", "unused", "symlink-only", "fix", "with-config"];
        for flag in key_flags {
            let dflag = std::format!("--{}", flag);
            assert!(BASH_SCRIPT.contains(&dflag), "Bash missing flag {}", dflag);
            assert!(ZSH_SCRIPT.contains(&dflag), "Zsh missing flag {}", dflag);
            let fish_flag = std::format!("-l {}", flag);
            assert!(FISH_SCRIPT.contains(&fish_flag), "Fish missing flag {}", fish_flag);
            assert!(POWERSHELL_SCRIPT.contains(&dflag), "PowerShell missing flag {}", dflag);
        }
    }
}
