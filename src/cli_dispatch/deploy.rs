pub fn run_deploy<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(subcommand) = args.next() else {
        eprintln!("missing deploy subcommand");
        eprintln!("{}", super::usage());
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

pub fn run_debug_deploy<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(subcommand) = args.next() else {
        eprintln!("missing debug deploy subcommand");
        eprintln!("{}", super::usage());
        return 2;
    };
    if !is_debug_only_deploy_subcommand(&subcommand) {
        eprintln!("unknown debug deploy subcommand: {subcommand}");
        eprintln!("{}", super::usage());
        return 2;
    }
    dispatch_deploy(subcommand, args)
}

pub fn run_verify<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(subcommand) = args.next() else {
        eprintln!("missing verify subcommand");
        eprintln!("{}", super::usage());
        return 2;
    };

    match subcommand.as_str() {
        "receipt" => crate::cmd_deployment_verification_receipt::run(args),
        "bundle" => crate::cmd_deployment_verification_bundle::run(args),
        "handoff" => crate::cmd_deployment_verification_handoff::run(args),
        _ => {
            eprintln!("unknown verify subcommand: {subcommand}");
            eprintln!("{}", super::usage());
            2
        }
    }
}

pub fn run_rollback<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(subcommand) = args.next() else {
        eprintln!("missing rollback subcommand");
        eprintln!("{}", super::usage());
        return 2;
    };

    match subcommand.as_str() {
        "verification-receipt-locator" => {
            crate::cmd_deployment_verification_receipt_locator_rollback::run(args)
        }
        "verification-bundle-locator" => {
            crate::cmd_deployment_verification_bundle_locator_rollback::run(args)
        }
        "verification-bundle" => crate::cmd_deployment_verification_bundle_rollback::run(args),
        _ => {
            eprintln!("unknown rollback subcommand: {subcommand}");
            eprintln!("{}", super::usage());
            2
        }
    }
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

