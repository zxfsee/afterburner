mod deploy;

pub fn run<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(command) = args.next() else {
        eprintln!("{}", usage());
        return 2;
    };

    match command.as_str() {
        "train" => crate::cmd_train::run(args),
        "infer" => crate::cmd_infer::run(args),
        "eval" => crate::cmd_eval::run(args),
        "cleanup" => run_cleanup(args),
        "debug" => run_debug(args),
        "deploy" => deploy::run_deploy(args),
        "verify" => deploy::run_verify(args),
        "rollback" => deploy::run_rollback(args),
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
        "deploy" => deploy::run_debug_deploy(args),
        "inventory-burn-bpk-surface" => crate::cmd_burn_bpk_migration_surface_inventory::run(args),
        "simulate-scheduler-runtime" => crate::cmd_scheduler_runtime_simulation::run(args),
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
        "dry-run" => crate::cmd_cleanup_dry_run::run(args),
        "evidence-bundle" => crate::cmd_cleanup_evidence_bundle::run(args),
        "execute" => crate::cmd_cleanup_execute::run(args),
        "inventory" => crate::cmd_cleanup_inventory::run(args),
        "policy" => crate::cmd_cleanup_policy::run(args),
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
        "environment-snapshot" => crate::cmd_profiling_environment_snapshot::run(args),
        "infer" => crate::cmd_profiling_infer::run(args),
        "distributed-runtime-benchmark" => crate::cmd_distributed_runtime_benchmark::run(args),
        "distributed-runtime-profile" => crate::cmd_distributed_runtime_profile::run(args),
        "optimized-model-local-profile" => crate::cmd_optimized_model_local_profile::run(args),
        "provenance-bundle" => crate::cmd_profiling_provenance_bundle::run(args),
        "provenance-receipt" => crate::cmd_profiling_provenance_receipt::run(args),
        "refresh-environment-snapshot" => {
            crate::cmd_profiling_environment_snapshot_refresh::run(args)
        }
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
        "bundle" => crate::cmd_distributed_shard_lineage_evidence_bundle::run(args),
        "bundle-manage" => run_lineage_bundle_manage(args),
        "evidence-bundle" => crate::cmd_distributed_shard_lineage_evidence_bundle::run(args),
        "reconcile-evidence-bundle" => {
            crate::cmd_distributed_shard_lineage_evidence_bundle_reconcile::run(args)
        }
        "record-evidence-bundle-reconciliation-history" => {
            crate::cmd_distributed_shard_lineage_evidence_bundle_reconciliation_history::run(args)
        }
        "handoff" => crate::cmd_distributed_shard_lineage_handoff::run(args),
        "handoff-manage" => run_lineage_handoff_manage(args),
        "record-handoff-history" => crate::cmd_distributed_shard_lineage_handoff_history::run(args),
        "reconcile-handoff" => crate::cmd_distributed_shard_lineage_handoff_reconcile::run(args),
        "record-handoff-reconciliation-history" => {
            crate::cmd_distributed_shard_lineage_handoff_reconciliation_history::run(args)
        }
        "locator-manage" => run_lineage_locator_manage(args),
        "record-locator-history" => crate::cmd_distributed_shard_lineage_locator_history::run(args),
        "point-locator" => crate::cmd_distributed_shard_lineage_locator_pointer::run(args),
        "reconcile-transport-locator" => {
            crate::cmd_distributed_shard_lineage_transport_locator_reconcile::run(args)
        }
        "record-transport-locator-reconciliation-history" => {
            crate::cmd_distributed_shard_lineage_transport_locator_reconciliation_history::run(args)
        }
        "point-transport-locator" => {
            crate::cmd_distributed_shard_lineage_transport_locator::run(args)
        }
        "receipt" => crate::cmd_distributed_shard_lineage_receipt::run(args),
        _ => {
            eprintln!("unknown lineage subcommand: {subcommand}");
            eprintln!("{}", usage());
            2
        }
    }
}

fn run_lineage_bundle_manage<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(action) = args.next() else {
        eprintln!("missing lineage bundle-manage action");
        eprintln!("{}", usage());
        return 2;
    };
    match action.as_str() {
        "reconcile" => crate::cmd_distributed_shard_lineage_evidence_bundle_reconcile::run(args),
        "record-reconciliation-history" => {
            crate::cmd_distributed_shard_lineage_evidence_bundle_reconciliation_history::run(args)
        }
        _ => {
            eprintln!("unknown lineage bundle-manage action: {action}");
            eprintln!("{}", usage());
            2
        }
    }
}

