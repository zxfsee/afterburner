use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::queue_workflow_metadata::{
    QueueWorkflowMetadataError, extract_backtick_values, extract_marked_section, first_todo_item,
    is_allowed_path, normalize_candidate_path,
};
use crate::workflow_objective_lock_cli::Objective;
use crate::workflow_objective_lock_state::hash_path;
pub use crate::workflow_objective_lock_state::{
    ObjectiveLock, ObjectiveLockError, read_lock, validate_action, validate_paths,
    validate_worktree_paths, write_lock,
};
use crate::workflow_process::{WorkflowProcessError, jj_output_lines};

const TODO_START: &str = "## TODO";
const TODO_END: &str = "## [Trunk]";

impl From<WorkflowProcessError> for ObjectiveLockError {
    fn from(value: WorkflowProcessError) -> Self {
        match value {
            WorkflowProcessError::Io(err) => Self::Io(err),
            other => Self::Parse(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoScope {
    pub title: String,
    pub scope: Vec<String>,
}

pub fn build_lock(
    objective: &Objective,
    cargo_toml: &Path,
    repo_root: &Path,
    expected_action: Option<String>,
    allow_existing_paths: &[String],
) -> Result<ObjectiveLock, ObjectiveLockError> {
    let lock = match objective {
        Objective::QueueOnly => ObjectiveLock {
            objective: objective.clone(),
            expected_action,
            allowed_paths: vec![
                "Cargo.toml".to_string(),
                "CHANGELOG.md".to_string(),
                "docs/backlog.md".to_string(),
            ],
            forbidden_paths: vec![
                "src/".to_string(),
                "tests/".to_string(),
                "fixtures/".to_string(),
                "docs/workflows.md".to_string(),
            ],
            source_title: None,
            carried_worktree_paths: capture_existing_paths(repo_root, allow_existing_paths)?,
        },
        Objective::TopScopeFix => ObjectiveLock {
            objective: objective.clone(),
            expected_action,
            allowed_paths: vec!["Cargo.toml".to_string(), "CHANGELOG.md".to_string()],
            forbidden_paths: vec![
                "docs/backlog.md".to_string(),
                "src/".to_string(),
                "tests/".to_string(),
                "fixtures/".to_string(),
                "docs/workflows.md".to_string(),
            ],
            source_title: None,
            carried_worktree_paths: capture_top_scope_fix_paths(
                cargo_toml,
                repo_root,
                allow_existing_paths,
            )?,
        },
        Objective::BacklogOnly => ObjectiveLock {
            objective: objective.clone(),
            expected_action,
            allowed_paths: vec!["docs/backlog.md".to_string()],
            forbidden_paths: vec![
                "Cargo.toml".to_string(),
                "CHANGELOG.md".to_string(),
                "src/".to_string(),
                "tests/".to_string(),
            ],
            source_title: None,
            carried_worktree_paths: capture_existing_paths(repo_root, allow_existing_paths)?,
        },
        Objective::DocsOnly => ObjectiveLock {
            objective: objective.clone(),
            expected_action,
            allowed_paths: vec![
                "README.md".to_string(),
                "ARCHITECTURE.md".to_string(),
                "docs/".to_string(),
            ],
            forbidden_paths: vec![
                "Cargo.toml".to_string(),
                "CHANGELOG.md".to_string(),
                "src/".to_string(),
                "tests/".to_string(),
                "fixtures/".to_string(),
            ],
            source_title: None,
            carried_worktree_paths: capture_existing_paths(repo_root, allow_existing_paths)?,
        },
        Objective::ReviewOnly => ObjectiveLock {
            objective: objective.clone(),
            expected_action,
            allowed_paths: Vec::new(),
            forbidden_paths: vec!["<any repo mutation>".to_string()],
            source_title: None,
            carried_worktree_paths: capture_existing_paths(repo_root, allow_existing_paths)?,
        },
        Objective::ExecuteTopItem => {
            let cargo_text = fs::read_to_string(cargo_toml)?;
            let top_scope = top_todo_scope(&cargo_text)?;
            ObjectiveLock {
                objective: objective.clone(),
                expected_action,
                allowed_paths: top_scope.scope,
                forbidden_paths: vec![
                    "Cargo.toml".to_string(),
                    "CHANGELOG.md".to_string(),
                    "docs/backlog.md".to_string(),
                ],
                source_title: Some(top_scope.title),
                carried_worktree_paths: capture_existing_paths(repo_root, allow_existing_paths)?,
            }
        }
    };
    Ok(lock)
}

pub fn top_todo_scope(cargo_toml: &str) -> Result<TodoScope, ObjectiveLockError> {
    let todo_section =
        extract_marked_section(cargo_toml, TODO_START, TODO_END).map_err(|err| match err {
            QueueWorkflowMetadataError::MissingMarkedSection { .. } => ObjectiveLockError::Parse(
                "Cargo.toml changelog template missing TODO section".into(),
            ),
            other => ObjectiveLockError::Parse(other.to_string()),
        })?;
    let item = first_todo_item(todo_section).map_err(|err| match err {
        QueueWorkflowMetadataError::MissingItem => {
            ObjectiveLockError::Parse("Cargo.toml TODO section does not contain any item".into())
        }
        other => ObjectiveLockError::Parse(other.to_string()),
    })?;
    let scope_line = item
        .lines
        .iter()
        .find(|line| line.contains("Scope:"))
        .ok_or_else(|| {
            ObjectiveLockError::Parse(format!("TODO `{}` is missing `Scope:`", item.title))
        })?;
    let scope = extract_backtick_values(scope_line);
    if scope.is_empty() {
        return Err(ObjectiveLockError::Parse(format!(
            "TODO `{}` must declare at least one backtick-quoted scope path",
            item.title
        )));
    }

    Ok(TodoScope {
        title: item.title,
        scope,
    })
}

pub fn check_repo_locks(repo_root: &Path) -> Result<(), ObjectiveLockError> {
    let index_lock = repo_root.join(".git/index.lock");
    if index_lock.exists() {
        let metadata = fs::metadata(&index_lock)?;
        let size_note = if metadata.len() == 0 {
            " (empty lockfile; likely stale)"
        } else {
            ""
        };
        return Err(ObjectiveLockError::Parse(format!(
            "repo metadata lock detected at `{}`{size_note}\nrun `just repo-lock-repair` to remove an aged stale lock, or inspect live Git/jj processes before retrying",
            index_lock.display()
        )));
    }
    Ok(())
}

pub fn repair_repo_locks(
    repo_root: &Path,
    stale_after_seconds: u64,
) -> Result<(), ObjectiveLockError> {
    let index_lock = repo_root.join(".git/index.lock");
    if !index_lock.exists() {
        return Ok(());
    }

    let metadata = fs::metadata(&index_lock)?;
    let modified = metadata.modified().map_err(|err| {
        ObjectiveLockError::Parse(format!(
            "repo metadata lock detected at `{}` but its timestamp could not be read: {err}",
            index_lock.display()
        ))
    })?;
    let age = std::time::SystemTime::now()
        .duration_since(modified)
        .unwrap_or_else(|_| std::time::Duration::from_secs(0))
        .as_secs();

    if age < stale_after_seconds {
        return Err(ObjectiveLockError::Parse(format!(
            "repo metadata lock at `{}` is only {}s old; refusing automatic removal before the stale threshold of {}s\ninspect live Git/jj processes or retry later",
            index_lock.display(),
            age,
            stale_after_seconds
        )));
    }

    fs::remove_file(&index_lock)?;
    eprintln!(
        "removed stale repo metadata lock `{}` (age: {}s, threshold: {}s)",
        index_lock.display(),
        age,
        stale_after_seconds
    );
    Ok(())
}

pub fn jj_changed_paths(repo_root: &Path) -> Result<Vec<String>, ObjectiveLockError> {
    let changed = jj_output_lines(repo_root, &["diff", "--name-only"])?;
    let mut paths = BTreeSet::new();
    for path in changed {
        if !path.is_empty() {
            paths.insert(normalize_candidate_path(&path));
        }
    }

    Ok(paths.into_iter().collect())
}

fn capture_existing_paths(
    repo_root: &Path,
    paths: &[String],
) -> Result<BTreeMap<String, String>, ObjectiveLockError> {
    let mut carried = BTreeMap::new();
    for path in paths {
        let normalized = normalize_candidate_path(path);
        let full_path = repo_root.join(&normalized);
        carried.insert(normalized, hash_path(&full_path)?);
    }
    Ok(carried)
}

fn capture_top_scope_fix_paths(
    cargo_toml: &Path,
    repo_root: &Path,
    allow_existing_paths: &[String],
) -> Result<BTreeMap<String, String>, ObjectiveLockError> {
    let mut paths = allow_existing_paths
        .iter()
        .map(|path| normalize_candidate_path(path))
        .collect::<BTreeSet<_>>();

    if cargo_toml.exists()
        && (repo_root.join(".jj").exists() || repo_root.join(".git").exists())
        && let Ok(cargo_text) = fs::read_to_string(cargo_toml)
        && let Ok(top_scope) = top_todo_scope(&cargo_text)
        && let Ok(changed_paths) = jj_changed_paths(repo_root)
    {
        for path in changed_paths {
            if path != "Cargo.toml"
                && path != "CHANGELOG.md"
                && is_allowed_path(&top_scope.scope, &path)
            {
                paths.insert(path);
            }
        }
    }

    let collected = paths.into_iter().collect::<Vec<_>>();
    capture_existing_paths(repo_root, &collected)
}

#[cfg(test)]
mod tests {
    use super::{Objective, build_lock, top_todo_scope, validate_paths};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    #[test]
    fn top_todo_scope_reads_first_scope_list() {
        let cargo = "[package.metadata.git-cliff.changelog]\nheader = \"## TODO\n\n- A\n  - Scope: `src/bin/`, `tests/`, `justfile`\n\n- B\n  - Scope: `docs/`\n\n## [Trunk]\"\n";
        let scope = top_todo_scope(cargo).expect("scope");
        assert_eq!(scope.title, "A");
        assert_eq!(scope.scope, vec!["src/bin/", "tests/", "justfile"]);
    }

    #[test]
    fn execute_top_item_lock_uses_top_scope() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cargo_toml = dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            "[package.metadata.git-cliff.changelog]\nheader = \"## TODO\n\n- Turn objective lock guard [Runtime Infra]\n  - Scope: `src/bin/`, `tests/`, `justfile`, `NOTE.md`\n\n## [Trunk]\"\n",
        )
        .expect("write Cargo.toml");
        let lock = build_lock(
            &Objective::ExecuteTopItem,
            &PathBuf::from(&cargo_toml),
            dir.path(),
            Some("execute-top-item".into()),
            &[],
        )
        .expect("lock");
        assert_eq!(
            lock.allowed_paths,
            vec!["src/bin/", "tests/", "justfile", "NOTE.md"]
        );
        assert_eq!(
            lock.source_title.as_deref(),
            Some("Turn objective lock guard [Runtime Infra]")
        );
    }

    #[test]
    fn top_scope_fix_lock_only_allows_queue_metadata_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cargo_toml = dir.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            "[package.metadata.git-cliff.changelog]\nheader = \"## TODO\n\n- Turn objective lock guard [Runtime Infra]\n  - Scope: `src/bin/`, `tests/`, `justfile`, `NOTE.md`\n\n## [Trunk]\"\n",
        )
        .expect("write Cargo.toml");
        let lock = build_lock(
            &Objective::TopScopeFix,
            &PathBuf::from(&cargo_toml),
            dir.path(),
            Some("top-scope-fix".into()),
            &[],
        )
        .expect("lock");
        assert_eq!(lock.allowed_paths, vec!["Cargo.toml", "CHANGELOG.md"]);
        assert_eq!(lock.source_title, None);
    }

    #[test]
    fn validate_paths_flags_out_of_scope_mutations() {
        let lock = super::ObjectiveLock {
            objective: Objective::QueueOnly,
            expected_action: None,
            allowed_paths: vec!["Cargo.toml".into()],
            forbidden_paths: vec![],
            source_title: None,
            carried_worktree_paths: BTreeMap::new(),
        };
        let err = validate_paths(&lock, &["src/lib.rs".into()]).expect_err("reject path");
        assert!(err.to_string().contains("rejected paths"));
    }
}