fn dispatch_deploy<I>(subcommand: String, args: I) -> i32
where
    I: Iterator<Item = String>,
{
    match subcommand.as_str() {
        "hf-publish" => crate::cmd_hf_publish::run(args),
        "record-kube-rs-lease-history" => crate::cmd_kube_rs_lease_history::run(args),
        "record-kube-rs-lease-reconciliation-history" => {
            crate::cmd_kube_rs_lease_reconciliation_history::run(args)
        }
        "point-kube-rs-lease" => crate::cmd_kube_rs_lease_pointer::run(args),
        "kube-rs-lease-reconcile" => crate::cmd_kube_rs_lease_reconcile::run(args),
        "upload" => crate::cmd_upload::run(args),
        "stack-check" => crate::cmd_deployment_stack_check::run(args),
        "stack-launch-bundle" => crate::cmd_deployment_stack_launch_bundle::run(args),
        "reconcile-launch-bundle" => crate::cmd_deployment_stack_launch_bundle_reconcile::run(args),
        "record-launch-bundle-reconciliation-history" => {
            crate::cmd_deployment_stack_launch_bundle_reconciliation_history::run(args)
        }
        "stack-launch-handoff" => crate::cmd_deployment_stack_launch_handoff::run(args),
        "record-launch-handoff-history" => crate::cmd_deployment_stack_launch_handoff_history::run(args),
        "reconcile-launch-handoff" => crate::cmd_deployment_stack_launch_handoff_reconcile::run(args),
        "record-launch-handoff-reconciliation-history" => {
            crate::cmd_deployment_stack_launch_handoff_reconciliation_history::run(args)
        }
        "record-launch-locator-history" => crate::cmd_deployment_stack_launch_locator_history::run(args),
        "record-launch-locator-reconciliation-history" => {
            crate::cmd_deployment_stack_launch_locator_reconciliation_history::run(args)
        }
        "record-launch-transport-locator-history" => {
            crate::cmd_deployment_stack_launch_transport_locator_history::run(args)
        }
        "record-launch-transport-locator-reconciliation-history" => {
            crate::cmd_deployment_stack_launch_transport_locator_reconciliation_history::run(args)
        }
        "stack-launch-plan" => crate::cmd_deployment_stack_launch_plan::run(args),
        "stack-launch-receipt" => crate::cmd_deployment_stack_launch_receipt::run(args),
        "point-launch-locator" => crate::cmd_deployment_stack_launch_locator_pointer::run(args),
        "reconcile-launch-locator" => crate::cmd_deployment_stack_launch_locator_reconcile::run(args),
        "reconcile-launch-transport-locator" => {
            crate::cmd_deployment_stack_launch_transport_locator_reconcile::run(args)
        }
        "point-launch-transport-locator" => {
            crate::cmd_deployment_stack_launch_transport_locator::run(args)
        }
        "record-verification-receipt-locator-history" => {
            crate::cmd_deployment_verification_receipt_locator_history::run(args)
        }
        "point-verification-receipt-locator" => {
            crate::cmd_deployment_verification_receipt_locator_pointer::run(args)
        }
        "reconcile-verification-receipt-locator" => {
            crate::cmd_deployment_verification_receipt_locator_reconcile::run(args)
        }
        "record-verification-receipt-locator-reconciliation-history" => {
            crate::cmd_deployment_verification_receipt_locator_reconciliation_history::run(args)
        }
        "record-verification-receipt-locator-rollback-history" => {
            crate::cmd_deployment_verification_receipt_locator_rollback_history::run(args)
        }
        "reconcile-verification-receipt-rollback" => {
            crate::cmd_deployment_verification_receipt_rollback_reconcile::run(args)
        }
        "record-verification-receipt-rollback-reconciliation-history" => {
            crate::cmd_deployment_verification_receipt_rollback_reconciliation_history::run(args)
        }
        "supersede-verification-receipt-rollback" => {
            crate::cmd_deployment_verification_receipt_rollback_supersession::run(args)
        }
        "reconcile-verification-receipt-rollback-supersession" => {
            crate::cmd_deployment_verification_receipt_rollback_supersession_reconcile::run(args)
        }
        "record-verification-receipt-rollback-supersession-reconciliation-history" => {
            crate::cmd_deployment_verification_receipt_rollback_supersession_reconciliation_history::run(
                args,
            )
        }
        "record-verification-receipt-rollback-supersession-history" => {
            crate::cmd_deployment_verification_receipt_rollback_supersession_history::run(args)
        }
        "point-verification-receipt-transport-locator" => {
            crate::cmd_deployment_verification_receipt_transport_locator::run(args)
        }
        "record-verification-receipt-transport-locator-history" => {
            crate::cmd_deployment_verification_receipt_transport_locator_history::run(args)
        }
        "reconcile-verification-receipt-transport-locator" => {
            crate::cmd_deployment_verification_receipt_transport_locator_reconcile::run(args)
        }
        "record-verification-receipt-transport-locator-reconciliation-history" => {
            crate::cmd_deployment_verification_receipt_transport_locator_reconciliation_history::run(args)
        }
        "record-verification-receipt-history" => {
            crate::cmd_deployment_verification_receipt_history::run(args)
        }
        "reconcile-verification-receipt" => {
            crate::cmd_deployment_verification_receipt_reconcile::run(args)
        }
        "record-verification-receipt-reconciliation-history" => {
            crate::cmd_deployment_verification_receipt_reconciliation_history::run(args)
        }
        "point-verification-bundle-locator" => {
            crate::cmd_deployment_verification_bundle_locator_pointer::run(args)
        }
        "record-verification-bundle-locator-history" => {
            crate::cmd_deployment_verification_bundle_locator_history::run(args)
        }
        "reconcile-verification-bundle-locator" => {
            crate::cmd_deployment_verification_bundle_locator_reconcile::run(args)
        }
        "record-verification-bundle-locator-reconciliation-history" => {
            crate::cmd_deployment_verification_bundle_locator_reconciliation_history::run(args)
        }
        "record-verification-bundle-locator-rollback-history" => {
            crate::cmd_deployment_verification_bundle_locator_rollback_history::run(args)
        }
        "record-verification-bundle-rollback-history" => {
            crate::cmd_deployment_verification_bundle_rollback_history::run(args)
        }
        "reconcile-verification-bundle-rollback" => {
            crate::cmd_deployment_verification_bundle_rollback_reconcile::run(args)
        }
        "record-verification-bundle-rollback-reconciliation-history" => {
            crate::cmd_deployment_verification_bundle_rollback_reconciliation_history::run(args)
        }
        "supersede-verification-bundle-rollback" => {
            crate::cmd_deployment_verification_bundle_rollback_supersession::run(args)
        }
        "reconcile-verification-bundle-rollback-supersession" => {
            crate::cmd_deployment_verification_bundle_rollback_supersession_reconcile::run(args)
        }
        "record-verification-bundle-rollback-supersession-reconciliation-history" => {
            crate::cmd_deployment_verification_bundle_rollback_supersession_reconciliation_history::run(
                args,
            )
        }
        "record-verification-bundle-rollback-supersession-history" => {
            crate::cmd_deployment_verification_bundle_rollback_supersession_history::run(args)
        }
        "record-verification-bundle-history" => {
            crate::cmd_deployment_verification_bundle_history::run(args)
        }
        "point-verification-bundle-transport-locator" => {
            crate::cmd_deployment_verification_bundle_transport_locator::run(args)
        }
        "record-verification-bundle-transport-locator-history" => {
            crate::cmd_deployment_verification_bundle_transport_locator_history::run(args)
        }
        "record-verification-bundle-transport-locator-reconciliation-history" => {
            crate::cmd_deployment_verification_bundle_transport_locator_reconciliation_history::run(args)
        }
        "reconcile-verification-bundle-transport-locator" => {
            crate::cmd_deployment_verification_bundle_transport_locator_reconcile::run(args)
        }
        "reconcile-verification-bundle" => crate::cmd_deployment_verification_bundle_reconcile::run(args),
        "record-verification-bundle-reconciliation-history" => {
            crate::cmd_deployment_verification_bundle_reconciliation_history::run(args)
        }
        "record-verification-handoff-history" => {
            crate::cmd_deployment_verification_handoff_history::run(args)
        }
        "point-verification-handoff-transport-locator" => {
            crate::cmd_deployment_verification_handoff_transport_locator::run(args)
        }
        "record-verification-handoff-transport-locator-history" => {
            crate::cmd_deployment_verification_handoff_transport_locator_history::run(args)
        }
        "reconcile-verification-handoff-transport-locator" => {
            crate::cmd_deployment_verification_handoff_transport_locator_reconcile::run(args)
        }
        "reconcile-verification-handoff" => {
            crate::cmd_deployment_verification_handoff_reconcile::run(args)
        }
        "record-verification-handoff-reconciliation-history" => {
            crate::cmd_deployment_verification_handoff_reconciliation_history::run(args)
        }
        "scheduler-heartbeat" => crate::cmd_scheduler_heartbeat::run(args),
        "record-scheduler-heartbeat-history" => crate::cmd_scheduler_heartbeat_history::run(args),
        "point-scheduler-heartbeat" => crate::cmd_scheduler_heartbeat_pointer::run(args),
        "record-scheduler-heartbeat-pointer-history" => {
            crate::cmd_scheduler_heartbeat_pointer_history::run(args)
        }
        "record-scheduler-heartbeat-pointer-rollback-history" => {
            crate::cmd_scheduler_heartbeat_pointer_rollback_history::run(args)
        }
        "record-scheduler-heartbeat-pointer-rollback-reconciliation-history" => {
            crate::cmd_scheduler_heartbeat_pointer_rollback_reconciliation_history::run(args)
        }
        "reconcile-scheduler-heartbeat-pointer-rollback" => {
            crate::cmd_scheduler_heartbeat_pointer_rollback_reconcile::run(args)
        }
        "scheduler-heartbeat-point-rollback-supersede" => {
            crate::cmd_scheduler_heartbeat_pointer_rollback_supersession::run(args)
        }
        "record-scheduler-heartbeat-pointer-rollback-supersession-history" => {
            crate::cmd_scheduler_heartbeat_pointer_rollback_supersession_history::run(args)
        }
        "reconcile-scheduler-heartbeat-pointer-rollback-supersession" => {
            crate::cmd_scheduler_heartbeat_pointer_rollback_supersession_reconcile::run(args)
        }
        "record-scheduler-heartbeat-pointer-rollback-supersession-reconciliation-history" => {
            crate::cmd_scheduler_heartbeat_pointer_rollback_supersession_reconciliation_history::run(args)
        }
        "rollback-scheduler-heartbeat-pointer" => {
            crate::cmd_scheduler_heartbeat_pointer_rollback::run(args)
        }
        "record-scheduler-heartbeat-pointer-supersession-history" => {
            crate::cmd_scheduler_heartbeat_pointer_supersession_history::run(args)
        }
        "record-scheduler-heartbeat-pointer-supersession-reconciliation-history" => {
            crate::cmd_scheduler_heartbeat_pointer_supersession_reconciliation_history::run(args)
        }
        "reconcile-scheduler-heartbeat-pointer-supersession" => {
            crate::cmd_scheduler_heartbeat_pointer_supersession_reconcile::run(args)
        }
        "scheduler-heartbeat-point-supersede" => {
            crate::cmd_scheduler_heartbeat_pointer_supersession::run(args)
        }
        "reconcile-scheduler-heartbeat-pointer" => {
            crate::cmd_scheduler_heartbeat_pointer_reconcile::run(args)
        }
        "scheduler-heartbeat-reconcile" => crate::cmd_scheduler_heartbeat_reconcile::run(args),
        "record-scheduler-heartbeat-reconciliation-history" => {
            crate::cmd_scheduler_heartbeat_reconciliation_history::run(args)
        }
        "scheduler-heartbeat-supersede" => crate::cmd_scheduler_heartbeat_supersession::run(args),
        "scheduler-heartbeat-supersession-reconcile" => {
            crate::cmd_scheduler_heartbeat_supersession_reconcile::run(args)
        }
        "record-scheduler-heartbeat-supersession-reconciliation-history" => {
            crate::cmd_scheduler_heartbeat_supersession_reconciliation_history::run(args)
        }
        "record-scheduler-heartbeat-supersession-history" => {
            crate::cmd_scheduler_heartbeat_supersession_history::run(args)
        }
        "load-profile" => crate::cmd_distributed_load_profile::run(args),
        "single-node-scheduler" => crate::cmd_single_node_scheduler::run(args),
        _ => {
            eprintln!("unknown deploy subcommand: {subcommand}");
            eprintln!("{}", super::usage());
            2
        }
    }
}
