use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub enum WorkflowProcessError {
    Io(std::io::Error),
    Utf8(String),
    ProcessFailure(String),
    MissingSiblingBinary { name: String, path: PathBuf },
    EmptyOutput(String),
}

impl std::fmt::Display for WorkflowProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Utf8(msg) => write!(f, "{msg}"),
            Self::ProcessFailure(msg) => write!(f, "{msg}"),
            Self::MissingSiblingBinary { name: _, path } => write!(
                f,
                "workflow objective lock helper binary is missing at `{}`; build the repo before running queue execute preflight",
                path.display()
            ),
            Self::EmptyOutput(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for WorkflowProcessError {}

impl From<std::io::Error> for WorkflowProcessError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub fn run_command(
    repo_root: &Path,
    program: &str,
    args: &[&str],
) -> Result<(), WorkflowProcessError> {
    let output = Command::new(program)
        .current_dir(repo_root)
        .args(args)
        .output()?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let details = if stderr.is_empty() { stdout } else { stderr };
    Err(WorkflowProcessError::ProcessFailure(format!(
        "{program} {} failed: {details}",
        args.join(" ")
    )))
}

pub fn run_binary_command(
    repo_root: &Path,
    program: &Path,
    args: &[&str],
) -> Result<(), WorkflowProcessError> {
    let output = Command::new(program)
        .current_dir(repo_root)
        .args(args)
        .output()?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let details = if stderr.is_empty() { stdout } else { stderr };
    Err(WorkflowProcessError::ProcessFailure(format!(
        "{} {} failed: {details}",
        program.display(),
        args.join(" ")
    )))
}

pub fn sibling_binary_path(name: &str) -> Result<PathBuf, WorkflowProcessError> {
    let current_exe = std::env::current_exe()?;
    let sibling = current_exe
        .parent()
        .ok_or_else(|| {
            WorkflowProcessError::ProcessFailure(
                "current executable has no parent directory".into(),
            )
        })?
        .join(name);
    if sibling.exists() {
        Ok(sibling)
    } else {
        Err(WorkflowProcessError::MissingSiblingBinary {
            name: name.to_string(),
            path: sibling,
        })
    }
}

pub fn jj_commit_id(repo_root: &Path, revset: &str) -> Result<String, WorkflowProcessError> {
    let output = Command::new("jj")
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
        return Err(WorkflowProcessError::ProcessFailure(format!(
            "jj log for revset `{revset}` failed: {stderr}"
        )));
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|err| WorkflowProcessError::Utf8(format!("jj output is not utf8: {err}")))?;
    let commit = stdout.trim().to_string();
    if commit.is_empty() {
        return Err(WorkflowProcessError::EmptyOutput(format!(
            "jj log for revset `{revset}` returned no commit id"
        )));
    }
    Ok(commit)
}

pub fn jj_optional_commit_id(
    repo_root: &Path,
    revset: &str,
) -> Result<Option<String>, WorkflowProcessError> {
    let output = Command::new("jj")
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
        .map_err(|err| WorkflowProcessError::Utf8(format!("jj output is not utf8: {err}")))?;
    let commit = stdout.trim().to_string();
    if commit.is_empty() {
        Ok(None)
    } else {
        Ok(Some(commit))
    }
}

pub fn jj_output_lines(
    repo_root: &Path,
    args: &[&str],
) -> Result<Vec<String>, WorkflowProcessError> {
    let output = Command::new("jj")
        .current_dir(repo_root)
        .args(args)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(WorkflowProcessError::ProcessFailure(format!(
            "jj {} failed: {stderr}",
            args.join(" ")
        )));
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|err| WorkflowProcessError::Utf8(format!("jj output is not utf8: {err}")))?;
    Ok(stdout.lines().map(ToString::to_string).collect())
}

pub fn jj_changed_paths_between(
    repo_root: &Path,
    from_rev: &str,
    to_rev: &str,
) -> Result<Vec<String>, WorkflowProcessError> {
    let output = Command::new("jj")
        .current_dir(repo_root)
        .args(["diff", "--from", from_rev, "--to", to_rev, "--name-only"])
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(WorkflowProcessError::ProcessFailure(format!(
            "jj diff for completion boundary failed: {stderr}"
        )));
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|err| WorkflowProcessError::Utf8(format!("jj output is not utf8: {err}")))?;
    Ok(stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(ToString::to_string)
        .collect())
}
