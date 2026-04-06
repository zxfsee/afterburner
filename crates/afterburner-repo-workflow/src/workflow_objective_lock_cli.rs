use std::path::PathBuf;

const DEFAULT_LOCK_PATH: &str = ".git/afterburner/objective-lock.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Objective {
    QueueOnly,
    TopScopeFix,
    BacklogOnly,
    ExecuteTopItem,
    DocsOnly,
    ReviewOnly,
}

impl Objective {
    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw {
            "queue-only" => Ok(Self::QueueOnly),
            "top-scope-fix" => Ok(Self::TopScopeFix),
            "backlog-only" => Ok(Self::BacklogOnly),
            "execute-top-item" => Ok(Self::ExecuteTopItem),
            "docs-only" => Ok(Self::DocsOnly),
            "review-only" => Ok(Self::ReviewOnly),
            _ => Err(format!("unknown objective `{raw}`\n{}", usage())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::QueueOnly => "queue-only",
            Self::TopScopeFix => "top-scope-fix",
            Self::BacklogOnly => "backlog-only",
            Self::ExecuteTopItem => "execute-top-item",
            Self::DocsOnly => "docs-only",
            Self::ReviewOnly => "review-only",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandSpec {
    Pin {
        objective: Objective,
        cargo_toml: PathBuf,
        repo_root: PathBuf,
        lock_file: PathBuf,
        expected_action: Option<String>,
        allow_existing_paths: Vec<String>,
    },
    CheckPaths {
        lock_file: PathBuf,
        action: Option<String>,
        paths: Vec<String>,
    },
    CheckWorktree {
        lock_file: PathBuf,
        repo_root: PathBuf,
        action: Option<String>,
    },
    CheckRepoLocks {
        repo_root: PathBuf,
    },
    RepairRepoLocks {
        repo_root: PathBuf,
        stale_after_seconds: u64,
    },
    Clear {
        lock_file: PathBuf,
    },
}

pub fn parse_args<I>(args: I) -> Result<CommandSpec, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let Some(subcommand) = args.next() else {
        return Err(usage().to_string());
    };

    match subcommand.as_str() {
        "pin" => parse_pin_args(args),
        "check-paths" => parse_check_paths_args(args),
        "check-worktree" => parse_check_worktree_args(args),
        "check-repo-locks" => parse_check_repo_locks_args(args),
        "repair-repo-locks" => parse_repair_repo_locks_args(args),
        "clear" => parse_clear_args(args),
        "--help" | "-h" | "help" => Err(usage().to_string()),
        _ => Err(format!(
            "unknown objective-lock subcommand `{subcommand}`\n{}",
            usage()
        )),
    }
}

pub fn usage() -> &'static str {
    "usage: workflow_objective_lock <pin|check-paths|check-worktree|check-repo-locks|repair-repo-locks|clear> [options]\n\
pin: --objective <queue-only|top-scope-fix|backlog-only|execute-top-item|docs-only|review-only> [--cargo-toml PATH] [--repo-root PATH] [--lock-file PATH] [--expected-action ACTION] [--allow-existing-path PATH...]\n\
check-paths: [--lock-file PATH] [--action ACTION] --path PATH [--path PATH...]\n\
check-worktree: [--lock-file PATH] [--repo-root PATH] [--action ACTION]\n\
check-repo-locks: [--repo-root PATH]\n\
repair-repo-locks: [--repo-root PATH] [--stale-after-seconds N]\n\
clear: [--lock-file PATH]"
}

fn parse_pin_args<I>(args: I) -> Result<CommandSpec, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut objective = None::<Objective>;
    let mut cargo_toml = PathBuf::from("Cargo.toml");
    let mut repo_root = PathBuf::from(".");
    let mut lock_file = PathBuf::from(DEFAULT_LOCK_PATH);
    let mut expected_action = None::<String>;
    let mut allow_existing_paths = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--objective" => {
                objective = Some(Objective::parse(&parse_value(&mut args, "--objective")?)?)
            }
            "--cargo-toml" => cargo_toml = PathBuf::from(parse_value(&mut args, "--cargo-toml")?),
            "--repo-root" => repo_root = PathBuf::from(parse_value(&mut args, "--repo-root")?),
            "--lock-file" => lock_file = PathBuf::from(parse_value(&mut args, "--lock-file")?),
            "--expected-action" => {
                expected_action = Some(parse_value(&mut args, "--expected-action")?)
            }
            "--allow-existing-path" => {
                allow_existing_paths.push(parse_value(&mut args, "--allow-existing-path")?)
            }
            _ if arg.starts_with("--objective=") => {
                objective = Some(Objective::parse(arg.trim_start_matches("--objective="))?)
            }
            _ if arg.starts_with("--cargo-toml=") => {
                cargo_toml = PathBuf::from(arg.trim_start_matches("--cargo-toml=").to_string())
            }
            _ if arg.starts_with("--repo-root=") => {
                repo_root = PathBuf::from(arg.trim_start_matches("--repo-root=").to_string())
            }
            _ if arg.starts_with("--lock-file=") => {
                lock_file = PathBuf::from(arg.trim_start_matches("--lock-file=").to_string())
            }
            _ if arg.starts_with("--expected-action=") => {
                expected_action = Some(arg.trim_start_matches("--expected-action=").to_string())
            }
            _ if arg.starts_with("--allow-existing-path=") => allow_existing_paths
                .push(arg.trim_start_matches("--allow-existing-path=").to_string()),
            _ => return Err(format!("unknown argument `{arg}`\n{}", usage())),
        }
    }

    let objective =
        objective.ok_or_else(|| format!("missing required flag --objective\n{}", usage()))?;

    Ok(CommandSpec::Pin {
        objective,
        cargo_toml,
        repo_root,
        lock_file,
        expected_action,
        allow_existing_paths,
    })
}

