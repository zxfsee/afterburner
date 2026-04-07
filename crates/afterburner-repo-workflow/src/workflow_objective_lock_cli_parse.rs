use std::iter::Peekable;
use std::path::PathBuf;

use crate::workflow_objective_lock_cli::{CommandSpec, Objective, usage};

const DEFAULT_LOCK_PATH: &str = ".git/afterburner/objective-lock.json";

pub(super) fn parse_pin_args<I>(args: Peekable<I>) -> Result<CommandSpec, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args;
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
            _ => return Err(format!("unknown argument `{arg}`\n{}", usage())),
        }
    }

    let objective =
        objective.ok_or_else(|| format!("missing required flag --objective\n{}", usage()))?;

    Ok(CommandSpec::Pin {
        objective,
        cargo_toml,
        repo_root,
        lock_file,
        expected_action,
        allow_existing_paths,
    })
}

pub(super) fn parse_check_paths_args<I>(args: Peekable<I>) -> Result<CommandSpec, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args;
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
            _ => return Err(format!("unknown argument `{arg}`\n{}", usage())),
        }
    }

    if paths.is_empty() {
        return Err(format!(
            "check-paths requires at least one --path\n{}",
            usage()
        ));
    }

    Ok(CommandSpec::CheckPaths {
        lock_file,
        action,
        paths,
    })
}

pub(super) fn parse_check_worktree_args<I>(args: Peekable<I>) -> Result<CommandSpec, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args;
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
            _ => return Err(format!("unknown argument `{arg}`\n{}", usage())),
        }
    }

    Ok(CommandSpec::CheckWorktree {
        lock_file,
        repo_root,
        action,
    })
}

pub(super) fn parse_clear_args<I>(args: Peekable<I>) -> Result<CommandSpec, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args;
    let mut lock_file = PathBuf::from(DEFAULT_LOCK_PATH);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--lock-file" => lock_file = PathBuf::from(parse_value(&mut args, "--lock-file")?),
            _ if arg.starts_with("--lock-file=") => {
                lock_file = PathBuf::from(arg.trim_start_matches("--lock-file=").to_string())
            }
            _ => return Err(format!("unknown argument `{arg}`\n{}", usage())),
        }
    }

    Ok(CommandSpec::Clear { lock_file })
}

pub(super) fn parse_check_repo_locks_args<I>(args: Peekable<I>) -> Result<CommandSpec, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args;
    let mut repo_root = PathBuf::from(".");

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => repo_root = PathBuf::from(parse_value(&mut args, "--repo-root")?),
            _ if arg.starts_with("--repo-root=") => {
                repo_root = PathBuf::from(arg.trim_start_matches("--repo-root=").to_string())
            }
            _ => return Err(format!("unknown argument `{arg}`\n{}", usage())),
        }
    }

    Ok(CommandSpec::CheckRepoLocks { repo_root })
}

pub(super) fn parse_repair_repo_locks_args<I>(args: Peekable<I>) -> Result<CommandSpec, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args;
    let mut repo_root = PathBuf::from(".");
    let mut stale_after_seconds = 300_u64;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => repo_root = PathBuf::from(parse_value(&mut args, "--repo-root")?),
            "--stale-after-seconds" => {
                stale_after_seconds = parse_value(&mut args, "--stale-after-seconds")?
                    .parse::<u64>()
                    .map_err(|_| {
                        "`--stale-after-seconds` must be an unsigned integer".to_string()
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
                        "`--stale-after-seconds` must be an unsigned integer".to_string()
                    })?;
            }
            _ => return Err(format!("unknown argument `{arg}`\n{}", usage())),
        }
    }

    Ok(CommandSpec::RepairRepoLocks {
        repo_root,
        stale_after_seconds,
    })
}

fn parse_value<I>(args: &mut Peekable<I>, flag: &str) -> Result<String, String>
where
    I: Iterator<Item = String>,
{
    args.next()
        .ok_or_else(|| format!("missing value for {flag}\n{}", usage()))
}
