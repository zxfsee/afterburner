mod cmd_cleanup_dry_run;
mod cmd_cleanup_evidence_bundle;
mod cmd_cleanup_execute;
mod cmd_cleanup_inventory;
mod cmd_cleanup_policy;
mod cmd_deployment_stack_check;
mod cmd_deployment_verification_bundle;
mod cmd_deployment_verification_receipt;
mod cmd_distributed_load_profile;
mod cmd_distributed_runtime_benchmark;
mod cmd_distributed_runtime_profile;
mod cmd_distributed_shard_lineage_evidence_bundle;
mod cmd_distributed_shard_lineage_receipt;
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
mod cmd_hf_publish;
mod cmd_infer;
mod cmd_pretraining_source_approval_receipt;
mod cmd_pretraining_source_provenance_receipt;
mod cmd_profiling_environment_snapshot;
mod cmd_profiling_environment_snapshot_refresh;
mod cmd_profiling_provenance_bundle;
mod cmd_profiling_provenance_receipt;
mod cmd_single_node_scheduler;
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
        "deploy" => run_deploy(args),
        "lineage" => run_lineage(args),
        "profile" => run_profile(args),
        "source" => run_source(args),
        "drift" => run_drift(args),
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
        "dry-run" => cmd_cleanup_dry_run::run(args),
        "evidence-bundle" => cmd_cleanup_evidence_bundle::run(args),
        "execute" => cmd_cleanup_execute::run(args),
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
        "distributed-runtime-benchmark" => cmd_distributed_runtime_benchmark::run(args),
        "distributed-runtime-profile" => cmd_distributed_runtime_profile::run(args),
        "provenance-bundle" => cmd_profiling_provenance_bundle::run(args),
        "provenance-receipt" => cmd_profiling_provenance_receipt::run(args),
        "refresh-environment-snapshot" => cmd_profiling_environment_snapshot_refresh::run(args),
        _ => {
            eprintln!("unknown profile subcommand: {subcommand}");
            eprintln!("{}", usage());
            2
        }
    }
}

fn run_lineage<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(subcommand) = args.next() else {
        eprintln!("missing lineage subcommand");
        eprintln!("{}", usage());
        return 2;
    };
    match subcommand.as_str() {
        "evidence-bundle" => cmd_distributed_shard_lineage_evidence_bundle::run(args),
        "receipt" => cmd_distributed_shard_lineage_receipt::run(args),
        _ => {
            eprintln!("unknown lineage subcommand: {subcommand}");
            eprintln!("{}", usage());
            2
        }
    }
}

fn run_source<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(subcommand) = args.next() else {
        eprintln!("missing source subcommand");
        eprintln!("{}", usage());
        return 2;
    };
    match subcommand.as_str() {
        "approval-receipt" => cmd_pretraining_source_approval_receipt::run(args),
        "provenance-receipt" => cmd_pretraining_source_provenance_receipt::run(args),
        _ => {
            eprintln!("unknown source subcommand: {subcommand}");
            eprintln!("{}", usage());
            2
        }
    }
}

fn run_deploy<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(subcommand) = args.next() else {
        eprintln!("missing deploy subcommand");
        eprintln!("{}", usage());
        return 2;
    };
    match subcommand.as_str() {
        "hf-publish" => cmd_hf_publish::run(args),
        "upload" => cmd_upload::run(args),
        "stack-check" => cmd_deployment_stack_check::run(args),
        "verification-receipt" => cmd_deployment_verification_receipt::run(args),
        "verification-bundle" => cmd_deployment_verification_bundle::run(args),
        "load-profile" => cmd_distributed_load_profile::run(args),
        "single-node-scheduler" => cmd_single_node_scheduler::run(args),
        _ => {
            eprintln!("unknown deploy subcommand: {subcommand}");
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
    "usage: afterburner <train|infer|eval|deploy <...>|drift <...>|cleanup <...>|profile <...>|lineage <...>|source <...>> [args]"
}
