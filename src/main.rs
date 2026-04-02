#![recursion_limit = "256"]

mod cmd_burn_bpk_migration_surface_inventory;
mod cmd_cleanup_dry_run;
mod cmd_cleanup_evidence_bundle;
mod cmd_cleanup_execute;
mod cmd_cleanup_inventory;
mod cmd_cleanup_policy;
mod cmd_deployment_stack_check;
mod cmd_deployment_stack_launch_bundle;
mod cmd_deployment_stack_launch_bundle_reconcile;
mod cmd_deployment_stack_launch_bundle_reconciliation_history;
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
mod cmd_deployment_verification_bundle_history;
mod cmd_deployment_verification_bundle_locator_history;
mod cmd_deployment_verification_bundle_locator_pointer;
mod cmd_deployment_verification_bundle_locator_reconcile;
mod cmd_deployment_verification_bundle_locator_reconciliation_history;
mod cmd_deployment_verification_bundle_locator_rollback;
mod cmd_deployment_verification_bundle_locator_rollback_history;
mod cmd_deployment_verification_bundle_reconcile;
mod cmd_deployment_verification_bundle_reconciliation_history;
mod cmd_deployment_verification_bundle_rollback;
mod cmd_deployment_verification_bundle_rollback_history;
mod cmd_deployment_verification_bundle_rollback_reconcile;
mod cmd_deployment_verification_bundle_rollback_reconciliation_history;
mod cmd_deployment_verification_bundle_rollback_supersession;
mod cmd_deployment_verification_bundle_rollback_supersession_history;
mod cmd_deployment_verification_bundle_rollback_supersession_reconcile;
mod cmd_deployment_verification_bundle_rollback_supersession_reconciliation_history;
mod cmd_deployment_verification_bundle_transport_locator;
mod cmd_deployment_verification_bundle_transport_locator_history;
mod cmd_deployment_verification_bundle_transport_locator_reconcile;
mod cmd_deployment_verification_bundle_transport_locator_reconciliation_history;
mod cmd_deployment_verification_handoff;
mod cmd_deployment_verification_handoff_history;
mod cmd_deployment_verification_handoff_reconcile;
mod cmd_deployment_verification_handoff_reconciliation_history;
mod cmd_deployment_verification_handoff_transport_locator;
mod cmd_deployment_verification_handoff_transport_locator_history;
mod cmd_deployment_verification_handoff_transport_locator_reconcile;
mod cmd_deployment_verification_receipt;
mod cmd_deployment_verification_receipt_history;
mod cmd_deployment_verification_receipt_locator_history;
mod cmd_deployment_verification_receipt_locator_pointer;
mod cmd_deployment_verification_receipt_locator_reconcile;
mod cmd_deployment_verification_receipt_locator_reconciliation_history;
mod cmd_deployment_verification_receipt_locator_rollback;
mod cmd_deployment_verification_receipt_locator_rollback_history;
mod cmd_deployment_verification_receipt_reconcile;
mod cmd_deployment_verification_receipt_reconciliation_history;
mod cmd_deployment_verification_receipt_rollback_reconcile;
mod cmd_deployment_verification_receipt_rollback_reconciliation_history;
mod cmd_deployment_verification_receipt_rollback_supersession;
mod cmd_deployment_verification_receipt_rollback_supersession_history;
mod cmd_deployment_verification_receipt_rollback_supersession_reconcile;
mod cmd_deployment_verification_receipt_rollback_supersession_reconciliation_history;
mod cmd_deployment_verification_receipt_transport_locator;
mod cmd_deployment_verification_receipt_transport_locator_history;
mod cmd_deployment_verification_receipt_transport_locator_reconcile;
mod cmd_deployment_verification_receipt_transport_locator_reconciliation_history;
mod cmd_distributed_load_profile;
mod cmd_distributed_runtime_benchmark;
mod cmd_distributed_runtime_profile;
mod cmd_distributed_shard_lineage_evidence_bundle;
mod cmd_distributed_shard_lineage_evidence_bundle_reconcile;
mod cmd_distributed_shard_lineage_evidence_bundle_reconciliation_history;
mod cmd_distributed_shard_lineage_handoff;
mod cmd_distributed_shard_lineage_handoff_history;
mod cmd_distributed_shard_lineage_handoff_reconcile;
mod cmd_distributed_shard_lineage_handoff_reconciliation_history;
mod cmd_distributed_shard_lineage_locator_history;
mod cmd_distributed_shard_lineage_locator_pointer;
mod cmd_distributed_shard_lineage_receipt;
mod cmd_distributed_shard_lineage_transport_locator;
mod cmd_distributed_shard_lineage_transport_locator_reconcile;
mod cmd_distributed_shard_lineage_transport_locator_reconciliation_history;
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
mod cmd_optimized_model_local_profile;
mod cmd_pretraining_source_approval_receipt;
mod cmd_pretraining_source_provenance_evidence_bundle;
mod cmd_pretraining_source_provenance_receipt;
mod cmd_profiling_environment_snapshot;
mod cmd_profiling_environment_snapshot_refresh;
mod cmd_profiling_provenance_bundle;
mod cmd_profiling_provenance_receipt;
mod cmd_scheduler_heartbeat;
mod cmd_scheduler_heartbeat_history;
mod cmd_scheduler_heartbeat_pointer;
mod cmd_scheduler_heartbeat_pointer_history;
mod cmd_scheduler_heartbeat_pointer_reconcile;
mod cmd_scheduler_heartbeat_pointer_rollback;
mod cmd_scheduler_heartbeat_pointer_rollback_history;
mod cmd_scheduler_heartbeat_pointer_rollback_reconcile;
mod cmd_scheduler_heartbeat_pointer_rollback_reconciliation_history;
mod cmd_scheduler_heartbeat_pointer_rollback_supersession;
mod cmd_scheduler_heartbeat_pointer_rollback_supersession_history;
mod cmd_scheduler_heartbeat_pointer_rollback_supersession_reconcile;
mod cmd_scheduler_heartbeat_pointer_rollback_supersession_reconciliation_history;
mod cmd_scheduler_heartbeat_pointer_supersession;
mod cmd_scheduler_heartbeat_pointer_supersession_history;
mod cmd_scheduler_heartbeat_pointer_supersession_reconcile;
mod cmd_scheduler_heartbeat_pointer_supersession_reconciliation_history;
mod cmd_scheduler_heartbeat_reconcile;
mod cmd_scheduler_heartbeat_reconciliation_history;
mod cmd_scheduler_heartbeat_supersession;
mod cmd_scheduler_heartbeat_supersession_history;
mod cmd_scheduler_heartbeat_supersession_reconcile;
mod cmd_scheduler_heartbeat_supersession_reconciliation_history;
mod cmd_scheduler_runtime_simulation;
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
        "debug" => run_debug(args),
        "deploy" => run_deploy(args),
        "verify" => run_verify(args),
        "rollback" => run_rollback(args),
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

