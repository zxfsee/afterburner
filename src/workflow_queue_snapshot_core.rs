use std::fs;
use std::path::{Path, PathBuf};

use crate::queue_workflow_metadata::{
    QueueTextItem, QueueWorkflowMetadataError, extract_backtick_values, extract_marked_section,
    first_todo_item, is_allowed_path, normalize_scope_path, rebuild_item_block,
    split_items_after_marker, split_marked_items,
};
use crate::workflow_process::{
    WorkflowProcessError, jj_changed_paths_between, jj_commit_id, jj_optional_commit_id,
    run_binary_command, run_command, sibling_binary_path,
};
use crate::workflow_queue_snapshot_cli::QueueSnapshotCommand as Command;
use crate::workflow_queue_snapshot_lineage::{
    QueueSnapshotLineageError, stamp_snapshot_files, verify_snapshot_files,
};

const TODO_START: &str = "## TODO";
const TRUNK_MARKER: &str = "## [Trunk]";
const BACKLOG_ITEMS_START: &str = "## Items";

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct TodoScope {
    title: String,
    scope: Vec<String>,
    boundary: String,
    contracts: Vec<String>,
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
        } => check_top_runnable(cargo_toml, backlog),
        Command::PromoteNextRunnable {
            cargo_toml,
            backlog,
        } => promote_next_runnable(cargo_toml, backlog),
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

fn check_top_runnable(cargo_toml: PathBuf, backlog: PathBuf) -> Result<(), QueueSnapshotError> {
    let cargo_text = fs::read_to_string(&cargo_toml)?;
    let items = todo_items(&cargo_text)?;
    let Some(top) = items.first() else {
        return Ok(());
    };
    let Some(blocked_by) = &top.blocked_by else {
        return Ok(());
    };
    let next_runnable = items
        .iter()
        .skip(1)
        .find(|item| item.blocked_by.is_none())
        .map(|item| item.title.as_str());
    match next_runnable {
        Some(next) => Err(QueueSnapshotError::Parse(format!(
            "top active TODO `{}` is blocked by `{}`\nnext runnable item: `{}`\nrun `just queue-promote-next-runnable` to repair the active queue order",
            top.title, blocked_by, next
        ))),
        None => match first_runnable_backlog_item(&backlog)? {
            Some(next) => Err(QueueSnapshotError::Parse(format!(
                "top active TODO `{}` is blocked by `{}` and no runnable item exists in the active queue\nnext runnable backlog item: `{}`\nrun `just queue-promote-next-runnable` to promote it into the active queue",
                top.title, blocked_by, next.title
            ))),
            None => Err(QueueSnapshotError::Parse(format!(
                "top active TODO `{}` is blocked by `{}` and no runnable item exists in the active queue or backlog\nrepair the queue order or unblock a backlog item before execution",
                top.title, blocked_by
            ))),
        },
    }
}

fn promote_next_runnable(cargo_toml: PathBuf, backlog: PathBuf) -> Result<(), QueueSnapshotError> {
    let cargo_text = fs::read_to_string(&cargo_toml)?;
    let (prefix, suffix, mut items) = split_todo_items(&cargo_text)?;
    let Some(first_runnable) = items.iter().position(|item| item.blocked_by.is_none()) else {
        return promote_from_backlog(cargo_toml, backlog, prefix, suffix, items);
    };
    if first_runnable == 0 {
        return Ok(());
    }
    let promoted = items.remove(first_runnable);
    items.insert(0, promoted);
    let rebuilt = rebuild_todo_block(&prefix, &items, &suffix);
    fs::write(&cargo_toml, rebuilt)?;
    Ok(())
}

