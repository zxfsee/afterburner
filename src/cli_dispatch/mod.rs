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
        "lineage" => {
            eprintln!(
                "lineage operator flows moved to the grouped `just distributed-shard-lineage-*` recipes; low-level lineage writers remain behind `afterburner debug lineage ...`"
            );
            eprintln!("{}", usage());
            2
        }
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
        "lineage" => run_lineage(args),
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
        "bundle" => run_lineage_bundle(args),
        "handoff" => run_lineage_handoff(args),
        "locator" => run_lineage_locator(args),
        "receipt" => crate::cmd_distributed_shard_lineage_receipt::run(args),
        _ => {
            eprintln!("unknown lineage subcommand: {subcommand}");
            eprintln!("{}", usage());
            2
        }
    }
}

fn run_lineage_bundle<I>(args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let args = args.collect::<Vec<_>>();
    let Some(action) = args.first().cloned() else {
        return crate::cmd_distributed_shard_lineage_evidence_bundle::run(args.into_iter());
    };
    if action.starts_with('-') {
        return crate::cmd_distributed_shard_lineage_evidence_bundle::run(args.into_iter());
    }
    let rest = args.into_iter().skip(1);
    match action.as_str() {
        "reconcile" => crate::cmd_distributed_shard_lineage_evidence_bundle_reconcile::run(rest),
        "history" => {
            crate::cmd_distributed_shard_lineage_evidence_bundle_reconciliation_history::run(rest)
        }
        _ => {
            eprintln!("unknown lineage bundle action: {action}");
            eprintln!("{}", usage());
            2
        }
    }
}

fn run_lineage_handoff<I>(args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let args = args.collect::<Vec<_>>();
    let Some(action) = args.first().cloned() else {
        return crate::cmd_distributed_shard_lineage_handoff::run(args.into_iter());
    };
    if action.starts_with('-') {
        return crate::cmd_distributed_shard_lineage_handoff::run(args.into_iter());
    }
    let rest = args.into_iter().skip(1);
    match action.as_str() {
        "reconcile" => crate::cmd_distributed_shard_lineage_handoff_reconcile::run(rest),
        "history" => run_lineage_handoff_history(rest),
        _ => {
            eprintln!("unknown lineage handoff action: {action}");
            eprintln!("{}", usage());
            2
        }
    }
}

fn run_lineage_handoff_history<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(action) = args.next() else {
        eprintln!("missing lineage handoff history action");
        eprintln!("{}", usage());
        return 2;
    };
    if action == "--help" || action == "-h" {
        println!("{}", usage());
        return 0;
    }
    match action.as_str() {
        "record" => crate::cmd_distributed_shard_lineage_handoff_history::run(args),
        "reconciliation" => {
            crate::cmd_distributed_shard_lineage_handoff_reconciliation_history::run(args)
        }
        _ => {
            eprintln!("unknown lineage handoff history action: {action}");
            eprintln!("{}", usage());
            2
        }
    }
}

fn run_lineage_locator<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(action) = args.next() else {
        eprintln!("missing lineage locator action");
        eprintln!("{}", usage());
        return 2;
    };
    if action == "--help" || action == "-h" {
        println!("{}", usage());
        return 0;
    }
    match action.as_str() {
        "transport" => run_lineage_locator_transport(args),
        "point" => crate::cmd_distributed_shard_lineage_locator_pointer::run(args),
        "history" => crate::cmd_distributed_shard_lineage_locator_history::run(args),
        _ => {
            eprintln!("unknown lineage locator action: {action}");
            eprintln!("{}", usage());
            2
        }
    }
}

fn run_lineage_locator_transport<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(action) = args.next() else {
        eprintln!("missing lineage locator transport action");
        eprintln!("{}", usage());
        return 2;
    };
    if action == "--help" || action == "-h" {
        println!("{}", usage());
        return 0;
    }
    match action.as_str() {
        "point" => crate::cmd_distributed_shard_lineage_transport_locator::run(args),
        "reconcile" => crate::cmd_distributed_shard_lineage_transport_locator_reconcile::run(args),
        "history" => {
            crate::cmd_distributed_shard_lineage_transport_locator_reconciliation_history::run(args)
        }
        _ => {
            eprintln!("unknown lineage locator transport action: {action}");
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
    "usage: afterburner <train|infer|eval|deploy <...>|verify <...>|rollback <...>|drift <...>|cleanup <...>|profile <...>|source <...>|debug <deploy <...>|lineage <...>|simulate-scheduler-runtime <...>>> [args]"
}