fn run_debug<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(group) = args.next() else {
        eprintln!("missing debug subcommand");
        eprintln!("{}", usage());
        return 2;
    };

    match group.as_str() {
        "deploy" => run_debug_deploy(args),
        "inventory-burn-bpk-surface" => cmd_burn_bpk_migration_surface_inventory::run(args),
        "simulate-scheduler-runtime" => cmd_scheduler_runtime_simulation::run(args),
        _ => {
            eprintln!("unknown debug subcommand: {group}");
            eprintln!("{}", usage());
            2
        }
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
        "optimized-model-local-profile" => cmd_optimized_model_local_profile::run(args),
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
        "record-evidence-bundle-reconciliation-history" => {
            cmd_distributed_shard_lineage_evidence_bundle_reconciliation_history::run(args)
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
        "record-transport-locator-reconciliation-history" => {
            cmd_distributed_shard_lineage_transport_locator_reconciliation_history::run(args)
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
    if let Some(target) = deploy_redirect_target(&subcommand) {
        eprintln!(
            "deploy subcommand `{subcommand}` moved to `{target}`; use `afterburner {target} ...`"
        );
        return 2;
    }
    if is_debug_only_deploy_subcommand(&subcommand) {
        eprintln!(
            "deploy subcommand `{subcommand}` is debug-only; use `afterburner debug deploy {subcommand} ...`"
        );
        return 2;
    }
    dispatch_deploy(subcommand, args)
}

fn run_debug_deploy<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(subcommand) = args.next() else {
        eprintln!("missing debug deploy subcommand");
        eprintln!("{}", usage());
        return 2;
    };
    if !is_debug_only_deploy_subcommand(&subcommand) {
        eprintln!("unknown debug deploy subcommand: {subcommand}");
        eprintln!("{}", usage());
        return 2;
    }
    dispatch_deploy(subcommand, args)
}

fn is_debug_only_deploy_subcommand(subcommand: &str) -> bool {
    matches!(
        subcommand,
        "record-scheduler-heartbeat-history"
            | "record-scheduler-heartbeat-pointer-history"
            | "record-scheduler-heartbeat-pointer-rollback-history"
            | "record-scheduler-heartbeat-pointer-rollback-reconciliation-history"
            | "reconcile-scheduler-heartbeat-pointer-rollback"
            | "scheduler-heartbeat-point-rollback-supersede"
            | "record-scheduler-heartbeat-pointer-rollback-supersession-history"
            | "reconcile-scheduler-heartbeat-pointer-rollback-supersession"
            | "record-scheduler-heartbeat-pointer-rollback-supersession-reconciliation-history"
            | "record-scheduler-heartbeat-pointer-supersession-history"
            | "record-scheduler-heartbeat-pointer-supersession-reconciliation-history"
            | "rollback-scheduler-heartbeat-pointer"
            | "reconcile-scheduler-heartbeat-pointer-supersession"
            | "scheduler-heartbeat-point-supersede"
            | "reconcile-scheduler-heartbeat-pointer"
            | "scheduler-heartbeat-reconcile"
            | "record-scheduler-heartbeat-reconciliation-history"
            | "point-scheduler-heartbeat"
            | "scheduler-heartbeat-supersede"
            | "scheduler-heartbeat-supersession-reconcile"
            | "record-scheduler-heartbeat-supersession-history"
            | "record-scheduler-heartbeat-supersession-reconciliation-history"
    ) || subcommand.starts_with("record-verification-")
        || subcommand.starts_with("point-verification-")
        || subcommand.starts_with("reconcile-verification-")
        || subcommand.starts_with("supersede-verification-")
}

fn deploy_redirect_target(subcommand: &str) -> Option<&'static str> {
    match subcommand {
        "verification-receipt" => Some("verify receipt"),
        "verification-bundle" => Some("verify bundle"),
        "verification-handoff" => Some("verify handoff"),
        "rollback-verification-receipt-locator" => Some("rollback verification-receipt-locator"),
        "rollback-verification-bundle-locator" => Some("rollback verification-bundle-locator"),
        "rollback-verification-bundle" => Some("rollback verification-bundle"),
        _ => None,
    }
}