fn parse_check_paths_args<I>(args: I) -> Result<CommandSpec, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut lock_file = PathBuf::from(DEFAULT_LOCK_PATH);
    let mut action = None::<String>;
    let mut paths = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--lock-file" => lock_file = PathBuf::from(parse_value(&mut args, "--lock-file")?),
            "--action" => action = Some(parse_value(&mut args, "--action")?),
            "--path" => paths.push(parse_value(&mut args, "--path")?),
            _ if arg.starts_with("--lock-file=") => {
                lock_file = PathBuf::from(arg.trim_start_matches("--lock-file=").to_string())
            }
            _ if arg.starts_with("--action=") => {
                action = Some(arg.trim_start_matches("--action=").to_string())
            }
            _ if arg.starts_with("--path=") => {
                paths.push(arg.trim_start_matches("--path=").to_string())
            }
            _ => return Err(format!("unknown argument `{arg}`\n{}", usage())),
        }
    }

    if paths.is_empty() {
        return Err(format!(
            "check-paths requires at least one --path\n{}",
            usage()
        ));
    }

    Ok(CommandSpec::CheckPaths {
        lock_file,
        action,
        paths,
    })
}

fn parse_check_worktree_args<I>(args: I) -> Result<CommandSpec, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut lock_file = PathBuf::from(DEFAULT_LOCK_PATH);
    let mut repo_root = PathBuf::from(".");
    let mut action = None::<String>;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--lock-file" => lock_file = PathBuf::from(parse_value(&mut args, "--lock-file")?),
            "--repo-root" => repo_root = PathBuf::from(parse_value(&mut args, "--repo-root")?),
            "--action" => action = Some(parse_value(&mut args, "--action")?),
            _ if arg.starts_with("--lock-file=") => {
                lock_file = PathBuf::from(arg.trim_start_matches("--lock-file=").to_string())
            }
            _ if arg.starts_with("--repo-root=") => {
                repo_root = PathBuf::from(arg.trim_start_matches("--repo-root=").to_string())
            }
            _ if arg.starts_with("--action=") => {
                action = Some(arg.trim_start_matches("--action=").to_string())
            }
            _ => return Err(format!("unknown argument `{arg}`\n{}", usage())),
        }
    }

    Ok(CommandSpec::CheckWorktree {
        lock_file,
        repo_root,
        action,
    })
}

