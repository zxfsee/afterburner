use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use afterburner::command_artifacts::{
    JsonArtifactError, emit_json_artifact_written, write_json_value,
};
use serde_json::{Value, json};

const DEFAULT_OUT_PATH: &str = "artifacts/report/burn_bpk_migration_surface_inventory.json";
const CURRENT_BURN_VERSION: &str = "0.20.1";
const OBSERVED_PRE_RELEASE: &str = "0.21.0-pre.1";
const BURN_REFRESH_ADR: &str = "docs/adr/033-burn-dependency-refresh.md";

const SCAN_TARGETS: &[&str] = &[
    "Cargo.toml",
    "src",
    "tests",
    "fixtures",
    "README.md",
    "ARCHITECTURE.md",
    "docs/reference.md",
    "docs/adr",
];

#[derive(Debug)]
enum BurnBpkMigrationSurfaceInventoryError {
    InvalidArg(String),
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for BurnBpkMigrationSurfaceInventoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArg(msg) => write!(f, "{msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for BurnBpkMigrationSurfaceInventoryError {}

impl From<std::io::Error> for BurnBpkMigrationSurfaceInventoryError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<JsonArtifactError> for BurnBpkMigrationSurfaceInventoryError {
    fn from(value: JsonArtifactError) -> Self {
        match value {
            JsonArtifactError::Io(err) => Self::Io(err),
            JsonArtifactError::Serialize(err) => Self::Parse(format!(
                "serialize burn `.bpk` migration surface inventory json: {err}"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    repo_root: PathBuf,
    out_path: PathBuf,
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

fn run_inner<I>(args: I) -> Result<(), BurnBpkMigrationSurfaceInventoryError>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    let repo_root = args.repo_root.canonicalize()?;
    let mut touchpoints = collect_touchpoints(repo_root.as_path())?;
    touchpoints.sort_by(|lhs, rhs| lhs.path.cmp(&rhs.path));

    let summary = summary(&touchpoints);
    let report = json!({
        "schema_version": "1",
        "current_burn_version": CURRENT_BURN_VERSION,
        "observed_pre_release": OBSERVED_PRE_RELEASE,
        "scan_scope": SCAN_TARGETS,
        "touchpoints": touchpoints.iter().map(Touchpoint::as_json).collect::<Vec<_>>(),
        "summary": summary,
    });

    write_json_value(&args.out_path, &report)?;
    emit_json_artifact_written(
        "debug_cli",
        "burn_bpk_migration_surface_inventory_written",
        "inventory_path",
        &args.out_path,
        "inventory",
        report,
    );
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Touchpoint {
    path: String,
    tags: BTreeSet<&'static str>,
}

impl Touchpoint {
    fn as_json(&self) -> Value {
        json!({
            "path": self.path,
            "tags": self.tags.iter().copied().collect::<Vec<_>>(),
        })
    }
}

fn collect_touchpoints(
    root: &Path,
) -> Result<Vec<Touchpoint>, BurnBpkMigrationSurfaceInventoryError> {
    let mut files = Vec::new();
    for target in SCAN_TARGETS {
        gather_files(&root.join(target), root, &mut files)?;
    }

    let mut touchpoints = Vec::new();
    for path in files {
        let text = match fs::read_to_string(root.join(path.as_str())) {
            Ok(text) => text,
            Err(err) if err.kind() == std::io::ErrorKind::InvalidData => continue,
            Err(err) => return Err(err.into()),
        };
        let mut tags = BTreeSet::new();
        if text.contains(".mpk") {
            tags.insert("mpk_literal");
        }
        if text.contains(".bpk") {
            tags.insert("bpk_literal");
        }
        if text.contains("burn-mpk") {
            tags.insert("burn_mpk_export_format");
        }
        if text.contains(CURRENT_BURN_VERSION) {
            tags.insert("burn_current_version");
        }
        if text.contains(OBSERVED_PRE_RELEASE) {
            tags.insert("burn_pre_release");
        }
        if path == BURN_REFRESH_ADR {
            tags.insert("burn_refresh_decision");
        }

        if !tags.is_empty() {
            touchpoints.push(Touchpoint { path, tags });
        }
    }
    Ok(touchpoints)
}

fn gather_files(
    path: &Path,
    root: &Path,
    files: &mut Vec<String>,
) -> Result<(), BurnBpkMigrationSurfaceInventoryError> {
    if path.is_file() {
        files.push(relative(root, path)?);
        return Ok(());
    }
    if !path.is_dir() {
        return Err(BurnBpkMigrationSurfaceInventoryError::Parse(format!(
            "scan target `{}` must exist",
            path.display()
        )));
    }

    let mut entries = fs::read_dir(path)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort();
    for entry in entries {
        gather_files(&entry, root, files)?;
    }
    Ok(())
}

fn relative(root: &Path, path: &Path) -> Result<String, BurnBpkMigrationSurfaceInventoryError> {
    let relative = path.strip_prefix(root).map_err(|_| {
        BurnBpkMigrationSurfaceInventoryError::Parse(format!(
            "path `{}` must live under repo root `{}`",
            path.display(),
            root.display()
        ))
    })?;
    Ok(relative.to_string_lossy().to_string())
}

fn summary(touchpoints: &[Touchpoint]) -> Value {
    let mut tag_counts = BTreeMap::<&str, u64>::new();
    for touchpoint in touchpoints {
        for tag in &touchpoint.tags {
            *tag_counts.entry(tag).or_default() += 1;
        }
    }
    let tag_counts = tag_counts
        .into_iter()
        .map(|(tag, count)| (tag.to_string(), Value::from(count)))
        .collect::<serde_json::Map<_, _>>();

    json!({
        "touchpoint_count": touchpoints.len(),
        "tag_counts": tag_counts,
    })
}

fn parse_args<I>(args: I) -> Result<Args, BurnBpkMigrationSurfaceInventoryError>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut repo_root = PathBuf::from(".");
    let mut out_path = PathBuf::from(DEFAULT_OUT_PATH);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => {
                repo_root = PathBuf::from(parse_value::<String, _>(&mut args, "--repo-root")?)
            }
            "--out" => out_path = PathBuf::from(parse_value::<String, _>(&mut args, "--out")?),
            _ if arg.starts_with("--repo-root=") => {
                repo_root = PathBuf::from(arg.trim_start_matches("--repo-root=").to_string())
            }
            _ if arg.starts_with("--out=") => {
                out_path = PathBuf::from(arg.trim_start_matches("--out=").to_string())
            }
            _ => {
                return Err(BurnBpkMigrationSurfaceInventoryError::InvalidArg(format!(
                    "unknown argument for debug inventory-burn-bpk-surface: {arg}\n{}",
                    usage()
                )));
            }
        }
    }

    Ok(Args {
        repo_root,
        out_path,
    })
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, BurnBpkMigrationSurfaceInventoryError>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args.next().ok_or_else(|| {
        BurnBpkMigrationSurfaceInventoryError::InvalidArg(format!(
            "missing value for {flag}\n{}",
            usage()
        ))
    })?;
    value.parse::<T>().map_err(|_| {
        BurnBpkMigrationSurfaceInventoryError::InvalidArg(format!(
            "invalid value for {flag}\n{}",
            usage()
        ))
    })
}

fn usage() -> &'static str {
    "usage: afterburner debug inventory-burn-bpk-surface [--repo-root PATH] [--out PATH]"
}