fn run_verify<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(subcommand) = args.next() else {
        eprintln!("missing verify subcommand");
        eprintln!("{}", usage());
        return 2;
    };

    match subcommand.as_str() {
        "receipt" => cmd_deployment_verification_receipt::run(args),
        "bundle" => cmd_deployment_verification_bundle::run(args),
        "handoff" => cmd_deployment_verification_handoff::run(args),
        _ => {
            eprintln!("unknown verify subcommand: {subcommand}");
            eprintln!("{}", usage());
            2
        }
    }
}

fn run_rollback<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(subcommand) = args.next() else {
        eprintln!("missing rollback subcommand");
        eprintln!("{}", usage());
        return 2;
    };

    match subcommand.as_str() {
        "verification-receipt-locator" => {
            cmd_deployment_verification_receipt_locator_rollback::run(args)
        }
        "verification-bundle-locator" => {
            cmd_deployment_verification_bundle_locator_rollback::run(args)
        }
        "verification-bundle" => cmd_deployment_verification_bundle_rollback::run(args),
        _ => {
            eprintln!("unknown rollback subcommand: {subcommand}");
            eprintln!("{}", usage());
            2
        }
    }
}

fn dispatch_deploy<I>(subcommand: String, args: I) -> i32
where
    I: Iterator<Item = String>,
{
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
        "record-launch-bundle-reconciliation-history" => {
            cmd_deployment_stack_launch_bundle_reconciliation_history::run(args)
        }
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
        "record-verification-receipt-locator-history" => {
            cmd_deployment_verification_receipt_locator_history::run(args)
        }
        "point-verification-receipt-locator" => {
            cmd_deployment_verification_receipt_locator_pointer::run(args)
        }
        "reconcile-verification-receipt-locator" => {
            cmd_deployment_verification_receipt_locator_reconcile::run(args)
        }
        "record-verification-receipt-locator-reconciliation-history" => {
            cmd_deployment_verification_receipt_locator_reconciliation_history::run(args)
        }
        "record-verification-receipt-locator-rollback-history" => {
            cmd_deployment_verification_receipt_locator_rollback_history::run(args)
        }
        "reconcile-verification-receipt-rollback" => {
            cmd_deployment_verification_receipt_rollback_reconcile::run(args)
        }
        "record-verification-receipt-rollback-reconciliation-history" => {
            cmd_deployment_verification_receipt_rollback_reconciliation_history::run(args)
        }
        "supersede-verification-receipt-rollback" => {
            cmd_deployment_verification_receipt_rollback_supersession::run(args)
        }
        "reconcile-verification-receipt-rollback-supersession" => {
            cmd_deployment_verification_receipt_rollback_supersession_reconcile::run(args)
        }
        "record-verification-receipt-rollback-supersession-reconciliation-history" => {
            cmd_deployment_verification_receipt_rollback_supersession_reconciliation_history::run(
                args,
            )
        }
        "record-verification-receipt-rollback-supersession-history" => {
            cmd_deployment_verification_receipt_rollback_supersession_history::run(args)
        }
        "point-verification-receipt-transport-locator" => {
            cmd_deployment_verification_receipt_transport_locator::run(args)
        }
        "record-verification-receipt-transport-locator-history" => {
            cmd_deployment_verification_receipt_transport_locator_history::run(args)
        }
        "reconcile-verification-receipt-transport-locator" => {
            cmd_deployment_verification_receipt_transport_locator_reconcile::run(args)
        }
        "record-verification-receipt-transport-locator-reconciliation-history" => {
            cmd_deployment_verification_receipt_transport_locator_reconciliation_history::run(args)
        }
        "record-verification-receipt-history" => {
            cmd_deployment_verification_receipt_history::run(args)
        }
        "reconcile-verification-receipt" => {
            cmd_deployment_verification_receipt_reconcile::run(args)
        }
        "record-verification-receipt-reconciliation-history" => {
            cmd_deployment_verification_receipt_reconciliation_history::run(args)
        }
        "point-verification-bundle-locator" => {
            cmd_deployment_verification_bundle_locator_pointer::run(args)
        }
        "record-verification-bundle-locator-history" => {
            cmd_deployment_verification_bundle_locator_history::run(args)
        }
        "reconcile-verification-bundle-locator" => {
            cmd_deployment_verification_bundle_locator_reconcile::run(args)
        }
        "record-verification-bundle-locator-reconciliation-history" => {
            cmd_deployment_verification_bundle_locator_reconciliation_history::run(args)
        }
        "record-verification-bundle-locator-rollback-history" => {
            cmd_deployment_verification_bundle_locator_rollback_history::run(args)
        }
        "record-verification-bundle-rollback-history" => {
            cmd_deployment_verification_bundle_rollback_history::run(args)
        }
        "reconcile-verification-bundle-rollback" => {
            cmd_deployment_verification_bundle_rollback_reconcile::run(args)
        }
        "record-verification-bundle-rollback-reconciliation-history" => {
            cmd_deployment_verification_bundle_rollback_reconciliation_history::run(args)
        }
        "supersede-verification-bundle-rollback" => {
            cmd_deployment_verification_bundle_rollback_supersession::run(args)
        }
        "reconcile-verification-bundle-rollback-supersession" => {
            cmd_deployment_verification_bundle_rollback_supersession_reconcile::run(args)
        }
        "record-verification-bundle-rollback-supersession-reconciliation-history" => {
            cmd_deployment_verification_bundle_rollback_supersession_reconciliation_history::run(
                args,
            )
        }
        "record-verification-bundle-rollback-supersession-history" => {
            cmd_deployment_verification_bundle_rollback_supersession_history::run(args)
        }
        "record-verification-bundle-history" => {
            cmd_deployment_verification_bundle_history::run(args)
        }
        "point-verification-bundle-transport-locator" => {
            cmd_deployment_verification_bundle_transport_locator::run(args)
        }
        "record-verification-bundle-transport-locator-history" => {
            cmd_deployment_verification_bundle_transport_locator_history::run(args)
        }
        "record-verification-bundle-transport-locator-reconciliation-history" => {
            cmd_deployment_verification_bundle_transport_locator_reconciliation_history::run(args)
        }
        "reconcile-verification-bundle-transport-locator" => {
            cmd_deployment_verification_bundle_transport_locator_reconcile::run(args)
        }
        "reconcile-verification-bundle" => cmd_deployment_verification_bundle_reconcile::run(args),
        "record-verification-bundle-reconciliation-history" => {
            cmd_deployment_verification_bundle_reconciliation_history::run(args)
        }
        "record-verification-handoff-history" => {
            cmd_deployment_verification_handoff_history::run(args)
        }
        "point-verification-handoff-transport-locator" => {
            cmd_deployment_verification_handoff_transport_locator::run(args)
        }
        "record-verification-handoff-transport-locator-history" => {
            cmd_deployment_verification_handoff_transport_locator_history::run(args)
        }
        "reconcile-verification-handoff-transport-locator" => {
            cmd_deployment_verification_handoff_transport_locator_reconcile::run(args)
        }
        "reconcile-verification-handoff" => {
            cmd_deployment_verification_handoff_reconcile::run(args)
        }
        "record-verification-handoff-reconciliation-history" => {
            cmd_deployment_verification_handoff_reconciliation_history::run(args)
        }
        "scheduler-heartbeat" => cmd_scheduler_heartbeat::run(args),
        "record-scheduler-heartbeat-history" => cmd_scheduler_heartbeat_history::run(args),
        "point-scheduler-heartbeat" => cmd_scheduler_heartbeat_pointer::run(args),
        "record-scheduler-heartbeat-pointer-history" => {
            cmd_scheduler_heartbeat_pointer_history::run(args)
        }
        "record-scheduler-heartbeat-pointer-rollback-history" => {
            cmd_scheduler_heartbeat_pointer_rollback_history::run(args)
        }
        "record-scheduler-heartbeat-pointer-rollback-reconciliation-history" => {
            cmd_scheduler_heartbeat_pointer_rollback_reconciliation_history::run(args)
        }
        "reconcile-scheduler-heartbeat-pointer-rollback" => {
            cmd_scheduler_heartbeat_pointer_rollback_reconcile::run(args)
        }
        "scheduler-heartbeat-point-rollback-supersede" => {
            cmd_scheduler_heartbeat_pointer_rollback_supersession::run(args)
        }
        "record-scheduler-heartbeat-pointer-rollback-supersession-history" => {
            cmd_scheduler_heartbeat_pointer_rollback_supersession_history::run(args)
        }
        "reconcile-scheduler-heartbeat-pointer-rollback-supersession" => {
            cmd_scheduler_heartbeat_pointer_rollback_supersession_reconcile::run(args)
        }
        "record-scheduler-heartbeat-pointer-rollback-supersession-reconciliation-history" => {
            cmd_scheduler_heartbeat_pointer_rollback_supersession_reconciliation_history::run(args)
        }
        "rollback-scheduler-heartbeat-pointer" => {
            cmd_scheduler_heartbeat_pointer_rollback::run(args)
        }
        "record-scheduler-heartbeat-pointer-supersession-history" => {
            cmd_scheduler_heartbeat_pointer_supersession_history::run(args)
        }
        "record-scheduler-heartbeat-pointer-supersession-reconciliation-history" => {
            cmd_scheduler_heartbeat_pointer_supersession_reconciliation_history::run(args)
        }
        "reconcile-scheduler-heartbeat-pointer-supersession" => {
            cmd_scheduler_heartbeat_pointer_supersession_reconcile::run(args)
        }
        "scheduler-heartbeat-point-supersede" => {
            cmd_scheduler_heartbeat_pointer_supersession::run(args)
        }
        "reconcile-scheduler-heartbeat-pointer" => {
            cmd_scheduler_heartbeat_pointer_reconcile::run(args)
        }
        "scheduler-heartbeat-reconcile" => cmd_scheduler_heartbeat_reconcile::run(args),
        "record-scheduler-heartbeat-reconciliation-history" => {
            cmd_scheduler_heartbeat_reconciliation_history::run(args)
        }
        "scheduler-heartbeat-supersede" => cmd_scheduler_heartbeat_supersession::run(args),
        "scheduler-heartbeat-supersession-reconcile" => {
            cmd_scheduler_heartbeat_supersession_reconcile::run(args)
        }
        "record-scheduler-heartbeat-supersession-reconciliation-history" => {
            cmd_scheduler_heartbeat_supersession_reconciliation_history::run(args)
        }
        "record-scheduler-heartbeat-supersession-history" => {
            cmd_scheduler_heartbeat_supersession_history::run(args)
        }
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
    "usage: afterburner <train|infer|eval|deploy <...>|verify <...>|rollback <...>|drift <...>|cleanup <...>|profile <...>|lineage <...>|source <...>|debug <deploy <...>|simulate-scheduler-runtime <...>>> [args]"
}
