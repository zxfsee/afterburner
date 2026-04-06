use std::path::PathBuf;

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

fn parse_top_runnable_args<I>(args: I, promote: bool) -> Result<QueueSnapshotCommand, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut cargo_toml = PathBuf::from("Cargo.toml");
    let mut backlog = PathBuf::from("docs/backlog.md");

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--cargo-toml" => cargo_toml = PathBuf::from(parse_value(&mut args, "--cargo-toml")?),
            "--backlog" => backlog = PathBuf::from(parse_value(&mut args, "--backlog")?),
            _ if arg.starts_with("--cargo-toml=") => {
                cargo_toml = PathBuf::from(arg.trim_start_matches("--cargo-toml=").to_string())
            }
            _ if arg.starts_with("--backlog=") => {
                backlog = PathBuf::from(arg.trim_start_matches("--backlog=").to_string())
            }
            _ => return Err(format!("unknown argument `{arg}`\n{}", usage())),
        }
    }

    Ok(if promote {
        QueueSnapshotCommand::PromoteNextRunnable {
            cargo_toml,
            backlog,
        }
    } else {
        QueueSnapshotCommand::CheckTopRunnable {
            cargo_toml,
            backlog,
        }
    })
}

fn parse_execute_preflight_args<I>(args: I) -> Result<QueueSnapshotCommand, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut cargo_toml = PathBuf::from("Cargo.toml");
    let mut changelog = PathBuf::from("CHANGELOG.md");
    let mut repo_root = PathBuf::from(".");
    let mut repair_stale_snapshot = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--cargo-toml" => cargo_toml = PathBuf::from(parse_value(&mut args, "--cargo-toml")?),
            "--changelog" => changelog = PathBuf::from(parse_value(&mut args, "--changelog")?),
            "--repo-root" => repo_root = PathBuf::from(parse_value(&mut args, "--repo-root")?),
            "--repair-stale-snapshot" => repair_stale_snapshot = true,
            _ if arg.starts_with("--cargo-toml=") => {
                cargo_toml = PathBuf::from(arg.trim_start_matches("--cargo-toml=").to_string())
            }
            _ if arg.starts_with("--changelog=") => {
                changelog = PathBuf::from(arg.trim_start_matches("--changelog=").to_string())
            }
            _ if arg.starts_with("--repo-root=") => {
                repo_root = PathBuf::from(arg.trim_start_matches("--repo-root=").to_string())
            }
            _ => return Err(format!("unknown argument `{arg}`\n{}", usage())),
        }
    }

    Ok(QueueSnapshotCommand::ExecutePreflight {
        cargo_toml,
        changelog,
        repo_root,
        repair_stale_snapshot,
    })
}

fn parse_completion_boundary_args<I>(args: I) -> Result<QueueSnapshotCommand, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut cargo_toml = PathBuf::from("Cargo.toml");
    let mut repo_root = PathBuf::from(".");

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--cargo-toml" => cargo_toml = PathBuf::from(parse_value(&mut args, "--cargo-toml")?),
            "--repo-root" => repo_root = PathBuf::from(parse_value(&mut args, "--repo-root")?),
            _ if arg.starts_with("--cargo-toml=") => {
                cargo_toml = PathBuf::from(arg.trim_start_matches("--cargo-toml=").to_string())
            }
            _ if arg.starts_with("--repo-root=") => {
                repo_root = PathBuf::from(arg.trim_start_matches("--repo-root=").to_string())
            }
            _ => return Err(format!("unknown argument `{arg}`\n{}", usage())),
        }
    }

    Ok(QueueSnapshotCommand::CheckCompletionBoundary {
        cargo_toml,
        repo_root,
    })
}

fn parse_command_args<I>(args: I, stamp: bool) -> Result<QueueSnapshotCommand, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut cargo_toml = PathBuf::from("Cargo.toml");
    let mut changelog = PathBuf::from("CHANGELOG.md");
    let mut parent_commit = None::<String>;
    let mut previous_parent_commit = None::<String>;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--cargo-toml" => cargo_toml = PathBuf::from(parse_value(&mut args, "--cargo-toml")?),
            "--changelog" => changelog = PathBuf::from(parse_value(&mut args, "--changelog")?),
            "--parent-commit" => parent_commit = Some(parse_value(&mut args, "--parent-commit")?),
            "--previous-parent-commit" => {
                previous_parent_commit = Some(parse_value(&mut args, "--previous-parent-commit")?)
            }
            _ if arg.starts_with("--cargo-toml=") => {
                cargo_toml = PathBuf::from(arg.trim_start_matches("--cargo-toml=").to_string())
            }
            _ if arg.starts_with("--changelog=") => {
                changelog = PathBuf::from(arg.trim_start_matches("--changelog=").to_string())
            }
            _ if arg.starts_with("--parent-commit=") => {
                parent_commit = Some(arg.trim_start_matches("--parent-commit=").to_string())
            }
            _ if arg.starts_with("--previous-parent-commit=") => {
                previous_parent_commit = Some(
                    arg.trim_start_matches("--previous-parent-commit=")
                        .to_string(),
                )
            }
            _ => return Err(format!("unknown argument `{arg}`\n{}", usage())),
        }
    }

    let parent_commit =
        parent_commit.ok_or_else(|| format!("missing value for --parent-commit\n{}", usage()))?;

    Ok(if stamp {
        QueueSnapshotCommand::Stamp {
            cargo_toml,
            changelog,
            parent_commit,
        }
    } else {
        QueueSnapshotCommand::Verify {
            cargo_toml,
            changelog,
            parent_commit,
            previous_parent_commit,
        }
    })
}

fn parse_lineage_command_args<I>(args: I, stamp: bool) -> Result<QueueSnapshotCommand, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut cargo_toml = PathBuf::from("Cargo.toml");
    let mut changelog = PathBuf::from("CHANGELOG.md");
    let mut repo_root = PathBuf::from(".");

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--cargo-toml" => cargo_toml = PathBuf::from(parse_value(&mut args, "--cargo-toml")?),
            "--changelog" => changelog = PathBuf::from(parse_value(&mut args, "--changelog")?),
            "--repo-root" => repo_root = PathBuf::from(parse_value(&mut args, "--repo-root")?),
            _ if arg.starts_with("--cargo-toml=") => {
                cargo_toml = PathBuf::from(arg.trim_start_matches("--cargo-toml=").to_string())
            }
            _ if arg.starts_with("--changelog=") => {
                changelog = PathBuf::from(arg.trim_start_matches("--changelog=").to_string())
            }
            _ if arg.starts_with("--repo-root=") => {
                repo_root = PathBuf::from(arg.trim_start_matches("--repo-root=").to_string())
            }
            _ => return Err(format!("unknown argument `{arg}`\n{}", usage())),
        }
    }

    Ok(if stamp {
        QueueSnapshotCommand::StampCurrentParent {
            cargo_toml,
            changelog,
            repo_root,
        }
    } else {
        QueueSnapshotCommand::VerifyCurrentLineage {
            cargo_toml,
            changelog,
            repo_root,
        }
    })
}

fn parse_value<I>(args: &mut I, flag: &str) -> Result<String, String>
where
    I: Iterator<Item = String>,
{
    args.next()
        .ok_or_else(|| format!("missing value for {flag}\n{}", usage()))
}