fn parse_clear_args<I>(args: I) -> Result<CommandSpec, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut lock_file = PathBuf::from(DEFAULT_LOCK_PATH);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--lock-file" => lock_file = PathBuf::from(parse_value(&mut args, "--lock-file")?),
            _ if arg.starts_with("--lock-file=") => {
                lock_file = PathBuf::from(arg.trim_start_matches("--lock-file=").to_string())
            }
            _ => return Err(format!("unknown argument `{arg}`\n{}", usage())),
        }
    }

    Ok(CommandSpec::Clear { lock_file })
}

fn parse_check_repo_locks_args<I>(args: I) -> Result<CommandSpec, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut repo_root = PathBuf::from(".");

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => repo_root = PathBuf::from(parse_value(&mut args, "--repo-root")?),
            _ if arg.starts_with("--repo-root=") => {
                repo_root = PathBuf::from(arg.trim_start_matches("--repo-root=").to_string())
            }
            _ => return Err(format!("unknown argument `{arg}`\n{}", usage())),
        }
    }

    Ok(CommandSpec::CheckRepoLocks { repo_root })
}

fn parse_repair_repo_locks_args<I>(args: I) -> Result<CommandSpec, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut repo_root = PathBuf::from(".");
    let mut stale_after_seconds = 300_u64;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => repo_root = PathBuf::from(parse_value(&mut args, "--repo-root")?),
            "--stale-after-seconds" => {
                stale_after_seconds = parse_value(&mut args, "--stale-after-seconds")?
                    .parse::<u64>()
                    .map_err(|_| {
                        "`--stale-after-seconds` must be an unsigned integer".to_string()
                    })?;
            }
            _ if arg.starts_with("--repo-root=") => {
                repo_root = PathBuf::from(arg.trim_start_matches("--repo-root=").to_string())
            }
            _ if arg.starts_with("--stale-after-seconds=") => {
                stale_after_seconds = arg
                    .trim_start_matches("--stale-after-seconds=")
                    .parse::<u64>()
                    .map_err(|_| {
                        "`--stale-after-seconds` must be an unsigned integer".to_string()
                    })?;
            }
            _ => return Err(format!("unknown argument `{arg}`\n{}", usage())),
        }
    }

    Ok(CommandSpec::RepairRepoLocks {
        repo_root,
        stale_after_seconds,
    })
}

fn parse_value<I>(args: &mut I, flag: &str) -> Result<String, String>
where
    I: Iterator<Item = String>,
{
    args.next()
        .ok_or_else(|| format!("missing value for {flag}\n{}", usage()))
}

#[cfg(test)]
mod tests {
    use super::{CommandSpec, Objective, parse_args};
    use std::path::PathBuf;

    #[test]
    fn objective_lock_cli_rejects_missing_required_objective_flag() {
        let err = parse_args(["pin"].into_iter().map(str::to_string)).expect_err("missing flag");
        assert!(
            err.contains("missing required flag --objective"),
            "expected missing required objective error: {err}"
        );
    }

    #[test]
    fn objective_lock_cli_distinguishes_missing_objective_value() {
        let err = parse_args(["pin", "--objective"].into_iter().map(str::to_string))
            .expect_err("missing value");
        assert!(
            err.contains("missing value for --objective"),
            "expected missing objective value error: {err}"
        );
    }

    #[test]
    fn objective_lock_cli_parses_pin_with_defaults() {
        let command = parse_args(
            ["pin", "--objective", "queue-only"]
                .into_iter()
                .map(str::to_string),
        )
        .expect("parse");
        assert_eq!(
            command,
            CommandSpec::Pin {
                objective: Objective::QueueOnly,
                cargo_toml: PathBuf::from("Cargo.toml"),
                repo_root: PathBuf::from("."),
                lock_file: PathBuf::from(".git/afterburner/objective-lock.json"),
                expected_action: None,
                allow_existing_paths: Vec::new(),
            }
        );
    }
}
