use std::fs;
use std::path::Path;

use crate::queue_workflow_metadata::normalized_marked_section_block;
use sha2::{Digest, Sha256};

const TODO_START: &str = "## TODO";
const TODO_END: &str = "## [Trunk]";
const SNAPSHOT_PREFIX: &str = "<!-- queue-snapshot:";

#[derive(Debug, Clone, PartialEq, Eq)]
struct QueueSnapshot {
    todo_sha256: String,
    parent_commit: String,
}

#[derive(Debug)]
pub enum QueueSnapshotLineageError {
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for QueueSnapshotLineageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for QueueSnapshotLineageError {}

impl From<std::io::Error> for QueueSnapshotLineageError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub fn stamp_snapshot_files(
    cargo_toml: &Path,
    changelog: &Path,
    parent_commit: String,
) -> Result<(), QueueSnapshotLineageError> {
    let cargo_text = fs::read_to_string(cargo_toml)?;
    let todo_block = normalized_todo_block(&cargo_text)?;
    let snapshot = QueueSnapshot {
        todo_sha256: sha256_hex(todo_block.as_bytes()),
        parent_commit,
    };
    let changelog_text = fs::read_to_string(changelog)?;
    let updated = upsert_snapshot_comment(&changelog_text, &snapshot)?;
    fs::write(changelog, updated)?;
    Ok(())
}

pub fn verify_snapshot_files(
    cargo_toml: &Path,
    changelog: &Path,
    parent_commit: String,
    previous_parent_commit: Option<String>,
) -> Result<(), QueueSnapshotLineageError> {
    let cargo_text = fs::read_to_string(cargo_toml)?;
    let todo_block = normalized_todo_block(&cargo_text)?;
    let expected_sha = sha256_hex(todo_block.as_bytes());
    let changelog_text = fs::read_to_string(changelog)?;
    let snapshot = parse_snapshot_comment(&changelog_text)?;
    if snapshot.todo_sha256 != expected_sha {
        return Err(QueueSnapshotLineageError::Parse(format!(
            "queue snapshot hash mismatch: changelog has `{}`, current todo block is `{expected_sha}`",
            snapshot.todo_sha256
        )));
    }
    let matches_current_parent = snapshot.parent_commit == parent_commit;
    let matches_previous_parent = previous_parent_commit
        .as_deref()
        .is_some_and(|previous| snapshot.parent_commit == previous);
    if !matches_current_parent && !matches_previous_parent {
        let accepted = previous_parent_commit
            .as_ref()
            .map(|previous| format!("`{parent_commit}` or `{previous}`"))
            .unwrap_or_else(|| format!("`{parent_commit}`"));
        return Err(QueueSnapshotLineageError::Parse(format!(
            "queue snapshot lineage is stale: changelog has `{}`, expected {accepted}\nrun `just queue-resume` to repair and continue, or `just queue-refresh` if you only want to refresh the queue snapshot",
            snapshot.parent_commit
        )));
    }
    Ok(())
}

fn normalized_todo_block(cargo_toml: &str) -> Result<String, QueueSnapshotLineageError> {
    normalized_marked_section_block(cargo_toml, TODO_START, TODO_END).map_err(|_| {
        QueueSnapshotLineageError::Parse(
            "Cargo.toml changelog template missing TODO section".into(),
        )
    })
}

fn snapshot_comment(snapshot: &QueueSnapshot) -> String {
    format!(
        "<!-- queue-snapshot: todo_sha256={} parent_commit={} -->",
        snapshot.todo_sha256, snapshot.parent_commit
    )
}

fn upsert_snapshot_comment(
    changelog: &str,
    snapshot: &QueueSnapshot,
) -> Result<String, QueueSnapshotLineageError> {
    let comment = snapshot_comment(snapshot);
    if changelog.contains(SNAPSHOT_PREFIX) {
        let mut lines: Vec<String> = Vec::new();
        for line in changelog.lines() {
            if line.trim_start().starts_with(SNAPSHOT_PREFIX) {
                if matches!(lines.last(), Some(last) if !last.is_empty()) {
                    lines.push(String::new());
                }
                lines.push(comment.clone());
            } else {
                lines.push(line.to_string());
            }
        }
        return Ok(lines.join("\n"));
    }

    let marker = format!("\n{TODO_END}");
    let pos = changelog.find(&marker).ok_or_else(|| {
        QueueSnapshotLineageError::Parse("CHANGELOG.md missing `## [Trunk]` marker".into())
    })?;
    let mut out = String::new();
    out.push_str(&changelog[..pos]);
    if !out.ends_with("\n\n") {
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
    }
    out.push_str(&comment);
    out.push('\n');
    out.push_str(&changelog[pos..]);
    Ok(out)
}

fn parse_snapshot_comment(changelog: &str) -> Result<QueueSnapshot, QueueSnapshotLineageError> {
    let line = changelog
        .lines()
        .find(|line| line.trim_start().starts_with(SNAPSHOT_PREFIX))
        .ok_or_else(|| {
            QueueSnapshotLineageError::Parse("CHANGELOG.md missing queue snapshot comment".into())
        })?;

    let trimmed = line
        .trim()
        .trim_start_matches("<!--")
        .trim_end_matches("-->")
        .trim();
    let payload = trimmed
        .strip_prefix("queue-snapshot:")
        .ok_or_else(|| {
            QueueSnapshotLineageError::Parse("invalid queue snapshot comment prefix".into())
        })?
        .trim();

    let mut todo_sha256 = None::<String>;
    let mut parent_commit = None::<String>;
    for part in payload.split_whitespace() {
        if let Some(value) = part.strip_prefix("todo_sha256=") {
            todo_sha256 = Some(value.to_string());
        } else if let Some(value) = part.strip_prefix("parent_commit=") {
            parent_commit = Some(value.to_string());
        }
    }

    Ok(QueueSnapshot {
        todo_sha256: todo_sha256.ok_or_else(|| {
            QueueSnapshotLineageError::Parse("queue snapshot missing todo_sha256".into())
        })?,
        parent_commit: parent_commit.ok_or_else(|| {
            QueueSnapshotLineageError::Parse("queue snapshot missing parent_commit".into())
        })?,
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("{digest:x}")
}

#[cfg(test)]
mod tests {
    use super::{
        QueueSnapshot, normalized_todo_block, parse_snapshot_comment, sha256_hex,
        upsert_snapshot_comment,
    };

    #[test]
    fn normalized_todo_block_extracts_only_active_horizon() {
        let cargo = "prefix\n## TODO\n- a\n  - Goal: x  \n## [Trunk]\nrest";
        let block = normalized_todo_block(cargo).expect("todo block");
        assert_eq!(block, "- a\n  - Goal: x");
    }

    #[test]
    fn upsert_snapshot_comment_inserts_and_replaces() {
        let changelog = "# Changelog\n\n## TODO\n- a\n\n## [Trunk]\n";
        let snapshot = QueueSnapshot {
            todo_sha256: sha256_hex(b"todo"),
            parent_commit: "abc123".to_string(),
        };
        let stamped = upsert_snapshot_comment(changelog, &snapshot).expect("stamp");
        assert!(stamped.contains("queue-snapshot:"));
        let replaced = upsert_snapshot_comment(
            &stamped,
            &QueueSnapshot {
                todo_sha256: "def".to_string(),
                parent_commit: "fed".to_string(),
            },
        )
        .expect("replace");
        assert!(replaced.contains("todo_sha256=def parent_commit=fed"));
    }

    #[test]
    fn parse_snapshot_comment_reads_embedded_values() {
        let changelog =
            "# Changelog\n<!-- queue-snapshot: todo_sha256=abc parent_commit=def -->\n## [Trunk]\n";
        let snapshot = parse_snapshot_comment(changelog).expect("snapshot");
        assert_eq!(snapshot.todo_sha256, "abc");
        assert_eq!(snapshot.parent_commit, "def");
    }
}
