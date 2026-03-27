use std::fs;
use std::path::PathBuf;

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
enum QueueSnapshotError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for QueueSnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum Command {
    Stamp {
        cargo_toml: PathBuf,
        changelog: PathBuf,
        parent_commit: String,
    },
    Verify {
        cargo_toml: PathBuf,
        changelog: PathBuf,
        parent_commit: String,
    },
}

fn main() {
    let code = match parse_args(std::env::args().skip(1)) {
        Ok(command) => match run(command) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("{err}");
                2
            }
        },
        Err(err) => {
            eprintln!("{err}");
            2
        }
    };
    if code != 0 {
        std::process::exit(code);
    }
}

fn run(command: Command) -> Result<(), QueueSnapshotError> {
    match command {
        Command::Stamp {
            cargo_toml,
            changelog,
            parent_commit,
        } => {
            let cargo_text = fs::read_to_string(&cargo_toml)?;
            let todo_block = normalized_todo_block(&cargo_text)?;
            let snapshot = QueueSnapshot {
                todo_sha256: sha256_hex(todo_block.as_bytes()),
                parent_commit,
            };
            let changelog_text = fs::read_to_string(&changelog)?;
            let updated = upsert_snapshot_comment(&changelog_text, &snapshot)?;
            fs::write(&changelog, updated)?;
            Ok(())
        }
        Command::Verify {
            cargo_toml,
            changelog,
            parent_commit,
        } => {
            let cargo_text = fs::read_to_string(&cargo_toml)?;
            let todo_block = normalized_todo_block(&cargo_text)?;
            let expected_sha = sha256_hex(todo_block.as_bytes());
            let changelog_text = fs::read_to_string(&changelog)?;
            let snapshot = parse_snapshot_comment(&changelog_text)?;
            if snapshot.todo_sha256 != expected_sha {
                return Err(QueueSnapshotError::Parse(format!(
                    "queue snapshot hash mismatch: changelog has `{}`, current todo block is `{expected_sha}`",
                    snapshot.todo_sha256
                )));
            }
            if snapshot.parent_commit != parent_commit {
                return Err(QueueSnapshotError::Parse(format!(
                    "queue snapshot parent mismatch: changelog has `{}`, current parent is `{parent_commit}`",
                    snapshot.parent_commit
                )));
            }
            Ok(())
        }
    }
}

fn parse_args<I>(args: I) -> Result<Command, QueueSnapshotError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let Some(subcommand) = args.next() else {
        return Err(QueueSnapshotError::InvalidArg(usage().to_string()));
    };
    match subcommand.as_str() {
        "stamp" => parse_command_args(args, true),
        "verify" => parse_command_args(args, false),
        "--help" | "-h" | "help" => Err(QueueSnapshotError::InvalidArg(usage().to_string())),
        _ => Err(QueueSnapshotError::InvalidArg(format!(
            "unknown queue-snapshot subcommand `{subcommand}`\n{}",
            usage()
        ))),
    }
}

fn parse_command_args<I>(args: I, stamp: bool) -> Result<Command, QueueSnapshotError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut cargo_toml = PathBuf::from("Cargo.toml");
    let mut changelog = PathBuf::from("CHANGELOG.md");
    let mut parent_commit = None::<String>;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--cargo-toml" => cargo_toml = PathBuf::from(parse_value(&mut args, "--cargo-toml")?),
            "--changelog" => changelog = PathBuf::from(parse_value(&mut args, "--changelog")?),
            "--parent-commit" => parent_commit = Some(parse_value(&mut args, "--parent-commit")?),
            _ if arg.starts_with("--cargo-toml=") => {
                cargo_toml = PathBuf::from(arg.trim_start_matches("--cargo-toml=").to_string())
            }
            _ if arg.starts_with("--changelog=") => {
                changelog = PathBuf::from(arg.trim_start_matches("--changelog=").to_string())
            }
            _ if arg.starts_with("--parent-commit=") => {
                parent_commit = Some(arg.trim_start_matches("--parent-commit=").to_string())
            }
            _ => {
                return Err(QueueSnapshotError::InvalidArg(format!(
                    "unknown argument `{arg}`\n{}",
                    usage()
                )));
            }
        }
    }

    let parent_commit = parent_commit.ok_or_else(|| {
        QueueSnapshotError::InvalidArg(format!("missing value for --parent-commit\n{}", usage()))
    })?;

    Ok(if stamp {
        Command::Stamp {
            cargo_toml,
            changelog,
            parent_commit,
        }
    } else {
        Command::Verify {
            cargo_toml,
            changelog,
            parent_commit,
        }
    })
}

fn parse_value<I>(args: &mut I, flag: &str) -> Result<String, QueueSnapshotError>
where
    I: Iterator<Item = String>,
{
    args.next().ok_or_else(|| {
        QueueSnapshotError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })
}

fn normalized_todo_block(cargo_toml: &str) -> Result<String, QueueSnapshotError> {
    let section = cargo_toml
        .split(TODO_START)
        .nth(1)
        .and_then(|rest| rest.split(TODO_END).next())
        .ok_or_else(|| {
            QueueSnapshotError::Parse("Cargo.toml changelog template missing TODO section".into())
        })?;

    let normalized_lines = section.lines().map(str::trim_end).collect::<Vec<_>>();
    Ok(normalized_lines.join("\n").trim().to_string())
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
) -> Result<String, QueueSnapshotError> {
    let comment = snapshot_comment(snapshot);
    if changelog.contains(SNAPSHOT_PREFIX) {
        let mut lines = Vec::new();
        for line in changelog.lines() {
            if line.trim_start().starts_with(SNAPSHOT_PREFIX) {
                lines.push(comment.clone());
            } else {
                lines.push(line.to_string());
            }
        }
        return Ok(lines.join("\n"));
    }

    let marker = format!("\n{TODO_END}");
    let pos = changelog.find(&marker).ok_or_else(|| {
        QueueSnapshotError::Parse("CHANGELOG.md missing `## [Trunk]` marker".into())
    })?;
    let mut out = String::new();
    out.push_str(&changelog[..pos]);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&comment);
    out.push('\n');
    out.push_str(&changelog[pos..]);
    Ok(out)
}

fn parse_snapshot_comment(changelog: &str) -> Result<QueueSnapshot, QueueSnapshotError> {
    let line = changelog
        .lines()
        .find(|line| line.trim_start().starts_with(SNAPSHOT_PREFIX))
        .ok_or_else(|| {
            QueueSnapshotError::Parse("CHANGELOG.md missing queue snapshot comment".into())
        })?;

    let trimmed = line
        .trim()
        .trim_start_matches("<!--")
        .trim_end_matches("-->")
        .trim();
    let payload = trimmed
        .strip_prefix("queue-snapshot:")
        .ok_or_else(|| QueueSnapshotError::Parse("invalid queue snapshot comment prefix".into()))?
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
            QueueSnapshotError::Parse("queue snapshot missing todo_sha256".into())
        })?,
        parent_commit: parent_commit.ok_or_else(|| {
            QueueSnapshotError::Parse("queue snapshot missing parent_commit".into())
        })?,
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("{digest:x}")
}

fn usage() -> &'static str {
    "usage: afterburner_queue_snapshot <stamp|verify> [--cargo-toml PATH] [--changelog PATH] [--parent-commit ID]"
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
