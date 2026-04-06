use std::path::Path;
use std::process::Command;

pub fn run_jj(root: &Path, args: &[&str]) {
    let status = Command::new("jj")
        .current_dir(root)
        .args(args)
        .status()
        .unwrap_or_else(|err| panic!("jj {}: {err}", args.join(" ")));
    assert!(status.success(), "jj {} must succeed", args.join(" "));
}

pub fn init_jj_repo(root: &Path) {
    run_jj(
        root,
        &[
            "--config",
            "user.name=Afterburner Tests",
            "--config",
            "user.email=afterburner-tests@example.com",
            "git",
            "init",
            ".",
        ],
    );
    run_jj(
        root,
        &[
            "--config",
            "user.name=Afterburner Tests",
            "--config",
            "user.email=afterburner-tests@example.com",
            "commit",
            "-m",
            "init",
        ],
    );
}