fn run_lineage_handoff_manage<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(action) = args.next() else {
        eprintln!("missing lineage handoff-manage action");
        eprintln!("{}", usage());
        return 2;
    };
    match action.as_str() {
        "reconcile" => crate::cmd_distributed_shard_lineage_handoff_reconcile::run(args),
        "record-history" => crate::cmd_distributed_shard_lineage_handoff_history::run(args),
        "record-reconciliation-history" => {
            crate::cmd_distributed_shard_lineage_handoff_reconciliation_history::run(args)
        }
        _ => {
            eprintln!("unknown lineage handoff-manage action: {action}");
            eprintln!("{}", usage());
            2
        }
    }
}

fn run_lineage_locator_manage<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(action) = args.next() else {
        eprintln!("missing lineage locator-manage action");
        eprintln!("{}", usage());
        return 2;
    };
    match action.as_str() {
        "point-transport" => crate::cmd_distributed_shard_lineage_transport_locator::run(args),
        "reconcile-transport" => {
            crate::cmd_distributed_shard_lineage_transport_locator_reconcile::run(args)
        }
        "record-transport-reconciliation-history" => {
            crate::cmd_distributed_shard_lineage_transport_locator_reconciliation_history::run(args)
        }
        "point" => crate::cmd_distributed_shard_lineage_locator_pointer::run(args),
        "record-history" => crate::cmd_distributed_shard_lineage_locator_history::run(args),
        _ => {
            eprintln!("unknown lineage locator-manage action: {action}");
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
        "--help" | "-h" | "help" => {
            println!("{}", usage());
            0
        }
        "approval" => run_source_approval(args),
        "provenance" => run_source_provenance(args),
        _ => {
            eprintln!("unknown source subcommand: {subcommand}");
            eprintln!("{}", usage());
            2
        }
    }
}

fn run_source_approval<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(action) = args.next() else {
        eprintln!("missing source approval action");
        eprintln!("{}", usage());
        return 2;
    };
    match action.as_str() {
        "--help" | "-h" | "help" => {
            println!("{}", usage());
            0
        }
        "receipt" => crate::cmd_pretraining_source_approval_receipt::run(args),
        _ => {
            eprintln!("unknown source approval action: {action}");
            eprintln!("{}", usage());
            2
        }
    }
}

fn run_source_provenance<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(action) = args.next() else {
        eprintln!("missing source provenance action");
        eprintln!("{}", usage());
        return 2;
    };
    match action.as_str() {
        "--help" | "-h" | "help" => {
            println!("{}", usage());
            0
        }
        "bundle" => crate::cmd_pretraining_source_provenance_evidence_bundle::run(args),
        "receipt" => crate::cmd_pretraining_source_provenance_receipt::run(args),
        _ => {
            eprintln!("unknown source provenance action: {action}");
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
        "baseline" => crate::cmd_drift_baseline::run(args),
        "approve-baseline" => crate::cmd_drift_baseline_approval::run(args),
        "export-baseline-bundle" => crate::cmd_drift_baseline_bundle::run(args),
        "export-baseline-handoff" => crate::cmd_drift_baseline_handoff::run(args),
        "checkpoint-baseline" => crate::cmd_drift_baseline_checkpoint::run(args),
        "record-approved-baseline-history" => crate::cmd_drift_baseline_history::run(args),
        "point-baseline-transport-locator" => {
            crate::cmd_drift_baseline_transport_locator::run(args)
        }
        "point-approved-baseline" => crate::cmd_drift_baseline_pointer::run(args),
        "rollback-approved-baseline" => crate::cmd_drift_baseline_rollback::run(args),
        "refresh-baseline" => crate::cmd_drift_baseline_refresh::run(args),
        "supersede-baseline-approval" => crate::cmd_drift_baseline_supersession::run(args),
        "receipt" => crate::cmd_drift_receipt::run(args),
        _ => {
            eprintln!("unknown drift subcommand: {subcommand}");
            eprintln!("{}", usage());
            2
        }
    }
}

pub fn usage() -> &'static str {
    "usage: afterburner <train|infer|eval|deploy <...>|verify <...>|rollback <...>|drift <...>|cleanup <...>|profile <...>|lineage <...>|source <...>|debug <deploy <...>|simulate-scheduler-runtime <...>>> [args]"
}
