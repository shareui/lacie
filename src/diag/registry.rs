pub struct DiagItem {
    pub code: &'static str,
    pub title: &'static str,
    pub summary_template: &'static str,
    pub description: &'static str,
    pub solutions: &'static [&'static str],
}

pub static ERRORS: &[DiagItem] = &[
    DiagItem {
        code: "E001",
        title: "Configuration not initialized",
        summary_template: "Configuration not initialized, use `lacie init-lacie`",
        description: "The configuration file is missing, unreadable, or its `init` flag is not set to true.",
        solutions: &[
            "Run `lacie init-lacie`",
            "If LACIE_HOME is set, check that it points to the right directory",
        ],
    },
    DiagItem {
        code: "E002",
        title: "Interactive terminal required",
        summary_template: "Interactive terminal required, run `lacie {command}` in a terminal",
        description: "The command requires an interactive user terminal to ask for confirmations or inputs.",
        solutions: &[
            "Run the command directly in a terminal instead of a pipe or non-interactive script",
        ],
    },
    DiagItem {
        code: "E003",
        title: "Unknown config key",
        summary_template: "Unknown config key `{key}`, use `lacie config list`",
        description: "The specified configuration key is not recognized by lacie.",
        solutions: &[
            "Run `lacie config list` to inspect all valid configuration parameters",
        ],
    },
    DiagItem {
        code: "E004",
        title: "Invalid config value",
        summary_template: "Invalid value `{value}` for `{key}`, allowed: {allowed}",
        description: "The provided value is not supported for this configuration parameter.",
        solutions: &[
            "Provide one of the allowed values indicated in the error message",
        ],
    },
    DiagItem {
        code: "E005",
        title: "Release not found",
        summary_template: "Release not found for {ref}, use `lacie add {ref} --list`",
        description: "The specified package or release reference was not found in the source repository.",
        solutions: &[
            "Run `lacie add {ref} --list` to see available releases and tags",
        ],
    },
    DiagItem {
        code: "E006",
        title: "No binary for target",
        summary_template: "No binary for {package} for target {target} in release {tag}",
        description: "The release exists, but does not provide a binary compatible with your system target.",
        solutions: &[
            "Check if an alternative release or source build is available",
        ],
    },
    DiagItem {
        code: "E007",
        title: "Checksum mismatch",
        summary_template: "sha256 mismatch for {file}, nothing was installed",
        description: "The computed SHA256 checksum does not match the expected hash from manifest or flags.",
        solutions: &[
            "Verify the download source or pass `--set-hash` or `--ignore-hash` if intentional",
        ],
    },
    DiagItem {
        code: "E008",
        title: "Unknown or disabled source",
        summary_template: "Unknown or disabled source `{source}`",
        description: "The specified source plugin is either not installed or currently disabled.",
        solutions: &[
            "Run `lacie source list` to view sources and `lacie source enable {source}` to enable it",
        ],
    },
    DiagItem {
        code: "E009",
        title: "Package not installed",
        summary_template: "Package `{name}` is not installed",
        description: "The package requested for operation is not present in packages.toml.",
        solutions: &[
            "Run `lacie list` to check installed packages or install it via `lacie add`",
        ],
    },
    DiagItem {
        code: "E010",
        title: "Package version not installed",
        summary_template: "Version `{version}` of `{name}` is not installed, installed: {versions}",
        description: "The requested version has not been downloaded or installed for this package.",
        solutions: &[
            "Choose one of the installed versions or install the version with `lacie add`",
        ],
    },
    DiagItem {
        code: "E011",
        title: "Invalid switch arguments",
        summary_template: "`lacie switch` expects pairs of package and version",
        description: "Arguments to `lacie switch` must be supplied in pairs of package name and version.",
        solutions: &[
            "Run `lacie switch <package> <version>`",
        ],
    },
    DiagItem {
        code: "E012",
        title: "Ambiguous package name",
        summary_template: "Ambiguous package name `{name}`, use one of the ids: {ids}",
        description: "Multiple installed packages share this short command name.",
        solutions: &[
            "Specify the package using its full domain identifier (id)",
        ],
    },
    DiagItem {
        code: "E013",
        title: "Missing manifest",
        summary_template: "No lacie.toml found in {path}",
        description: "The archive or directory does not contain the mandatory `lacie.toml` manifest.",
        solutions: &[
            "Ensure the package repository or archive includes a valid `lacie.toml`",
        ],
    },
    DiagItem {
        code: "E014",
        title: "Path or link not found",
        summary_template: "Path or link not found, or unsupported format: {ref}",
        description: "The provided local path or remote URL cannot be resolved or is an unsupported archive type.",
        solutions: &[
            "Verify that the file exists and is one of .zip, .tar.gz, .tar.xz, or .tar.zst",
        ],
    },
    DiagItem {
        code: "E015",
        title: "File busy or locked",
        summary_template: "Could not move {file}, it may be in use by another program, nothing was changed",
        description: "Operating system denied moving the file, usually because an active process is locking it.",
        solutions: &[
            "Close any running instances of the program and retry",
        ],
    },
    DiagItem {
        code: "E016",
        title: "File cannot be removed",
        summary_template: "Could not remove {file}, it may be in use by another program, remaining files are kept",
        description: "File could not be deleted during uninstallation due to file lock or permissions.",
        solutions: &[
            "Terminate processes holding the file and run `lacie remove` again",
        ],
    },
    DiagItem {
        code: "E017",
        title: "Wrong number of arguments",
        summary_template: "Wrong number of arguments, expected {expected}",
        description: "The command was invoked with too few or too many arguments.",
        solutions: &[
            "Consult `lacie --help` or `lacie <command> --help`",
        ],
    },
    DiagItem {
        code: "E018",
        title: "Incompatible flags",
        summary_template: "Flags {a} and {b} cannot be combined",
        description: "The specified CLI options are mutually exclusive.",
        solutions: &[
            "Use only one of the conflicting flags",
        ],
    },
    DiagItem {
        code: "E019",
        title: "Network download failed",
        summary_template: "Could not download {url}, {reason}",
        description: "An error occurred while connecting or downloading an asset over the network.",
        solutions: &[
            "Check network connectivity or mirror availability",
        ],
    },
    DiagItem {
        code: "E020",
        title: "Package frozen",
        summary_template: "Package `{name}` is frozen, use `lacie unfreeze {name}` first",
        description: "Modifications to a frozen package are prevented until it is unfrozen.",
        solutions: &[
            "Run `lacie unfreeze {name}` before performing modifications",
        ],
    },
    DiagItem {
        code: "E021",
        title: "Invalid filter",
        summary_template: "Invalid filter `{filter}` for `lacie {command}`, available keys: {keys}",
        description: "The filter key provided in `--filter` is not supported for this command.",
        solutions: &[
            "Use one of the allowed filter keys listed in the message",
        ],
    },
    DiagItem {
        code: "E022",
        title: "Prebuild script failed",
        summary_template: "prebuild script of {package} failed: {reason}, nothing was installed",
        description: "The package's prebuild lifecycle hook script exited with an error.",
        solutions: &[
            "Check the prebuild script output or run with verbose logging",
        ],
    },
    DiagItem {
        code: "E023",
        title: "Postbuild script failed",
        summary_template: "postbuild script of {package} failed: {reason}, the installation was rolled back",
        description: "The package's postbuild lifecycle hook script failed; installation was rolled back.",
        solutions: &[
            "Fix the postbuild script conditions before re-running installation",
        ],
    },
    DiagItem {
        code: "E024",
        title: "Dependency cycle",
        summary_template: "Dependency cycle: {chain}",
        description: "A cyclical dependency was detected in the package dependency graph.",
        solutions: &[
            "Break the dependency cycle in `lacie.toml` manifests",
        ],
    },
    DiagItem {
        code: "E025",
        title: "Preremove script failed",
        summary_template: "preremove script of {package} failed: {reason}, nothing was removed",
        description: "The package's preremove script failed, aborting uninstallation.",
        solutions: &[
            "Use `--no-scripts` if you wish to bypass removal scripts",
        ],
    },
    DiagItem {
        code: "E026",
        title: "Check deps script failed",
        summary_template: "check_deps script of {package} failed: {reason}, nothing was installed",
        description: "The package dependency verification script reported missing requirements.",
        solutions: &[
            "Install the missing system dependencies required by the package",
        ],
    },
    DiagItem {
        code: "E027",
        title: "Command name collision",
        summary_template: "Command name `{name}` is already used by `{package}`, choose another name",
        description: "Another installed package already registered this executable name in active/.",
        solutions: &[
            "Rename the command during install or rename the existing one with `lacie rename`",
        ],
    },
    DiagItem {
        code: "E028",
        title: "Invalid manifest format",
        summary_template: "Invalid {file} in {ref}: {reason}",
        description: "The TOML manifest violates schema or contains syntax errors.",
        solutions: &[
            "Fix manifest errors and validate using `lacie validate`",
        ],
    },
    DiagItem {
        code: "E029",
        title: "Invalid ignore usage",
        summary_template: "`--ignore` can only be used when updating all packages",
        description: "The `--ignore` flag cannot be used when targeting specific packages.",
        solutions: &[
            "Run `lacie update --ignore <pkg>` without specifying explicit package arguments",
        ],
    },
    DiagItem {
        code: "E030",
        title: "Source already exists",
        summary_template: "Source `{name}` is already installed, use `lacie source update {name}`",
        description: "A source plugin with this identifier is already registered.",
        solutions: &[
            "Run `lacie source update {name}` to update it",
        ],
    },
    DiagItem {
        code: "E031",
        title: "Built-in source cannot be removed",
        summary_template: "Source `{name}` is built in and cannot be removed",
        description: "Built-in sources like github are part of lacie and cannot be uninstalled.",
        solutions: &[
            "Use `lacie source disable {name}` if you want to turn it off",
        ],
    },
    DiagItem {
        code: "E032",
        title: "Default source cannot be removed",
        summary_template: "Source `{name}` is the default_source, change it first",
        description: "The source currently set as `default_source` cannot be removed or disabled.",
        solutions: &[
            "Change `default_source` with `lacie config set default_source <other>` first",
        ],
    },
    DiagItem {
        code: "E033",
        title: "Source plugin error",
        summary_template: "Source `{name}` failed: {reason}",
        description: "The source plugin process failed or returned invalid JSON-RPC response.",
        solutions: &[
            "Verify that the plugin script is functioning and compatible",
        ],
    },
    DiagItem {
        code: "E034",
        title: "Source does not support search",
        summary_template: "Source `{name}` does not support search, specify the full storage",
        description: "This source plugin does not implement the search capability.",
        solutions: &[
            "Specify the complete storage path (e.g. owner/repo) explicitly",
        ],
    },
    DiagItem {
        code: "E035",
        title: "Removal script checksum mismatch",
        summary_template: "Stored {hook} script of {package} {version} does not match its recorded sha256, use --no-scripts to remove without it",
        description: "The removal script on disk was modified or corrupted since approval.",
        solutions: &[
            "Use `lacie remove --no-scripts` to bypass running the modified script",
        ],
    },
    DiagItem {
        code: "E036",
        title: "Ruby interpreter not found",
        summary_template: "Ruby interpreter not found in PATH",
        description: "Lacie requires an accessible Ruby (>= 3.0) interpreter in PATH to execute lifecycle hooks and source plugins.",
        solutions: &[
            "Install Ruby and ensure the `ruby` executable is available in PATH",
        ],
    },
];

