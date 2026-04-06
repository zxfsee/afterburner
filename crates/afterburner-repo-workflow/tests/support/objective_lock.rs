use std::fs;
use std::path::Path;
use std::process::Command;

pub fn write_sample_cargo_toml(path: &Path) {
    fs::write(
        path,
        "[package.metadata.git-cliff.changelog]\nheader = \"\"\"# Changelog\n\n## TODO\n\n- Turn objective lock guard [Runtime Infra]\n  - Goal: Pin the current execution objective class and reject out-of-scope repo mutations before work proceeds so queue-only, backlog-only, docs-only, and execute-item turns cannot silently drift into each other.\n  - Kind: `mixed`\n  - Boundary: `repo-workflow`\n  - Contracts: `ops`\n  - Scope: `src/bin/`, `tests/`, `justfile`, `docs/workflows.md`, `NOTE.md`\n\n## [Trunk]\n\"\"\"\n",
    )
    .expect("write Cargo.toml");
}

pub fn init_jj_repo(root: &Path) {
    run_jj(
        root,
        &[
            "--config",
            "user.name='Afterburner Tests'",
            "--config",
            "user.email='afterburner-tests@example.com'",
            "git",
            "init",
            ".",
        ],
    );
    run_jj(
        root,
        &[
            "--config",
            "user.name='Afterburner Tests'",
            "--config",
            "user.email='afterburner-tests@example.com'",
            "commit",
            "-m",
            "init",
        ],
    );
}

pub fn run_jj(root: &Path, args: &[&str]) {
    let status = Command::new("jj")
        .current_dir(root)
        .args(args)
        .status()
        .unwrap_or_else(|err| panic!("jj {}: {err}", args.join(" ")));
    assert!(status.success(), "jj {} must succeed", args.join(" "));
}
