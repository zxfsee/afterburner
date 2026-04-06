use std::fs;

use afterburner::workflow_objective_lock_cli::{CommandSpec, parse_args};
use afterburner::workflow_objective_lock_core::{
    ObjectiveLockError, build_lock, check_repo_locks, jj_changed_paths, read_lock,
    repair_repo_locks, validate_action, validate_paths, validate_worktree_paths, write_lock,
};

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
