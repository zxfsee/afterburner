mod cmd_cleanup_dry_run;
mod cmd_cleanup_evidence_bundle;
mod cmd_cleanup_execute;
mod cmd_cleanup_inventory;
mod cmd_cleanup_policy;
mod cmd_deployment_stack_check;
mod cmd_deployment_stack_launch_bundle;
mod cmd_deployment_stack_launch_bundle_reconcile;
mod cmd_deployment_stack_launch_handoff;
mod cmd_deployment_stack_launch_handoff_history;
mod cmd_deployment_stack_launch_handoff_reconcile;
mod cmd_deployment_stack_launch_handoff_reconciliation_history;
mod cmd_deployment_stack_launch_locator_history;
mod cmd_deployment_stack_launch_locator_pointer;
mod cmd_deployment_stack_launch_locator_reconcile;
mod cmd_deployment_stack_launch_locator_reconciliation_history;
mod cmd_deployment_stack_launch_plan;
mod cmd_deployment_stack_launch_receipt;
mod cmd_deployment_stack_launch_transport_locator;
mod cmd_deployment_stack_launch_transport_locator_history;
mod cmd_deployment_stack_launch_transport_locator_reconcile;
mod cmd_deployment_stack_launch_transport_locator_reconciliation_history;
mod cmd_deployment_verification_bundle;
mod cmd_deployment_verification_handoff;
mod cmd_deployment_verification_receipt;
mod cmd_distributed_load_profile;
mod cmd_distributed_runtime_benchmark;
mod cmd_distributed_runtime_profile;
mod cmd_distributed_shard_lineage_evidence_bundle;
mod cmd_distributed_shard_lineage_evidence_bundle_reconcile;
mod cmd_distributed_shard_lineage_handoff;
mod cmd_distributed_shard_lineage_handoff_history;
mod cmd_distributed_shard_lineage_handoff_reconcile;
mod cmd_distributed_shard_lineage_handoff_reconciliation_history;
mod cmd_distributed_shard_lineage_locator_history;
mod cmd_distributed_shard_lineage_locator_pointer;
mod cmd_distributed_shard_lineage_receipt;
mod cmd_distributed_shard_lineage_transport_locator;
mod cmd_distributed_shard_lineage_transport_locator_reconcile;
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
mod cmd_kube_rs_lease_history;
mod cmd_kube_rs_lease_pointer;
mod cmd_kube_rs_lease_reconcile;
mod cmd_kube_rs_lease_reconciliation_history;
mod cmd_pretraining_source_approval_receipt;
mod cmd_pretraining_source_provenance_evidence_bundle;
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
        "reconcile-evidence-bundle" => {
            cmd_distributed_shard_lineage_evidence_bundle_reconcile::run(args)
        }
        "handoff" => cmd_distributed_shard_lineage_handoff::run(args),
        "record-handoff-history" => cmd_distributed_shard_lineage_handoff_history::run(args),
        "reconcile-handoff" => cmd_distributed_shard_lineage_handoff_reconcile::run(args),
        "record-handoff-reconciliation-history" => {
            cmd_distributed_shard_lineage_handoff_reconciliation_history::run(args)
        }
        "record-locator-history" => cmd_distributed_shard_lineage_locator_history::run(args),
        "point-locator" => cmd_distributed_shard_lineage_locator_pointer::run(args),
        "reconcile-transport-locator" => {
            cmd_distributed_shard_lineage_transport_locator_reconcile::run(args)
        }
        "point-transport-locator" => cmd_distributed_shard_lineage_transport_locator::run(args),
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
        "provenance-evidence-bundle" => {
            cmd_pretraining_source_provenance_evidence_bundle::run(args)
        }
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
        "record-kube-rs-lease-history" => cmd_kube_rs_lease_history::run(args),
        "record-kube-rs-lease-reconciliation-history" => {
            cmd_kube_rs_lease_reconciliation_history::run(args)
        }
        "point-kube-rs-lease" => cmd_kube_rs_lease_pointer::run(args),
        "kube-rs-lease-reconcile" => cmd_kube_rs_lease_reconcile::run(args),
        "upload" => cmd_upload::run(args),
        "stack-check" => cmd_deployment_stack_check::run(args),
        "stack-launch-bundle" => cmd_deployment_stack_launch_bundle::run(args),
        "reconcile-launch-bundle" => cmd_deployment_stack_launch_bundle_reconcile::run(args),
        "stack-launch-handoff" => cmd_deployment_stack_launch_handoff::run(args),
        "record-launch-handoff-history" => cmd_deployment_stack_launch_handoff_history::run(args),
        "reconcile-launch-handoff" => cmd_deployment_stack_launch_handoff_reconcile::run(args),
        "record-launch-handoff-reconciliation-history" => {
            cmd_deployment_stack_launch_handoff_reconciliation_history::run(args)
        }
        "record-launch-locator-history" => cmd_deployment_stack_launch_locator_history::run(args),
        "record-launch-locator-reconciliation-history" => {
            cmd_deployment_stack_launch_locator_reconciliation_history::run(args)
        }
        "record-launch-transport-locator-history" => {
            cmd_deployment_stack_launch_transport_locator_history::run(args)
        }
        "record-launch-transport-locator-reconciliation-history" => {
            cmd_deployment_stack_launch_transport_locator_reconciliation_history::run(args)
        }
        "stack-launch-plan" => cmd_deployment_stack_launch_plan::run(args),
        "stack-launch-receipt" => cmd_deployment_stack_launch_receipt::run(args),
        "point-launch-locator" => cmd_deployment_stack_launch_locator_pointer::run(args),
        "reconcile-launch-locator" => cmd_deployment_stack_launch_locator_reconcile::run(args),
        "reconcile-launch-transport-locator" => {
            cmd_deployment_stack_launch_transport_locator_reconcile::run(args)
        }
        "point-launch-transport-locator" => {
            cmd_deployment_stack_launch_transport_locator::run(args)
        }
        "verification-receipt" => cmd_deployment_verification_receipt::run(args),
        "verification-bundle" => cmd_deployment_verification_bundle::run(args),
        "verification-handoff" => cmd_deployment_verification_handoff::run(args),
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
