use std::path::PathBuf;

use crate::workflow_queue_snapshot_cli_parse::{
    parse_command_args, parse_completion_boundary_args, parse_execute_preflight_args,
    parse_lineage_command_args, parse_top_runnable_args,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueSnapshotCommand {
    Stamp {
        cargo_toml: PathBuf,
        changelog: PathBuf,
        parent_commit: String,
    },
    StampCurrentParent {
        cargo_toml: PathBuf,
        changelog: PathBuf,
        repo_root: PathBuf,
    },
    Verify {
        cargo_toml: PathBuf,
        changelog: PathBuf,
        parent_commit: String,
        previous_parent_commit: Option<String>,
    },
    VerifyCurrentLineage {
        cargo_toml: PathBuf,
        changelog: PathBuf,
        repo_root: PathBuf,
    },
    CheckCompletionBoundary {
        cargo_toml: PathBuf,
        repo_root: PathBuf,
    },
    CheckTopRunnable {
        cargo_toml: PathBuf,
        backlog: PathBuf,
    },
    PromoteNextRunnable {
        cargo_toml: PathBuf,
        backlog: PathBuf,
    },
    ExecutePreflight {
        cargo_toml: PathBuf,
        changelog: PathBuf,
        repo_root: PathBuf,
        repair_stale_snapshot: bool,
    },
}

pub fn usage() -> &'static str {
    "usage: workflow_queue_snapshot <stamp|verify|stamp-current-parent|verify-current-lineage|check-completion-boundary|check-top-runnable|promote-next-runnable|execute-preflight> [options]\n\
stamp: --cargo-toml PATH --changelog PATH --parent-commit COMMIT\n\
verify: --cargo-toml PATH --changelog PATH --parent-commit COMMIT [--previous-parent-commit COMMIT]\n\
stamp-current-parent: [--cargo-toml PATH] [--changelog PATH] [--repo-root PATH]\n\
verify-current-lineage: [--cargo-toml PATH] [--changelog PATH] [--repo-root PATH]\n\
check-completion-boundary: [--cargo-toml PATH] [--repo-root PATH]\n\
check-top-runnable: [--cargo-toml PATH] [--backlog PATH]\n\
promote-next-runnable: [--cargo-toml PATH] [--backlog PATH]\n\
execute-preflight: [--cargo-toml PATH] [--changelog PATH] [--repo-root PATH] [--repair-stale-snapshot]"
}

pub fn parse_args<I>(args: I) -> Result<QueueSnapshotCommand, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let Some(subcommand) = args.next() else {
        return Err(usage().to_string());
    };
    match subcommand.as_str() {
        "stamp" => parse_command_args(args, true),
        "verify" => parse_command_args(args, false),
        "stamp-current-parent" => parse_lineage_command_args(args, true),
        "verify-current-lineage" => parse_lineage_command_args(args, false),
        "check-completion-boundary" => parse_completion_boundary_args(args),
        "check-top-runnable" => parse_top_runnable_args(args, false),
        "promote-next-runnable" => parse_top_runnable_args(args, true),
        "execute-preflight" => parse_execute_preflight_args(args),
        "--help" | "-h" | "help" => Err(usage().to_string()),
        _ => Err(format!(
            "unknown queue-snapshot subcommand `{subcommand}`\n{}",
            usage()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{QueueSnapshotCommand, parse_args};
    use std::path::PathBuf;

    #[test]
    fn queue_snapshot_cli_parses_lineage_defaults() {
        let command =
            parse_args(["stamp-current-parent"].into_iter().map(str::to_string)).expect("parse");
        assert_eq!(
            command,
            QueueSnapshotCommand::StampCurrentParent {
                cargo_toml: PathBuf::from("Cargo.toml"),
                changelog: PathBuf::from("CHANGELOG.md"),
                repo_root: PathBuf::from("."),
            }
        );
    }

    #[test]
    fn queue_snapshot_cli_parses_top_runnable_flags() {
        let command = parse_args(
            [
                "promote-next-runnable",
                "--cargo-toml",
                "Custom.toml",
                "--backlog=docs/custom-backlog.md",
            ]
            .into_iter()
            .map(str::to_string),
        )
        .expect("parse");
        assert_eq!(
            command,
            QueueSnapshotCommand::PromoteNextRunnable {
                cargo_toml: PathBuf::from("Custom.toml"),
                backlog: PathBuf::from("docs/custom-backlog.md"),
            }
        );
    }

    #[test]
    fn queue_snapshot_cli_parses_execute_preflight_repair_flag() {
        let command = parse_args(
            [
                "execute-preflight",
                "--cargo-toml=Queue.toml",
                "--changelog",
                "Queue.md",
                "--repo-root",
                "/tmp/repo",
                "--repair-stale-snapshot",
            ]
            .into_iter()
            .map(str::to_string),
        )
        .expect("parse");
        assert_eq!(
            command,
            QueueSnapshotCommand::ExecutePreflight {
                cargo_toml: PathBuf::from("Queue.toml"),
                changelog: PathBuf::from("Queue.md"),
                repo_root: PathBuf::from("/tmp/repo"),
                repair_stale_snapshot: true,
            }
        );
    }

    #[test]
    fn queue_snapshot_cli_rejects_verify_without_parent_commit() {
        let err = parse_args(["verify"].into_iter().map(str::to_string)).expect_err("missing arg");
        assert!(
            err.contains("missing required flag --parent-commit"),
            "expected missing required parent-commit error: {err}"
        );
    }

    #[test]
    fn queue_snapshot_cli_distinguishes_missing_parent_commit_value() {
        let err = parse_args(
            ["verify", "--parent-commit"]
                .into_iter()
                .map(str::to_string),
        )
        .expect_err("missing value");
        assert!(
            err.contains("missing value for --parent-commit"),
            "expected missing value error: {err}"
        );
    }

    #[test]
    fn queue_snapshot_cli_rejects_unknown_argument() {
        let err = parse_args(
            ["check-top-runnable", "--bogus"]
                .into_iter()
                .map(str::to_string),
        )
        .expect_err("unknown flag");
        assert!(
            err.contains("unknown argument `--bogus`"),
            "expected unknown argument error: {err}"
        );
    }
}
