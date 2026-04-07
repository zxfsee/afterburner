use std::path::PathBuf;

use crate::workflow_objective_lock_cli_parse::{
    parse_check_paths_args, parse_check_repo_locks_args, parse_check_worktree_args,
    parse_clear_args, parse_pin_args, parse_repair_repo_locks_args,
};

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
