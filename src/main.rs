mod cmd_cleanup_inventory;
mod cmd_cleanup_policy;
mod cmd_deployment_stack_check;
mod cmd_deployment_verification_bundle;
mod cmd_distributed_load_profile;
mod cmd_drift_baseline;
mod cmd_drift_baseline_approval;
mod cmd_drift_baseline_bundle;
mod cmd_drift_baseline_checkpoint;
mod cmd_drift_baseline_handoff;
mod cmd_drift_baseline_history;
mod cmd_drift_baseline_pointer;
mod cmd_drift_baseline_refresh;
mod cmd_drift_baseline_rollback;
mod cmd_drift_baseline_supersession;
mod cmd_drift_baseline_transport_locator;
mod cmd_drift_receipt;
mod cmd_eval;
mod cmd_infer;
mod cmd_profiling_environment_snapshot;
mod cmd_profiling_environment_snapshot_refresh;
mod cmd_train;
mod cmd_upload;

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        eprintln!("{}", usage());
        std::process::exit(2);
    };

    let code = match command.as_str() {
        "train" => cmd_train::run(args),
        "infer" => cmd_infer::run(args),
        "eval" => cmd_eval::run(args),
        "cleanup" => run_cleanup(args),
        "deployment-stack-check" => cmd_deployment_stack_check::run(args),
        "deployment-verification-bundle" => cmd_deployment_verification_bundle::run(args),
        "distributed-load-profile" => cmd_distributed_load_profile::run(args),
        "profile" => run_profile(args),
        "drift" => run_drift(args),
        "upload" => cmd_upload::run(args),
        "--help" | "-h" | "help" => {
            println!("{}", usage());
            0
        }
        _ => {
            eprintln!("unknown subcommand: {command}");
            eprintln!("{}", usage());
            2
        }
    };

    if code != 0 {
        std::process::exit(code);
    }
}

fn run_cleanup<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(subcommand) = args.next() else {
        eprintln!("missing cleanup subcommand");
        eprintln!("{}", usage());
        return 2;
    };
    match subcommand.as_str() {
        "inventory" => cmd_cleanup_inventory::run(args),
        "policy" => cmd_cleanup_policy::run(args),
        _ => {
            eprintln!("unknown cleanup subcommand: {subcommand}");
            eprintln!("{}", usage());
            2
        }
    }
}

fn run_profile<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(subcommand) = args.next() else {
        eprintln!("missing profile subcommand");
        eprintln!("{}", usage());
        return 2;
    };
    match subcommand.as_str() {
        "environment-snapshot" => cmd_profiling_environment_snapshot::run(args),
        "refresh-environment-snapshot" => cmd_profiling_environment_snapshot_refresh::run(args),
        _ => {
            eprintln!("unknown profile subcommand: {subcommand}");
            eprintln!("{}", usage());
            2
        }
    }
}

fn run_drift<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(subcommand) = args.next() else {
        eprintln!("missing drift subcommand");
        eprintln!("{}", usage());
        return 2;
    };
    match subcommand.as_str() {
        "baseline" => cmd_drift_baseline::run(args),
        "approve-baseline" => cmd_drift_baseline_approval::run(args),
        "export-baseline-bundle" => cmd_drift_baseline_bundle::run(args),
        "export-baseline-handoff" => cmd_drift_baseline_handoff::run(args),
        "checkpoint-baseline" => cmd_drift_baseline_checkpoint::run(args),
        "record-approved-baseline-history" => cmd_drift_baseline_history::run(args),
        "point-baseline-transport-locator" => cmd_drift_baseline_transport_locator::run(args),
        "point-approved-baseline" => cmd_drift_baseline_pointer::run(args),
        "rollback-approved-baseline" => cmd_drift_baseline_rollback::run(args),
        "refresh-baseline" => cmd_drift_baseline_refresh::run(args),
        "supersede-baseline-approval" => cmd_drift_baseline_supersession::run(args),
        "receipt" => cmd_drift_receipt::run(args),
        _ => {
            eprintln!("unknown drift subcommand: {subcommand}");
            eprintln!("{}", usage());
            2
        }
    }
}

fn usage() -> &'static str {
    "usage: afterburner <train|infer|eval|drift <...>|cleanup <...>|profile <...>|upload|deployment-stack-check|deployment-verification-bundle|distributed-load-profile> [args]"
}
