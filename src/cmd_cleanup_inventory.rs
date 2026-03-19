use std::fs;
use std::path::{Path, PathBuf};

use afterburner::observability::emit_event;
use serde_json::{Value, json};

const ARTIFACT_CLEANUP_INVENTORY_PATH: &str = "artifacts/deploy/artifact_cleanup_inventory.json";

#[derive(Debug)]
enum CleanupInventoryError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for CleanupInventoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for CleanupInventoryError {}

impl From<std::io::Error> for CleanupInventoryError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    artifacts_root: PathBuf,
    out_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct InventoryEntry {
    path: String,
    category: &'static str,
    reason: &'static str,
}

pub fn run<I>(args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let args: Vec<String> = args.collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{}", usage());
        return 0;
    }

    match run_inner(args.into_iter()) {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("{err}");
            2
        }
    }
}

fn run_inner<I>(args: I) -> Result<(), CleanupInventoryError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let inference_root = args.artifacts_root.join("inference");
    let current_pointer = inference_root.join("current");
    let active_inference_version = fs::read_to_string(&current_pointer)?.trim().to_string();
    if active_inference_version.is_empty() {
        return Err(CleanupInventoryError::Parse(format!(
            "inference current pointer `{}` must not be empty",
            current_pointer.display()
        )));
    }

    let active_dir = inference_root.join(active_inference_version.as_str());
    if !active_dir.is_dir() {
        return Err(CleanupInventoryError::Parse(format!(
            "active inference directory `{}` must exist",
            active_dir.display()
        )));
    }

    let mut retained = vec![
        InventoryEntry {
            path: relative_path(args.artifacts_root.as_path(), current_pointer.as_path())?,
            category: "runtime_pointer",
            reason: "active inference pointer must be retained",
        },
        InventoryEntry {
            path: relative_path(args.artifacts_root.as_path(), active_dir.as_path())?,
            category: "active_inference_directory",
            reason: "referenced by current inference pointer",
        },
    ];

    for (path, category, reason) in [
        (
            args.artifacts_root.join("deploy"),
            "rollout_evidence",
            "active rollout evidence stays retained",
        ),
        (
            args.artifacts_root.join("train"),
            "train_contracts",
            "train-side decision artifacts stay retained",
        ),
        (
            args.artifacts_root.join("eval"),
            "eval_state",
            "current eval and drift state stays retained",
        ),
    ] {
        if path.exists() {
            retained.push(InventoryEntry {
                path: relative_path(args.artifacts_root.as_path(), path.as_path())?,
                category,
                reason,
            });
        }
    }

    let mut prune_candidates = Vec::new();
    if inference_root.is_dir() {
        for entry in fs::read_dir(&inference_root)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let version_name =
                path.file_name()
                    .and_then(|name| name.to_str())
                    .ok_or_else(|| {
                        CleanupInventoryError::Parse(format!(
                            "failed to read inference directory name for `{}`",
                            path.display()
                        ))
                    })?;
            if version_name == active_inference_version {
                continue;
            }
            prune_candidates.push(InventoryEntry {
                path: relative_path(args.artifacts_root.as_path(), path.as_path())?,
                category: "inactive_inference_directory",
                reason: "not referenced by current inference pointer",
            });
        }
    }

    let profiling_path = args.artifacts_root.join("profiling");
    if profiling_path.exists() {
        prune_candidates.push(InventoryEntry {
            path: relative_path(args.artifacts_root.as_path(), profiling_path.as_path())?,
            category: "profiling_artifacts",
            reason: "profiling artifacts are prune candidates once no active runtime or rollout state references them",
        });
    }

    retained.sort();
    prune_candidates.sort();

    let inventory = json!({
        "schema_version": "1",
        "artifacts_root": args.artifacts_root.display().to_string(),
        "active_inference_version": active_inference_version,
        "retained": retained.iter().map(entry_json).collect::<Vec<_>>(),
        "prune_candidates": prune_candidates.iter().map(entry_json).collect::<Vec<_>>(),
    });

    if let Some(parent) = args.out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&inventory).map_err(|err| {
        CleanupInventoryError::Parse(format!("serialize cleanup inventory json: {err}"))
    })?;
    fs::write(&args.out_path, text)?;

    emit_event(
        "info",
        "ops_cli",
        "artifact_cleanup_inventory_written",
        json!({
            "inventory_path": args.out_path.display().to_string(),
            "inventory": inventory,
        }),
    );
    Ok(())
}

fn entry_json(entry: &InventoryEntry) -> Value {
    json!({
        "path": entry.path,
        "category": entry.category,
        "reason": entry.reason,
    })
}

fn relative_path(root: &Path, path: &Path) -> Result<String, CleanupInventoryError> {
    let relative = path.strip_prefix(root).map_err(|_| {
        CleanupInventoryError::Parse(format!(
            "path `{}` must live under artifacts root `{}`",
            path.display(),
            root.display()
        ))
    })?;
    Ok(relative.to_string_lossy().to_string())
}

fn parse_args<I>(args: I) -> Result<Args, CleanupInventoryError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut artifacts_root = PathBuf::from("artifacts");
    let mut out_path = PathBuf::from(ARTIFACT_CLEANUP_INVENTORY_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--artifacts-root" => {
                let value: String = parse_value(&mut args, "--artifacts-root")?;
                artifacts_root = PathBuf::from(value);
            }
            "--out" => {
                let value: String = parse_value(&mut args, "--out")?;
                out_path = PathBuf::from(value);
            }
            _ if arg.starts_with("--artifacts-root=") => {
                artifacts_root =
                    PathBuf::from(arg.trim_start_matches("--artifacts-root=").to_string())
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(CleanupInventoryError::InvalidArg(format!(
                    "unknown argument for cleanup-inventory: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        artifacts_root,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, CleanupInventoryError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        CleanupInventoryError::InvalidArg(format!("missing value for {flag}\n{}", usage()))
    })?;
    value.parse::<T>().map_err(|_| {
        CleanupInventoryError::InvalidArg(format!("invalid value for {flag}\n{}", usage()))
    })
}

fn usage() -> &'static str {
    "usage: afterburner cleanup-inventory [--artifacts-root PATH] [--out PATH]"
}
