use std::fs;
use std::path::Path;

use crate::queue_workflow_metadata::{
    QueueTextItem, QueueWorkflowMetadataError, extract_backtick_values, extract_marked_section,
    first_todo_item, normalize_scope_path, rebuild_item_block, split_items_after_marker,
    split_marked_items,
};

const TODO_START: &str = "## TODO";
const TRUNK_MARKER: &str = "## [Trunk]";
const BACKLOG_ITEMS_START: &str = "## Items";

#[derive(Debug)]
pub enum QueueOrderError {
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for QueueOrderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for QueueOrderError {}

impl From<std::io::Error> for QueueOrderError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TodoScope {
    pub(crate) title: String,
    pub(crate) scope: Vec<String>,
    pub(crate) boundary: String,
    pub(crate) contracts: Vec<String>,
}

pub fn check_top_runnable(cargo_toml: &Path, backlog: &Path) -> Result<(), QueueOrderError> {
    let cargo_text = fs::read_to_string(cargo_toml)?;
    let items = todo_items_allow_empty(&cargo_text)?;
    let Some(top) = items.first() else {
        return match first_runnable_backlog_item(backlog)? {
            Some(next) => Err(QueueOrderError::Parse(format!(
                "active TODO queue is empty\nnext runnable backlog item: `{}`\nrun `just queue-promote-next-runnable` to promote it into the active queue",
                next.title
            ))),
            None => Ok(()),
        };
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
        Some(next) => Err(QueueOrderError::Parse(format!(
            "top active TODO `{}` is blocked by `{}`\nnext runnable item: `{}`\nrun `just queue-promote-next-runnable` to repair the active queue order",
            top.title, blocked_by, next
        ))),
        None => match first_runnable_backlog_item(backlog)? {
            Some(next) => Err(QueueOrderError::Parse(format!(
                "top active TODO `{}` is blocked by `{}` and no runnable item exists in the active queue\nnext runnable backlog item: `{}`\nrun `just queue-promote-next-runnable` to promote it into the active queue",
                top.title, blocked_by, next.title
            ))),
            None => Err(QueueOrderError::Parse(format!(
                "top active TODO `{}` is blocked by `{}` and no runnable item exists in the active queue or backlog\nrepair the queue order or unblock a backlog item before execution",
                top.title, blocked_by
            ))),
        },
    }
}

pub fn promote_next_runnable(cargo_toml: &Path, backlog: &Path) -> Result<(), QueueOrderError> {
    let cargo_text = fs::read_to_string(cargo_toml)?;
    let (prefix, suffix, mut items) = split_todo_items(&cargo_text)?;
    let Some(first_runnable) = items.iter().position(|item| item.blocked_by.is_none()) else {
        return promote_from_backlog(cargo_toml, backlog, prefix, suffix, items);
    };
    if first_runnable == 0 {
        return Ok(());
    }
    let promoted = items.remove(first_runnable);
    items.insert(0, promoted);
    fs::write(cargo_toml, rebuild_todo_block(&prefix, &items, &suffix))?;
    Ok(())
}

pub fn top_todo_scope(cargo_toml: &str) -> Result<TodoScope, QueueOrderError> {
    let todo_section =
        extract_marked_section(cargo_toml, TODO_START, TRUNK_MARKER).map_err(|_| {
            QueueOrderError::Parse("Cargo.toml changelog template missing TODO section".into())
        })?;
    let item = first_todo_item(todo_section).map_err(|_| {
        QueueOrderError::Parse("Cargo.toml TODO section does not contain any item".into())
    })?;
    let scope_line = item
        .lines
        .iter()
        .find(|line| line.contains("Scope:"))
        .ok_or_else(|| {
            QueueOrderError::Parse(format!("TODO `{}` is missing `Scope:`", item.title))
        })?;
    let boundary_line = item
        .lines
        .iter()
        .find(|line| line.contains("Boundary:"))
        .ok_or_else(|| {
            QueueOrderError::Parse(format!("TODO `{}` is missing `Boundary:`", item.title))
        })?;
    let contracts_line = item
        .lines
        .iter()
        .find(|line| line.contains("Contracts:"))
        .ok_or_else(|| {
            QueueOrderError::Parse(format!("TODO `{}` is missing `Contracts:`", item.title))
        })?;
    let scope = extract_backtick_values(scope_line);
    if scope.is_empty() {
        return Err(QueueOrderError::Parse(format!(
            "TODO `{}` must declare at least one backtick-quoted scope path",
            item.title
        )));
    }
    let boundary = extract_backtick_values(boundary_line)
        .into_iter()
        .next()
        .ok_or_else(|| {
            QueueOrderError::Parse(format!(
                "TODO `{}` must declare one backtick-quoted boundary value",
                item.title
            ))
        })?;
    let contracts = extract_backtick_values(contracts_line);
    if contracts.is_empty() {
        return Err(QueueOrderError::Parse(format!(
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

pub fn is_docs_family_shared_overlap_only(todo: &TodoScope, changed_paths: &[String]) -> bool {
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

fn promote_from_backlog(
    cargo_toml: &Path,
    backlog: &Path,
    prefix: String,
    suffix: String,
    mut active_items: Vec<QueueTextItem>,
) -> Result<(), QueueOrderError> {
    let Some(backlog_text) = read_optional_file(backlog)? else {
        return Err(QueueOrderError::Parse(
            "cannot promote next runnable item because no runnable TODO exists in the active queue or backlog"
                .into(),
        ));
    };
    let (backlog_prefix, backlog_suffix, mut backlog_items) = split_backlog_items(&backlog_text)?;
    let Some(first_runnable_backlog) = backlog_items
        .iter()
        .position(|item| item.blocked_by.is_none())
    else {
        return Err(QueueOrderError::Parse(
            "cannot promote next runnable item because no runnable TODO exists in the active queue or backlog"
                .into(),
        ));
    };
    let promoted = backlog_items.remove(first_runnable_backlog);
    active_items.insert(0, promoted);
    fs::write(
        cargo_toml,
        rebuild_todo_block(&prefix, &active_items, &suffix),
    )?;
    fs::write(
        backlog,
        rebuild_item_block(&backlog_prefix, &backlog_items, &backlog_suffix),
    )?;
    Ok(())
}

fn split_todo_items(
    cargo_toml: &str,
) -> Result<(String, String, Vec<QueueTextItem>), QueueOrderError> {
    split_marked_items(cargo_toml, TODO_START, TRUNK_MARKER).map_err(|err| {
        QueueOrderError::Parse(match err {
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
) -> Result<(String, String, Vec<QueueTextItem>), QueueOrderError> {
    split_items_after_marker(backlog_text, BACKLOG_ITEMS_START).map_err(|err| {
        QueueOrderError::Parse(match err {
            QueueWorkflowMetadataError::MissingMarker { .. } => {
                "docs/backlog.md missing `## Items` section".into()
            }
            other => other.to_string(),
        })
    })
}

fn todo_items_allow_empty(cargo_toml: &str) -> Result<Vec<QueueTextItem>, QueueOrderError> {
    let section = extract_marked_section(cargo_toml, TODO_START, TRUNK_MARKER).map_err(|_| {
        QueueOrderError::Parse("Cargo.toml changelog template missing TODO section".into())
    })?;
    crate::queue_workflow_metadata::parse_item_section(section, false)
        .map_err(|other| QueueOrderError::Parse(other.to_string()))
}

fn rebuild_todo_block(prefix: &str, items: &[QueueTextItem], suffix: &str) -> String {
    rebuild_item_block(prefix, items, suffix)
}

fn read_optional_file(path: &Path) -> Result<Option<String>, QueueOrderError> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(QueueOrderError::Io(err)),
    }
}

fn first_runnable_backlog_item(backlog: &Path) -> Result<Option<QueueTextItem>, QueueOrderError> {
    let Some(backlog_text) = read_optional_file(backlog)? else {
        return Ok(None);
    };
    let (_, _, items) = split_backlog_items(&backlog_text)?;
    Ok(items.into_iter().find(|item| item.blocked_by.is_none()))
}
