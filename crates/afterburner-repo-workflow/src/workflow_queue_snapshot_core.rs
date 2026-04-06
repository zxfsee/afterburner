use std::fs;
use std::path::{Path, PathBuf};

use crate::queue_workflow_metadata::is_allowed_path;
use crate::workflow_process::{
    WorkflowProcessError, jj_changed_paths_between, jj_commit_id, jj_optional_commit_id,
    run_binary_command, run_command, sibling_binary_path,
};
use crate::workflow_queue_order::{
    QueueOrderError, check_top_runnable, is_docs_family_shared_overlap_only, promote_next_runnable,
    top_todo_scope,
};
use crate::workflow_queue_snapshot_cli::QueueSnapshotCommand as Command;
use crate::workflow_queue_snapshot_lineage::{
    QueueSnapshotLineageError, stamp_snapshot_files, verify_snapshot_files,
};

#[derive(Debug)]
pub enum QueueSnapshotError {
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for QueueSnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for QueueSnapshotError {}

impl From<std::io::Error> for QueueSnapshotError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<WorkflowProcessError> for QueueSnapshotError {
    fn from(value: WorkflowProcessError) -> Self {
        match value {
            WorkflowProcessError::Io(err) => Self::Io(err),
            other => Self::Parse(other.to_string()),
        }
    }
}

impl From<QueueSnapshotLineageError> for QueueSnapshotError {
    fn from(value: QueueSnapshotLineageError) -> Self {
        match value {
            QueueSnapshotLineageError::Io(err) => Self::Io(err),
            QueueSnapshotLineageError::Parse(msg) => Self::Parse(msg),
        }
    }
}

impl From<QueueOrderError> for QueueSnapshotError {
    fn from(value: QueueOrderError) -> Self {
        match value {
            QueueOrderError::Io(err) => Self::Io(err),
            QueueOrderError::Parse(msg) => Self::Parse(msg),
        }
    }
}

pub fn run(command: Command) -> Result<(), QueueSnapshotError> {
    match command {
        Command::Stamp {
            cargo_toml,
            changelog,
            parent_commit,
        } => stamp_snapshot_files(&cargo_toml, &changelog, parent_commit).map_err(Into::into),
        Command::StampCurrentParent {
            cargo_toml,
            changelog,
            repo_root,
        } => {
            let parent_commit = jj_commit_id(&repo_root, "@-")?;
            stamp_snapshot_files(&cargo_toml, &changelog, parent_commit).map_err(Into::into)
        }
        Command::Verify {
            cargo_toml,
            changelog,
            parent_commit,
            previous_parent_commit,
        } => verify_snapshot_files(
            &cargo_toml,
            &changelog,
            parent_commit,
            previous_parent_commit,
        )
        .map_err(Into::into),
        Command::VerifyCurrentLineage {
            cargo_toml,
            changelog,
            repo_root,
        } => {
            let parent_commit = jj_commit_id(&repo_root, "@-")?;
            let previous_parent_commit = jj_optional_commit_id(&repo_root, "@--")?;
            verify_snapshot_files(
                &cargo_toml,
                &changelog,
                parent_commit,
                previous_parent_commit,
            )
            .map_err(Into::into)
        }
        Command::CheckCompletionBoundary {
            cargo_toml,
            repo_root,
        } => check_completion_boundary(cargo_toml, repo_root),
        Command::CheckTopRunnable {
            cargo_toml,
            backlog,
        } => check_top_runnable(&cargo_toml, &backlog).map_err(Into::into),
        Command::PromoteNextRunnable {
            cargo_toml,
            backlog,
        } => promote_next_runnable(&cargo_toml, &backlog).map_err(Into::into),
        Command::ExecutePreflight {
            cargo_toml,
            changelog,
            repo_root,
            repair_stale_snapshot,
        } => execute_preflight(cargo_toml, changelog, repo_root, repair_stale_snapshot),
    }
}

fn check_completion_boundary(
    cargo_toml: PathBuf,
    repo_root: PathBuf,
) -> Result<(), QueueSnapshotError> {
    if jj_optional_commit_id(&repo_root, "@--")?.is_none() {
        return Ok(());
    }

    let cargo_text = fs::read_to_string(&cargo_toml)?;
    let todo = top_todo_scope(&cargo_text)?;
    let changed = jj_changed_paths_between(&repo_root, "@--", "@-")?;
    if changed.is_empty() {
        return Ok(());
    }

    let touched_queue_files = changed
        .iter()
        .any(|path| path == "Cargo.toml" || path == "CHANGELOG.md");
    let touched_top_scope_paths = changed
        .iter()
        .filter(|path| is_allowed_path(&todo.scope, path.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let touched_top_scope = !touched_top_scope_paths.is_empty();

    if touched_top_scope
        && !touched_queue_files
        && !is_docs_family_shared_overlap_only(&todo, &touched_top_scope_paths)
    {
        return Err(QueueSnapshotError::Parse(format!(
            "queue completion boundary violation: current top TODO `{}` still appears active even though the most recent landed commit touched its scope {:?}\nadvance `Cargo.toml` and `CHANGELOG.md` first, then run `just queue-resume`",
            todo.title, todo.scope
        )));
    }

    Ok(())
}

fn execute_preflight(
    cargo_toml: PathBuf,
    changelog: PathBuf,
    repo_root: PathBuf,
    repair_stale_snapshot: bool,
) -> Result<(), QueueSnapshotError> {
    let objective_lock_bin = sibling_binary_path("workflow_objective_lock")?;
    run_binary_command(
        repo_root.as_path(),
        &objective_lock_bin,
        &["check-repo-locks", "--repo-root", "."],
    )?;

    if repair_stale_snapshot {
        refresh_queue_snapshot(&cargo_toml, &changelog, &repo_root)?;
        run_command(
            repo_root.as_path(),
            "cargo",
            &["nextest", "run", "--locked", "--test", "todo_queue_horizon"],
        )?;
    }

    let backlog = repo_root.join("docs/backlog.md");
    check_top_runnable(&cargo_toml, &backlog)?;
    check_completion_boundary(cargo_toml.clone(), repo_root.clone())?;
    let parent_commit = jj_commit_id(&repo_root, "@-")?;
    let previous_parent_commit = jj_optional_commit_id(&repo_root, "@--")?;
    verify_snapshot_files(
        &cargo_toml,
        &changelog,
        parent_commit,
        previous_parent_commit,
    )?;

    let cargo_toml_str = cargo_toml
        .to_str()
        .ok_or_else(|| QueueSnapshotError::Parse("cargo_toml path is not utf8".into()))?;
    let repo_root_str = repo_root
        .to_str()
        .ok_or_else(|| QueueSnapshotError::Parse("repo_root path is not utf8".into()))?;
    let pin_args = vec![
        "pin",
        "--objective",
        "execute-top-item",
        "--cargo-toml",
        cargo_toml_str,
        "--repo-root",
        repo_root_str,
        "--expected-action",
        "execute-top-item",
        "--allow-existing-path",
        "Cargo.toml",
        "--allow-existing-path",
        "CHANGELOG.md",
    ];
    run_binary_command(repo_root.as_path(), &objective_lock_bin, &pin_args)?;
    run_binary_command(
        repo_root.as_path(),
        &objective_lock_bin,
        &["check-worktree", "--action", "execute-top-item"],
    )?;
    Ok(())
}

fn refresh_queue_snapshot(
    cargo_toml: &Path,
    changelog: &Path,
    repo_root: &Path,
) -> Result<(), QueueSnapshotError> {
    let objective_lock_bin = sibling_binary_path("workflow_objective_lock")?;
    run_binary_command(
        repo_root,
        &objective_lock_bin,
        &[
            "pin",
            "--objective",
            "queue-only",
            "--expected-action",
            "queue-refresh",
        ],
    )?;
    let changelog_name = changelog
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| QueueSnapshotError::Parse("changelog filename is not utf8".into()))?;
    run_binary_command(
        repo_root,
        &objective_lock_bin,
        &[
            "check-paths",
            "--action",
            "queue-refresh",
            "--path",
            changelog_name,
        ],
    )?;
    let changelog_str = changelog
        .to_str()
        .ok_or_else(|| QueueSnapshotError::Parse("changelog path is not utf8".into()))?;
    run_command(repo_root, "git-cliff", &["-o", changelog_str])?;
    let parent_commit = jj_commit_id(repo_root, "@-")?;
    stamp_snapshot_files(cargo_toml, changelog, parent_commit)?;
    let backlog = repo_root.join("docs/backlog.md");
    check_top_runnable(cargo_toml, &backlog)?;
    let previous_parent_commit = jj_optional_commit_id(repo_root, "@--")?;
    let current_parent_commit = jj_commit_id(repo_root, "@-")?;
    verify_snapshot_files(
        cargo_toml,
        changelog,
        current_parent_commit,
        previous_parent_commit,
    )?;
    run_binary_command(repo_root, &objective_lock_bin, &["clear"])?;
    Ok(())
}
