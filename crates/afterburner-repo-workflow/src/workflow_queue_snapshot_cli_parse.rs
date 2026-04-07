use std::iter::Peekable;
use std::path::PathBuf;

use crate::workflow_queue_snapshot_cli::{QueueSnapshotCommand, usage};

pub(super) fn parse_top_runnable_args<I>(
    args: Peekable<I>,
    promote: bool,
) -> Result<QueueSnapshotCommand, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args;
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

pub(super) fn parse_execute_preflight_args<I>(
    args: Peekable<I>,
) -> Result<QueueSnapshotCommand, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args;
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

pub(super) fn parse_completion_boundary_args<I>(
    args: Peekable<I>,
) -> Result<QueueSnapshotCommand, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args;
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

pub(super) fn parse_command_args<I>(
    args: Peekable<I>,
    stamp: bool,
) -> Result<QueueSnapshotCommand, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args;
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

    let parent_commit = parent_commit
        .ok_or_else(|| format!("missing required flag --parent-commit\n{}", usage()))?;

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

pub(super) fn parse_lineage_command_args<I>(
    args: Peekable<I>,
    stamp: bool,
) -> Result<QueueSnapshotCommand, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args;
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

fn parse_value<I>(args: &mut Peekable<I>, flag: &str) -> Result<String, String>
where
    I: Iterator<Item = String>,
{
    args.next()
        .ok_or_else(|| format!("missing value for {flag}\n{}", usage()))
}
