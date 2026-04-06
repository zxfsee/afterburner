use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use afterburner::queue_workflow_metadata::{
    QueueWorkflowMetadataError, extract_backtick_values, extract_marked_section, first_todo_item,
    is_allowed_path, normalize_candidate_path,
};
use afterburner::workflow_process::{WorkflowProcessError, jj_output_lines};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const DEFAULT_LOCK_PATH: &str = ".git/afterburner/objective-lock.json";
const TODO_START: &str = "## TODO";
const TODO_END: &str = "## [Trunk]";

#[derive(Debug, Clone, PartialEq, Eq)]
enum Objective {
    QueueOnly,
    TopScopeFix,
    BacklogOnly,
    ExecuteTopItem,
    DocsOnly,
    ReviewOnly,
}

impl Objective {
    fn parse(raw: &str) -> Result<Self, ObjectiveLockError> {
        match raw {
            "queue-only" => Ok(Self::QueueOnly),
            "top-scope-fix" => Ok(Self::TopScopeFix),
            "backlog-only" => Ok(Self::BacklogOnly),
            "execute-top-item" => Ok(Self::ExecuteTopItem),
            "docs-only" => Ok(Self::DocsOnly),
            "review-only" => Ok(Self::ReviewOnly),
            _ => Err(ObjectiveLockError::InvalidArg(format!(
                "unknown objective `{raw}`\n{}",
                usage()
            ))),
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::QueueOnly => "queue-only",
            Self::TopScopeFix => "top-scope-fix",
            Self::BacklogOnly => "backlog-only",
            Self::ExecuteTopItem => "execute-top-item",
            Self::DocsOnly => "docs-only",
            Self::ReviewOnly => "review-only",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ObjectiveLock {
    objective: Objective,
    expected_action: Option<String>,
    allowed_paths: Vec<String>,
    forbidden_paths: Vec<String>,
    source_title: Option<String>,
    carried_worktree_paths: BTreeMap<String, String>,
}

#[derive(Debug)]
enum ObjectiveLockError {
    InvalidArg(String),
    Io(std::io::Error),
    Json(serde_json::Error),
    Parse(String),
}

impl std::fmt::Display for ObjectiveLockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Json(err) => write!(f, "json error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for ObjectiveLockError {}

impl From<std::io::Error> for ObjectiveLockError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for ObjectiveLockError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<WorkflowProcessError> for ObjectiveLockError {
    fn from(value: WorkflowProcessError) -> Self {
        match value {
            WorkflowProcessError::Io(err) => Self::Io(err),
            other => Self::Parse(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CommandSpec {
    Pin {
        objective: Objective,
        cargo_toml: PathBuf,
        repo_root: PathBuf,
        lock_file: PathBuf,
        expected_action: Option<String>,
        allow_existing_paths: Vec<String>,
    },
    CheckPaths {
        lock_file: PathBuf,
        action: Option<String>,
        paths: Vec<String>,
    },
    CheckWorktree {
        lock_file: PathBuf,
        repo_root: PathBuf,
        action: Option<String>,
    },
    CheckRepoLocks {
        repo_root: PathBuf,
    },
    RepairRepoLocks {
        repo_root: PathBuf,
        stale_after_seconds: u64,
    },
    Clear {
        lock_file: PathBuf,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TodoScope {
    title: String,
    scope: Vec<String>,
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

fn run(command: CommandSpec) -> Result<(), ObjectiveLockError> {
    match command {
        CommandSpec::Pin {
            objective,
            cargo_toml,
            repo_root,
            lock_file,
            expected_action,
            allow_existing_paths,
        } => {
            let lock = build_lock(
                &objective,
                &cargo_toml,
                &repo_root,
                expected_action,
                &allow_existing_paths,
            )?;
            write_lock(&lock_file, &lock)?;
            Ok(())
        }
        CommandSpec::CheckPaths {
            lock_file,
            action,
            paths,
        } => {
            let lock = read_lock(&lock_file)?;
            validate_action(&lock, action.as_deref())?;
            validate_paths(&lock, &paths)
        }
        CommandSpec::CheckWorktree {
            lock_file,
            repo_root,
            action,
        } => {
            let lock = read_lock(&lock_file)?;
            validate_action(&lock, action.as_deref())?;
            let changed_paths = jj_changed_paths(&repo_root)?;
            validate_worktree_paths(&lock, &repo_root, &changed_paths)
        }
        CommandSpec::CheckRepoLocks { repo_root } => check_repo_locks(&repo_root),
        CommandSpec::RepairRepoLocks {
            repo_root,
            stale_after_seconds,
        } => repair_repo_locks(&repo_root, stale_after_seconds),
        CommandSpec::Clear { lock_file } => {
            if lock_file.exists() {
                fs::remove_file(lock_file)?;
            }
            Ok(())
        }
    }
}

fn parse_args<I>(args: I) -> Result<CommandSpec, ObjectiveLockError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let Some(subcommand) = args.next() else {
        return Err(ObjectiveLockError::InvalidArg(usage().to_string()));
    };

    match subcommand.as_str() {
        "pin" => parse_pin_args(args),
        "check-paths" => parse_check_paths_args(args),
        "check-worktree" => parse_check_worktree_args(args),
        "check-repo-locks" => parse_check_repo_locks_args(args),
        "repair-repo-locks" => parse_repair_repo_locks_args(args),
        "clear" => parse_clear_args(args),
        "--help" | "-h" | "help" => Err(ObjectiveLockError::InvalidArg(usage().to_string())),
        _ => Err(ObjectiveLockError::InvalidArg(format!(
            "unknown objective-lock subcommand `{subcommand}`\n{}",
            usage()
        ))),
    }
}

fn parse_pin_args<I>(args: I) -> Result<CommandSpec, ObjectiveLockError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut objective = None::<Objective>;
    let mut cargo_toml = PathBuf::from("Cargo.toml");
    let mut repo_root = PathBuf::from(".");
    let mut lock_file = PathBuf::from(DEFAULT_LOCK_PATH);
    let mut expected_action = None::<String>;
    let mut allow_existing_paths = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--objective" => {
                objective = Some(Objective::parse(&parse_value(&mut args, "--objective")?)?)
            }
            "--cargo-toml" => cargo_toml = PathBuf::from(parse_value(&mut args, "--cargo-toml")?),
            "--repo-root" => repo_root = PathBuf::from(parse_value(&mut args, "--repo-root")?),
            "--lock-file" => lock_file = PathBuf::from(parse_value(&mut args, "--lock-file")?),
            "--expected-action" => {
                expected_action = Some(parse_value(&mut args, "--expected-action")?)
            }
            "--allow-existing-path" => {
                allow_existing_paths.push(parse_value(&mut args, "--allow-existing-path")?)
            }
            _ if arg.starts_with("--objective=") => {
                objective = Some(Objective::parse(arg.trim_start_matches("--objective="))?)
            }
            _ if arg.starts_with("--cargo-toml=") => {
                cargo_toml = PathBuf::from(arg.trim_start_matches("--cargo-toml=").to_string())
            }
            _ if arg.starts_with("--repo-root=") => {
                repo_root = PathBuf::from(arg.trim_start_matches("--repo-root=").to_string())
            }
            _ if arg.starts_with("--lock-file=") => {
                lock_file = PathBuf::from(arg.trim_start_matches("--lock-file=").to_string())
            }
            _ if arg.starts_with("--expected-action=") => {
                expected_action = Some(arg.trim_start_matches("--expected-action=").to_string())
            }
            _ if arg.starts_with("--allow-existing-path=") => allow_existing_paths
                .push(arg.trim_start_matches("--allow-existing-path=").to_string()),
            _ => {
                return Err(ObjectiveLockError::InvalidArg(format!(
                    "unknown argument `{arg}`\n{}",
                    usage()
                )));
            }
        }
    }

    let objective = objective.ok_or_else(|| {
        ObjectiveLockError::InvalidArg(format!("missing value for --objective\n{}", usage()))
    })?;

    Ok(CommandSpec::Pin {
        objective,
        cargo_toml,
        repo_root,
        lock_file,
        expected_action,
        allow_existing_paths,
    })
}

fn parse_check_paths_args<I>(args: I) -> Result<CommandSpec, ObjectiveLockError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut lock_file = PathBuf::from(DEFAULT_LOCK_PATH);
    let mut action = None::<String>;
    let mut paths = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--lock-file" => lock_file = PathBuf::from(parse_value(&mut args, "--lock-file")?),
            "--action" => action = Some(parse_value(&mut args, "--action")?),
            "--path" => paths.push(parse_value(&mut args, "--path")?),
            _ if arg.starts_with("--lock-file=") => {
                lock_file = PathBuf::from(arg.trim_start_matches("--lock-file=").to_string())
            }
            _ if arg.starts_with("--action=") => {
                action = Some(arg.trim_start_matches("--action=").to_string())
            }
            _ if arg.starts_with("--path=") => {
                paths.push(arg.trim_start_matches("--path=").to_string())
            }
            _ => {
                return Err(ObjectiveLockError::InvalidArg(format!(
                    "unknown argument `{arg}`\n{}",
                    usage()
                )));
            }
        }
    }

    if paths.is_empty() {
        return Err(ObjectiveLockError::InvalidArg(format!(
            "check-paths requires at least one --path\n{}",
            usage()
        )));
    }

    Ok(CommandSpec::CheckPaths {
        lock_file,
        action,
        paths,
    })
}

fn parse_check_worktree_args<I>(args: I) -> Result<CommandSpec, ObjectiveLockError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut lock_file = PathBuf::from(DEFAULT_LOCK_PATH);
    let mut repo_root = PathBuf::from(".");
    let mut action = None::<String>;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--lock-file" => lock_file = PathBuf::from(parse_value(&mut args, "--lock-file")?),
            "--repo-root" => repo_root = PathBuf::from(parse_value(&mut args, "--repo-root")?),
            "--action" => action = Some(parse_value(&mut args, "--action")?),
            _ if arg.starts_with("--lock-file=") => {
                lock_file = PathBuf::from(arg.trim_start_matches("--lock-file=").to_string())
            }
            _ if arg.starts_with("--repo-root=") => {
                repo_root = PathBuf::from(arg.trim_start_matches("--repo-root=").to_string())
            }
            _ if arg.starts_with("--action=") => {
                action = Some(arg.trim_start_matches("--action=").to_string())
            }
            _ => {
                return Err(ObjectiveLockError::InvalidArg(format!(
                    "unknown argument `{arg}`\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(CommandSpec::CheckWorktree {
        lock_file,
        repo_root,
        action,
    })
}

fn parse_clear_args<I>(args: I) -> Result<CommandSpec, ObjectiveLockError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut lock_file = PathBuf::from(DEFAULT_LOCK_PATH);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--lock-file" => lock_file = PathBuf::from(parse_value(&mut args, "--lock-file")?),
            _ if arg.starts_with("--lock-file=") => {
                lock_file = PathBuf::from(arg.trim_start_matches("--lock-file=").to_string())
            }
            _ => {
                return Err(ObjectiveLockError::InvalidArg(format!(
                    "unknown argument `{arg}`\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(CommandSpec::Clear { lock_file })
}

fn parse_check_repo_locks_args<I>(args: I) -> Result<CommandSpec, ObjectiveLockError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut repo_root = PathBuf::from(".");

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => repo_root = PathBuf::from(parse_value(&mut args, "--repo-root")?),
            _ if arg.starts_with("--repo-root=") => {
                repo_root = PathBuf::from(arg.trim_start_matches("--repo-root=").to_string())
            }
            _ => {
                return Err(ObjectiveLockError::InvalidArg(format!(
                    "unknown argument `{arg}`\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(CommandSpec::CheckRepoLocks { repo_root })
}

fn parse_repair_repo_locks_args<I>(args: I) -> Result<CommandSpec, ObjectiveLockError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut repo_root = PathBuf::from(".");
    let mut stale_after_seconds = 300_u64;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => repo_root = PathBuf::from(parse_value(&mut args, "--repo-root")?),
            "--stale-after-seconds" => {
                stale_after_seconds = parse_value(&mut args, "--stale-after-seconds")?
                    .parse::<u64>()
                    .map_err(|_| {
                        ObjectiveLockError::InvalidArg(
                            "`--stale-after-seconds` must be an unsigned integer".into(),
                        )
                    })?;
            }
            _ if arg.starts_with("--repo-root=") => {
                repo_root = PathBuf::from(arg.trim_start_matches("--repo-root=").to_string())
            }
            _ if arg.starts_with("--stale-after-seconds=") => {
                stale_after_seconds = arg
                    .trim_start_matches("--stale-after-seconds=")
                    .parse::<u64>()
                    .map_err(|_| {
                        ObjectiveLockError::InvalidArg(
                            "`--stale-after-seconds` must be an unsigned integer".into(),
                        )
                    })?;
            }
            _ => {
                return Err(ObjectiveLockError::InvalidArg(format!(
                    "unknown argument `{arg}`\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(CommandSpec::RepairRepoLocks {
        repo_root,
        stale_after_seconds,
    })
}

fn parse_value<I>(args: &mut I, flag: &str) -> Result<String, ObjectiveLockError>
where
    I: Iterator<Item = String>,
{
    args.next().ok_or_else(|| {
        ObjectiveLockError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })
}

fn build_lock(
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

fn write_lock(path: &Path, lock: &ObjectiveLock) -> Result<(), ObjectiveLockError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut payload = json!({
        "schema_version": 1,
        "objective": lock.objective.as_str(),
        "allowed_paths": lock.allowed_paths.clone(),
        "forbidden_paths": lock.forbidden_paths.clone(),
    });
    if let Some(expected_action) = &lock.expected_action {
        payload["expected_action"] = Value::String(expected_action.clone());
    }
    if let Some(source_title) = &lock.source_title {
        payload["source_title"] = Value::String(source_title.clone());
    }
    if !lock.carried_worktree_paths.is_empty() {
        payload["carried_worktree_paths"] = json!(lock.carried_worktree_paths);
    }
    fs::write(path, serde_json::to_vec_pretty(&payload)?)?;
    Ok(())
}

fn read_lock(path: &Path) -> Result<ObjectiveLock, ObjectiveLockError> {
    let raw = fs::read_to_string(path).map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            ObjectiveLockError::Parse(format!(
                "objective lock not found at `{}`; pin one first",
                path.display()
            ))
        } else {
            ObjectiveLockError::Io(err)
        }
    })?;
    let value: Value = serde_json::from_str(&raw)?;

    let objective = Objective::parse(string_field(&value, "objective")?)?;
    let expected_action = optional_string_field(&value, "expected_action");
    let source_title = optional_string_field(&value, "source_title");
    let allowed_paths = string_array_field(&value, "allowed_paths")?;
    let forbidden_paths = string_array_field(&value, "forbidden_paths")?;
    let carried_worktree_paths = optional_string_map_field(&value, "carried_worktree_paths")?;

    Ok(ObjectiveLock {
        objective,
        expected_action,
        allowed_paths,
        forbidden_paths,
        source_title,
        carried_worktree_paths,
    })
}

fn string_field<'a>(value: &'a Value, field: &str) -> Result<&'a str, ObjectiveLockError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| ObjectiveLockError::Parse(format!("objective lock missing `{field}`")))
}

fn optional_string_field(value: &Value, field: &str) -> Option<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn string_array_field(value: &Value, field: &str) -> Result<Vec<String>, ObjectiveLockError> {
    let Some(array) = value.get(field).and_then(Value::as_array) else {
        return Err(ObjectiveLockError::Parse(format!(
            "objective lock missing `{field}`"
        )));
    };
    array
        .iter()
        .map(|entry| {
            entry.as_str().map(ToString::to_string).ok_or_else(|| {
                ObjectiveLockError::Parse(format!(
                    "objective lock field `{field}` must contain only strings"
                ))
            })
        })
        .collect()
}

fn optional_string_map_field(
    value: &Value,
    field: &str,
) -> Result<BTreeMap<String, String>, ObjectiveLockError> {
    let Some(object) = value.get(field) else {
        return Ok(BTreeMap::new());
    };
    let Some(map) = object.as_object() else {
        return Err(ObjectiveLockError::Parse(format!(
            "objective lock field `{field}` must be an object"
        )));
    };
    map.iter()
        .map(|(key, value)| {
            value
                .as_str()
                .map(|raw| (key.clone(), raw.to_string()))
                .ok_or_else(|| {
                    ObjectiveLockError::Parse(format!(
                        "objective lock field `{field}` must map to string hashes"
                    ))
                })
        })
        .collect()
}

fn validate_action(lock: &ObjectiveLock, action: Option<&str>) -> Result<(), ObjectiveLockError> {
    match (&lock.expected_action, action) {
        (Some(expected), Some(actual)) if expected != actual => {
            Err(ObjectiveLockError::Parse(format!(
                "objective lock expects action `{expected}`, got `{actual}` for objective `{}`",
                lock.objective.as_str()
            )))
        }
        (Some(expected), None) => Err(ObjectiveLockError::Parse(format!(
            "objective lock expects action `{expected}` for objective `{}`",
            lock.objective.as_str()
        ))),
        _ => Ok(()),
    }
}

fn validate_paths(lock: &ObjectiveLock, paths: &[String]) -> Result<(), ObjectiveLockError> {
    let mut rejected = Vec::new();
    for path in paths.iter().map(|path| normalize_candidate_path(path)) {
        if !is_allowed_path(&lock.allowed_paths, &path) {
            rejected.push(path);
        }
    }

    if rejected.is_empty() {
        return Ok(());
    }

    let scope_source = lock
        .source_title
        .as_ref()
        .map(|title| format!(" from top TODO `{title}`"))
        .unwrap_or_default();
    let mut message = format!(
        "objective `{}`{} forbids repo mutations outside {:?}; rejected paths: {:?}",
        lock.objective.as_str(),
        scope_source,
        lock.allowed_paths,
        rejected
    );
    if lock.objective == Objective::ExecuteTopItem
        && rejected
            .iter()
            .all(|path| path == "Cargo.toml" || path == "CHANGELOG.md")
    {
        message.push_str(
            "\nif you are repairing stale top TODO scope metadata, run `just queue-fix-top-scope` first",
        );
    }
    Err(ObjectiveLockError::Parse(message))
}

fn validate_worktree_paths(
    lock: &ObjectiveLock,
    repo_root: &Path,
    paths: &[String],
) -> Result<(), ObjectiveLockError> {
    let filtered = paths
        .iter()
        .filter_map(|path| {
            let normalized = normalize_candidate_path(path);
            match lock.carried_worktree_paths.get(&normalized) {
                Some(expected_hash)
                    if file_sha256_hex(&repo_root.join(&normalized))
                        .is_ok_and(|current_hash| current_hash == *expected_hash) =>
                {
                    None
                }
                _ => Some(normalized),
            }
        })
        .collect::<Vec<_>>();
    validate_paths(lock, &filtered)
}

fn top_todo_scope(cargo_toml: &str) -> Result<TodoScope, ObjectiveLockError> {
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

fn jj_changed_paths(repo_root: &Path) -> Result<Vec<String>, ObjectiveLockError> {
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
        carried.insert(normalized, file_sha256_hex(&full_path)?);
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

fn file_sha256_hex(path: &Path) -> Result<String, ObjectiveLockError> {
    let bytes = fs::read(path)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn check_repo_locks(repo_root: &Path) -> Result<(), ObjectiveLockError> {
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

fn repair_repo_locks(repo_root: &Path, stale_after_seconds: u64) -> Result<(), ObjectiveLockError> {
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

fn usage() -> &'static str {
    "usage: workflow_objective_lock <pin|check-paths|check-worktree|check-repo-locks|repair-repo-locks|clear> [options]\n\
pin: --objective <queue-only|top-scope-fix|backlog-only|execute-top-item|docs-only|review-only> [--cargo-toml PATH] [--repo-root PATH] [--lock-file PATH] [--expected-action ACTION] [--allow-existing-path PATH...]\n\
check-paths: [--lock-file PATH] [--action ACTION] --path PATH [--path PATH...]\n\
check-worktree: [--lock-file PATH] [--repo-root PATH] [--action ACTION]\n\
check-repo-locks: [--repo-root PATH]\n\
repair-repo-locks: [--repo-root PATH] [--stale-after-seconds N]\n\
clear: [--lock-file PATH]"
}

#[cfg(test)]
mod tests {
    use super::{Objective, build_lock, extract_backtick_values, is_allowed_path, top_todo_scope};
    use std::path::PathBuf;

    #[test]
    fn top_todo_scope_reads_first_scope_list() {
        let cargo = "[package.metadata.git-cliff.changelog]\nheader = \"## TODO\n\n- A\n  - Scope: `src/bin/`, `tests/`, `justfile`\n\n- B\n  - Scope: `docs/`\n\n## [Trunk]\"\n";
        let scope = top_todo_scope(cargo).expect("scope");
        assert_eq!(scope.title, "A");
        assert_eq!(scope.scope, vec!["src/bin/", "tests/", "justfile"]);
    }

    #[test]
    fn extract_backtick_values_keeps_only_quoted_paths() {
        let values = extract_backtick_values("- Scope: `src/`, `tests/`, `docs/workflows.md`");
        assert_eq!(values, vec!["src/", "tests/", "docs/workflows.md"]);
    }

    #[test]
    fn path_allowlist_matches_files_and_directories() {
        assert!(is_allowed_path(&["Cargo.toml".into()], "Cargo.toml"));
        assert!(is_allowed_path(&["src/".into()], "src/bin/lock.rs"));
        assert!(!is_allowed_path(&["src/".into()], "tests/lock.rs"));
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
}
