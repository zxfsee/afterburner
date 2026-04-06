use std::fs;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

use afterburner::queue_workflow_metadata::{
    extract_backtick_values, extract_marked_section, first_todo_item, is_allowed_path,
    normalize_candidate_path, normalize_scope_path, normalized_marked_section_block,
};
use sha2::{Digest, Sha256};

const TODO_START: &str = "## TODO";
const TODO_END: &str = "## [Trunk]";
const BACKLOG_ITEMS_START: &str = "## Items";
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
        } => stamp_snapshot(cargo_toml, changelog, parent_commit),
        Command::StampCurrentParent {
            cargo_toml,
            changelog,
            repo_root,
        } => {
            let parent_commit = jj_commit_id(&repo_root, "@-")?;
            stamp_snapshot(cargo_toml, changelog, parent_commit)
        }
        Command::Verify {
            cargo_toml,
            changelog,
            parent_commit,
            previous_parent_commit,
        } => verify_snapshot(cargo_toml, changelog, parent_commit, previous_parent_commit),
        Command::VerifyCurrentLineage {
            cargo_toml,
            changelog,
            repo_root,
        } => {
            let parent_commit = jj_commit_id(&repo_root, "@-")?;
            let previous_parent_commit = jj_optional_commit_id(&repo_root, "@--")?;
            verify_snapshot(cargo_toml, changelog, parent_commit, previous_parent_commit)
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

fn stamp_snapshot(
    cargo_toml: PathBuf,
    changelog: PathBuf,
    parent_commit: String,
) -> Result<(), QueueSnapshotError> {
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

fn verify_snapshot(
    cargo_toml: PathBuf,
    changelog: PathBuf,
    parent_commit: String,
    previous_parent_commit: Option<String>,
) -> Result<(), QueueSnapshotError> {
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
    let matches_current_parent = snapshot.parent_commit == parent_commit;
    let matches_previous_parent = previous_parent_commit
        .as_deref()
        .is_some_and(|previous| snapshot.parent_commit == previous);
    if !matches_current_parent && !matches_previous_parent {
        let accepted = previous_parent_commit
            .as_ref()
            .map(|previous| format!("`{parent_commit}` or `{previous}`"))
            .unwrap_or_else(|| format!("`{parent_commit}`"));
        return Err(QueueSnapshotError::Parse(format!(
            "queue snapshot lineage is stale: changelog has `{}`, expected {accepted}\nrun `just queue-resume` to repair and continue, or `just queue-refresh` if you only want to refresh the queue snapshot",
            snapshot.parent_commit
        )));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TodoScope {
    title: String,
    scope: Vec<String>,
    boundary: String,
    contracts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TodoItem {
    title: String,
    lines: Vec<String>,
    blocked_by: Option<String>,
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
    mut active_items: Vec<TodoItem>,
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
    let objective_lock_bin = workflow_objective_lock_binary()?;
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
    verify_snapshot(
        cargo_toml.clone(),
        changelog.clone(),
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
    cargo_toml: &PathBuf,
    changelog: &PathBuf,
    repo_root: &PathBuf,
) -> Result<(), QueueSnapshotError> {
    let objective_lock_bin = workflow_objective_lock_binary()?;
    run_binary_command(
        repo_root.as_path(),
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
        repo_root.as_path(),
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
    run_command(repo_root.as_path(), "git-cliff", &["-o", changelog_str])?;
    let parent_commit = jj_commit_id(repo_root, "@-")?;
    stamp_snapshot(cargo_toml.clone(), changelog.clone(), parent_commit)?;
    let backlog = repo_root.join("docs/backlog.md");
    check_top_runnable(cargo_toml.clone(), backlog)?;
    let previous_parent_commit = jj_optional_commit_id(repo_root, "@--")?;
    let current_parent_commit = jj_commit_id(repo_root, "@-")?;
    verify_snapshot(
        cargo_toml.clone(),
        changelog.clone(),
        current_parent_commit,
        previous_parent_commit,
    )?;
    run_binary_command(repo_root.as_path(), &objective_lock_bin, &["clear"])?;
    Ok(())
}

fn run_command(
    repo_root: &std::path::Path,
    program: &str,
    args: &[&str],
) -> Result<(), QueueSnapshotError> {
    let output = ProcessCommand::new(program)
        .current_dir(repo_root)
        .args(args)
        .output()?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let details = if stderr.is_empty() { stdout } else { stderr };
    Err(QueueSnapshotError::Parse(format!(
        "{program} {} failed: {details}",
        args.join(" ")
    )))
}

fn run_binary_command(
    repo_root: &std::path::Path,
    program: &PathBuf,
    args: &[&str],
) -> Result<(), QueueSnapshotError> {
    let output = ProcessCommand::new(program)
        .current_dir(repo_root)
        .args(args)
        .output()?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let details = if stderr.is_empty() { stdout } else { stderr };
    Err(QueueSnapshotError::Parse(format!(
        "{} {} failed: {details}",
        program.display(),
        args.join(" ")
    )))
}

fn workflow_objective_lock_binary() -> Result<PathBuf, QueueSnapshotError> {
    let current_exe = std::env::current_exe()?;
    let sibling = current_exe
        .parent()
        .ok_or_else(|| {
            QueueSnapshotError::Parse("current executable has no parent directory".into())
        })?
        .join("workflow_objective_lock");
    if sibling.exists() {
        Ok(sibling)
    } else {
        Err(QueueSnapshotError::Parse(format!(
            "workflow objective lock helper binary is missing at `{}`; build the repo before running queue execute preflight",
            sibling.display()
        )))
    }
}

fn jj_commit_id(repo_root: &PathBuf, revset: &str) -> Result<String, QueueSnapshotError> {
    let output = ProcessCommand::new("jj")
        .current_dir(repo_root)
        .args([
            "log",
            "--ignore-working-copy",
            "-r",
            revset,
            "--no-graph",
            "-T",
            "commit_id",
        ])
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(QueueSnapshotError::Parse(format!(
            "jj log for revset `{revset}` failed: {stderr}"
        )));
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|err| QueueSnapshotError::Parse(format!("jj output is not utf8: {err}")))?;
    let commit = stdout.trim().to_string();
    if commit.is_empty() {
        return Err(QueueSnapshotError::Parse(format!(
            "jj log for revset `{revset}` returned no commit id"
        )));
    }
    Ok(commit)
}

fn jj_optional_commit_id(
    repo_root: &PathBuf,
    revset: &str,
) -> Result<Option<String>, QueueSnapshotError> {
    let output = ProcessCommand::new("jj")
        .current_dir(repo_root)
        .args([
            "log",
            "--ignore-working-copy",
            "-r",
            revset,
            "--no-graph",
            "-T",
            "commit_id",
        ])
        .output()?;
    if !output.status.success() {
        return Ok(None);
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|err| QueueSnapshotError::Parse(format!("jj output is not utf8: {err}")))?;
    let commit = stdout.trim().to_string();
    if commit.is_empty() {
        Ok(None)
    } else {
        Ok(Some(commit))
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
        "stamp-current-parent" => parse_lineage_command_args(args, true),
        "verify-current-lineage" => parse_lineage_command_args(args, false),
        "check-completion-boundary" => parse_completion_boundary_args(args),
        "check-top-runnable" => parse_top_runnable_args(args, false),
        "promote-next-runnable" => parse_top_runnable_args(args, true),
        "execute-preflight" => parse_execute_preflight_args(args),
        "--help" | "-h" | "help" => Err(QueueSnapshotError::InvalidArg(usage().to_string())),
        _ => Err(QueueSnapshotError::InvalidArg(format!(
            "unknown queue-snapshot subcommand `{subcommand}`\n{}",
            usage()
        ))),
    }
}

fn parse_top_runnable_args<I>(args: I, promote: bool) -> Result<Command, QueueSnapshotError>
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
            _ => {
                return Err(QueueSnapshotError::InvalidArg(format!(
                    "unknown argument `{arg}`\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(if promote {
        Command::PromoteNextRunnable {
            cargo_toml,
            backlog,
        }
    } else {
        Command::CheckTopRunnable {
            cargo_toml,
            backlog,
        }
    })
}

fn parse_execute_preflight_args<I>(args: I) -> Result<Command, QueueSnapshotError>
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
            _ => {
                return Err(QueueSnapshotError::InvalidArg(format!(
                    "unknown argument `{arg}`\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Command::ExecutePreflight {
        cargo_toml,
        changelog,
        repo_root,
        repair_stale_snapshot,
    })
}

fn parse_completion_boundary_args<I>(args: I) -> Result<Command, QueueSnapshotError>
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
            _ => {
                return Err(QueueSnapshotError::InvalidArg(format!(
                    "unknown argument `{arg}`\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Command::CheckCompletionBoundary {
        cargo_toml,
        repo_root,
    })
}

fn parse_command_args<I>(args: I, stamp: bool) -> Result<Command, QueueSnapshotError>
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
            previous_parent_commit,
        }
    })
}

fn parse_lineage_command_args<I>(args: I, stamp: bool) -> Result<Command, QueueSnapshotError>
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
            _ => {
                return Err(QueueSnapshotError::InvalidArg(format!(
                    "unknown argument `{arg}`\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(if stamp {
        Command::StampCurrentParent {
            cargo_toml,
            changelog,
            repo_root,
        }
    } else {
        Command::VerifyCurrentLineage {
            cargo_toml,
            changelog,
            repo_root,
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
    normalized_marked_section_block(cargo_toml, TODO_START, TODO_END).map_err(|_| {
        QueueSnapshotError::Parse("Cargo.toml changelog template missing TODO section".into())
    })
}

fn split_todo_items(
    cargo_toml: &str,
) -> Result<(String, String, Vec<TodoItem>), QueueSnapshotError> {
    let start = cargo_toml.find(TODO_START).ok_or_else(|| {
        QueueSnapshotError::Parse("Cargo.toml changelog template missing TODO section".into())
    })?;
    let suffix_start = cargo_toml[start..]
        .find(TODO_END)
        .map(|offset| start + offset)
        .ok_or_else(|| {
            QueueSnapshotError::Parse(
                "Cargo.toml changelog template missing `## [Trunk]` marker".into(),
            )
        })?;
    let prefix_end = start + TODO_START.len();
    let prefix = cargo_toml[..prefix_end].to_string();
    let section = cargo_toml[prefix_end..suffix_start].to_string();
    let suffix = cargo_toml[suffix_start..].to_string();
    Ok((prefix, suffix, parse_todo_items_section(&section)?))
}

fn split_backlog_items(
    backlog_text: &str,
) -> Result<(String, String, Vec<TodoItem>), QueueSnapshotError> {
    let start = backlog_text.find(BACKLOG_ITEMS_START).ok_or_else(|| {
        QueueSnapshotError::Parse("docs/backlog.md missing `## Items` section".into())
    })?;
    let prefix_end = start + BACKLOG_ITEMS_START.len();
    let prefix = backlog_text[..prefix_end].to_string();
    let section = backlog_text[prefix_end..].to_string();
    let items = parse_item_section(&section, false)?;
    Ok((prefix, String::new(), items))
}

fn todo_items(cargo_toml: &str) -> Result<Vec<TodoItem>, QueueSnapshotError> {
    let section = extract_marked_section(cargo_toml, TODO_START, TODO_END).map_err(|_| {
        QueueSnapshotError::Parse("Cargo.toml changelog template missing TODO section".into())
    })?;
    parse_item_section(section, true)
}

fn parse_todo_items_section(section: &str) -> Result<Vec<TodoItem>, QueueSnapshotError> {
    parse_item_section(section, true)
}

fn parse_item_section(
    section: &str,
    require_non_empty: bool,
) -> Result<Vec<TodoItem>, QueueSnapshotError> {
    let mut items = Vec::new();
    let mut current = Vec::<String>::new();

    for line in section.lines() {
        if line.starts_with("- ") {
            if !current.is_empty() {
                items.push(parse_todo_item(&current)?);
            }
            current = vec![line.to_string()];
        } else if !current.is_empty() {
            current.push(line.to_string());
        }
    }

    if !current.is_empty() {
        items.push(parse_todo_item(&current)?);
    }

    if require_non_empty && items.is_empty() {
        return Err(QueueSnapshotError::Parse(
            "Cargo.toml TODO section does not contain any item".into(),
        ));
    }
    Ok(items)
}

fn parse_todo_item(lines: &[String]) -> Result<TodoItem, QueueSnapshotError> {
    let title = lines[0]
        .strip_prefix("- ")
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .ok_or_else(|| QueueSnapshotError::Parse("TODO item missing title".into()))?
        .to_string();
    let blocked_by = lines.iter().find_map(|line| {
        line.trim()
            .strip_prefix("- Blocked-by:")
            .map(|value| value.trim().to_string())
    });
    Ok(TodoItem {
        title,
        lines: lines.to_vec(),
        blocked_by,
    })
}

fn rebuild_todo_block(prefix: &str, items: &[TodoItem], suffix: &str) -> String {
    rebuild_item_block(prefix, items, suffix)
}

fn rebuild_item_block(prefix: &str, items: &[TodoItem], suffix: &str) -> String {
    let mut out = String::new();
    out.push_str(prefix);
    out.push('\n');
    out.push('\n');
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        for line in &item.lines {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !suffix.starts_with('\n') {
        out.push('\n');
    }
    out.push_str(suffix);
    out
}

fn read_optional_file(path: &PathBuf) -> Result<Option<String>, QueueSnapshotError> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(QueueSnapshotError::Io(err)),
    }
}

fn first_runnable_backlog_item(backlog: &PathBuf) -> Result<Option<TodoItem>, QueueSnapshotError> {
    let Some(backlog_text) = read_optional_file(backlog)? else {
        return Ok(None);
    };
    let (_, _, items) = split_backlog_items(&backlog_text)?;
    Ok(items.into_iter().find(|item| item.blocked_by.is_none()))
}

fn top_todo_scope(cargo_toml: &str) -> Result<TodoScope, QueueSnapshotError> {
    let todo_section = extract_marked_section(cargo_toml, TODO_START, TODO_END).map_err(|_| {
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

fn jj_changed_paths_between(
    repo_root: &PathBuf,
    from_rev: &str,
    to_rev: &str,
) -> Result<Vec<String>, QueueSnapshotError> {
    let output = ProcessCommand::new("jj")
        .current_dir(repo_root)
        .args(["diff", "--from", from_rev, "--to", to_rev, "--name-only"])
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(QueueSnapshotError::Parse(format!(
            "jj diff for completion boundary failed: {stderr}"
        )));
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|err| QueueSnapshotError::Parse(format!("jj output is not utf8: {err}")))?;
    Ok(stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(normalize_candidate_path)
        .collect())
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
        QueueSnapshotError::Parse("CHANGELOG.md missing `## [Trunk]` marker".into())
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
    "usage: workflow_queue_snapshot <stamp|verify|stamp-current-parent|verify-current-lineage|check-completion-boundary|check-top-runnable|promote-next-runnable|execute-preflight> [options]\n\
stamp: [--cargo-toml PATH] [--changelog PATH] --parent-commit ID\n\
verify: [--cargo-toml PATH] [--changelog PATH] --parent-commit ID [--previous-parent-commit ID]\n\
stamp-current-parent: [--cargo-toml PATH] [--changelog PATH] [--repo-root PATH]\n\
verify-current-lineage: [--cargo-toml PATH] [--changelog PATH] [--repo-root PATH]\n\
check-completion-boundary: [--cargo-toml PATH] [--repo-root PATH]\n\
check-top-runnable: [--cargo-toml PATH] [--backlog PATH]\n\
promote-next-runnable: [--cargo-toml PATH] [--backlog PATH]\n\
execute-preflight: [--cargo-toml PATH] [--changelog PATH] [--repo-root PATH] [--repair-stale-snapshot]"
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