fn promote_from_backlog(
    cargo_toml: PathBuf,
    backlog: PathBuf,
    prefix: String,
    suffix: String,
    mut active_items: Vec<QueueTextItem>,
) -> Result<(), QueueSnapshotError> {
    let Some(backlog_text) = read_optional_file(&backlog)? else {
        return Err(QueueSnapshotError::Parse(
            "cannot promote next runnable item because no runnable TODO exists in the active queue or backlog"
                .into(),
        ));
    };
    let (backlog_prefix, backlog_suffix, mut backlog_items) = split_backlog_items(&backlog_text)?;
    let Some(first_runnable_backlog) = backlog_items
        .iter()
        .position(|item| item.blocked_by.is_none())
    else {
        return Err(QueueSnapshotError::Parse(
            "cannot promote next runnable item because no runnable TODO exists in the active queue or backlog"
                .into(),
        ));
    };
    let promoted = backlog_items.remove(first_runnable_backlog);
    active_items.insert(0, promoted);
    fs::write(
        &cargo_toml,
        rebuild_todo_block(&prefix, &active_items, &suffix),
    )?;
    fs::write(
        &backlog,
        rebuild_item_block(&backlog_prefix, &backlog_items, &backlog_suffix),
    )?;
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
    check_top_runnable(cargo_toml.clone(), backlog)?;
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
    check_top_runnable(cargo_toml.to_path_buf(), backlog)?;
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

fn split_todo_items(
    cargo_toml: &str,
) -> Result<(String, String, Vec<QueueTextItem>), QueueSnapshotError> {
    split_marked_items(cargo_toml, TODO_START, TRUNK_MARKER).map_err(|err| {
        QueueSnapshotError::Parse(match err {
            QueueWorkflowMetadataError::MissingMarkedSection { .. } => {
                "Cargo.toml changelog template missing TODO section".into()
            }
            QueueWorkflowMetadataError::MissingMarker { .. } => {
                "Cargo.toml changelog template missing `## [Trunk]` marker".into()
            }
            QueueWorkflowMetadataError::MissingItem => {
                "Cargo.toml TODO section does not contain any item".into()
            }
            other => other.to_string(),
        })
    })
}

fn split_backlog_items(
    backlog_text: &str,
) -> Result<(String, String, Vec<QueueTextItem>), QueueSnapshotError> {
    split_items_after_marker(backlog_text, BACKLOG_ITEMS_START).map_err(|err| {
        QueueSnapshotError::Parse(match err {
            QueueWorkflowMetadataError::MissingMarker { .. } => {
                "docs/backlog.md missing `## Items` section".into()
            }
            other => other.to_string(),
        })
    })
}

fn todo_items(cargo_toml: &str) -> Result<Vec<QueueTextItem>, QueueSnapshotError> {
    let section = extract_marked_section(cargo_toml, TODO_START, TRUNK_MARKER).map_err(|_| {
        QueueSnapshotError::Parse("Cargo.toml changelog template missing TODO section".into())
    })?;
    crate::queue_workflow_metadata::parse_item_section(section, true).map_err(|err| match err {
        QueueWorkflowMetadataError::MissingItem => {
            QueueSnapshotError::Parse("Cargo.toml TODO section does not contain any item".into())
        }
        other => QueueSnapshotError::Parse(other.to_string()),
    })
}

fn rebuild_todo_block(prefix: &str, items: &[QueueTextItem], suffix: &str) -> String {
    rebuild_item_block(prefix, items, suffix)
}

fn read_optional_file(path: &Path) -> Result<Option<String>, QueueSnapshotError> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(QueueSnapshotError::Io(err)),
    }
}

fn first_runnable_backlog_item(
    backlog: &Path,
) -> Result<Option<QueueTextItem>, QueueSnapshotError> {
    let Some(backlog_text) = read_optional_file(backlog)? else {
        return Ok(None);
    };
    let (_, _, items) = split_backlog_items(&backlog_text)?;
    Ok(items.into_iter().find(|item| item.blocked_by.is_none()))
}

fn top_todo_scope(cargo_toml: &str) -> Result<TodoScope, QueueSnapshotError> {
    let todo_section =
        extract_marked_section(cargo_toml, TODO_START, TRUNK_MARKER).map_err(|_| {
            QueueSnapshotError::Parse("Cargo.toml changelog template missing TODO section".into())
        })?;
    let item = first_todo_item(todo_section).map_err(|_| {
        QueueSnapshotError::Parse("Cargo.toml TODO section does not contain any item".into())
    })?;
    let scope_line = item
        .lines
        .iter()
        .find(|line| line.contains("Scope:"))
        .ok_or_else(|| {
            QueueSnapshotError::Parse(format!("TODO `{}` is missing `Scope:`", item.title))
        })?;
    let boundary_line = item
        .lines
        .iter()
        .find(|line| line.contains("Boundary:"))
        .ok_or_else(|| {
            QueueSnapshotError::Parse(format!("TODO `{}` is missing `Boundary:`", item.title))
        })?;
    let contracts_line = item
        .lines
        .iter()
        .find(|line| line.contains("Contracts:"))
        .ok_or_else(|| {
            QueueSnapshotError::Parse(format!("TODO `{}` is missing `Contracts:`", item.title))
        })?;
    let scope = extract_backtick_values(scope_line);
    if scope.is_empty() {
        return Err(QueueSnapshotError::Parse(format!(
            "TODO `{}` must declare at least one backtick-quoted scope path",
            item.title
        )));
    }
    let boundary = extract_backtick_values(boundary_line)
        .into_iter()
        .next()
        .ok_or_else(|| {
            QueueSnapshotError::Parse(format!(
                "TODO `{}` must declare one backtick-quoted boundary value",
                item.title
            ))
        })?;
    let contracts = extract_backtick_values(contracts_line);
    if contracts.is_empty() {
        return Err(QueueSnapshotError::Parse(format!(
            "TODO `{}` must declare at least one backtick-quoted contract value",
            item.title
        )));
    }

    Ok(TodoScope {
        title: item.title,
        scope,
        boundary,
        contracts,
    })
}

fn is_docs_family_shared_overlap_only(todo: &TodoScope, changed_paths: &[String]) -> bool {
    if todo.boundary != "repo-workflow" || !todo.contracts.iter().any(|contract| contract == "docs")
    {
        return false;
    }

    let shared_scope_entries = todo
        .scope
        .iter()
        .filter_map(|path| match normalize_scope_path(path).as_str() {
            "docs/workflows.md" => Some("docs/workflows.md".to_string()),
            "docs/reference.md" => Some("docs/reference.md".to_string()),
            "tests/" => Some("tests/".to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();
    if shared_scope_entries.is_empty() {
        return false;
    }

    changed_paths.iter().all(|path| {
        shared_scope_entries.iter().any(|shared| {
            if shared.ends_with('/') {
                path.starts_with(shared)
            } else {
                path == shared
            }
        })
    })
}