pub static WARNINGS: &[DiagItem] = &[
    DiagItem {
        code: "W001",
        title: "Could not add to PATH",
        summary_template: "Could not add {path} to PATH, lacie will not work until you add it manually, use `lacie explain W001`",
        description: "lacie could not add its active directory to PATH automatically. Until the directory is in PATH, lacie commands will not be found.",
        solutions: &[
            "Add the directory to PATH in the startup file of your shell (~/.bashrc, ~/.zshrc, etc.)",
        ],
    },
    DiagItem {
        code: "W002",
        title: "Insecure connection",
        summary_template: "Insecure connection to {url}, the manifest and its hash can be tampered with in transit",
        description: "The download URL uses unencrypted HTTP instead of HTTPS.",
        solutions: &[
            "Use HTTPS links whenever possible to protect against tampering",
        ],
    },
    DiagItem {
        code: "W003",
        title: "Fallback ABI selected",
        summary_template: "No {preferred} binary for {package}, only {fallback} is available, it may behave differently",
        description: "The preferred binary for your system was missing, but a compatible fallback was available.",
        solutions: &[
            "Confirm installation of the fallback build if acceptable",
        ],
    },
    DiagItem {
        code: "W004",
        title: "Version already installed",
        summary_template: "Version {version} of {package} is already installed with the same hash, use --force to reinstall",
        description: "The requested version is already present on the system with an identical checksum.",
        solutions: &[
            "Pass `--force` if you want to reinstall it anyway",
        ],
    },
    DiagItem {
        code: "W005",
        title: "Postremove script failed",
        summary_template: "postremove script of {package} failed: {reason}, the package was removed anyway",
        description: "The postremove hook script failed, but the package binary and entries were removed.",
        solutions: &[
            "Manually clean up any external files created by the package if needed",
        ],
    },
    DiagItem {
        code: "W006",
        title: "File not found next to manifest",
        summary_template: "File {file} was not found next to {ref}, upload it with the release",
        description: "A binary referenced by relative path was not found alongside the manifest.",
        solutions: &[
            "Ensure the asset is packaged or uploaded together with the manifest",
        ],
    },
    DiagItem {
        code: "W007",
        title: "Local path installation",
        summary_template: "Package {id} was installed from a local path, {path} may not exist on another machine",
        description: "The package was installed from local disk, making lockfiles non-portable across machines.",
        solutions: &[
            "Publish packages to a shared repository if cross-machine reproducibility is required",
        ],
    },
];

pub fn find_diag(code: &str) -> Option<&'static DiagItem> {
    ERRORS.iter().chain(WARNINGS.iter()).find(|item| item.code == code)
}
