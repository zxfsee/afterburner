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
    if let Some(target) = legacy_debug_deploy_target(&subcommand) {
        eprintln!(
            "deploy subcommand `{subcommand}` is debug-only; use `afterburner debug deploy {target} ...`"
        );
        return 2;
    }
    dispatch_deploy(subcommand, args)
}

pub fn run_debug_deploy<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(group) = args.next() else {
        eprintln!("missing debug deploy subcommand");
        eprintln!("{}", super::usage());
        return 2;
    };
    if let Some(target) = legacy_debug_deploy_target(&group) {
        eprintln!(
            "debug deploy subcommand `{group}` moved to `{target}`; use `afterburner debug deploy {target} ...`"
        );
        return 2;
    }
    if group == "scheduler-heartbeat" {
        return run_debug_scheduler_heartbeat(args);
    }
    let Some(action) = args.next() else {
        eprintln!("missing debug deploy action for `{group}`");
        eprintln!("{}", super::usage());
        return 2;
    };
    let Some(subcommand) = regrouped_debug_deploy_subcommand(&group, &action) else {
        eprintln!("unknown debug deploy command: {group} {action}");
        eprintln!("{}", super::usage());
        return 2;
    };
    dispatch_deploy(subcommand, args)
}

fn run_debug_scheduler_heartbeat<I>(args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let args = args.collect::<Vec<_>>();
    let Some(action) = args.first().cloned() else {
        eprintln!("missing debug deploy action for `scheduler-heartbeat`");
        eprintln!("{}", super::usage());
        return 2;
    };
    if action == "--help" || action == "-h" {
        println!("{}", super::usage());
        return 0;
    }
    let rest = args.into_iter().skip(1);
    match action.as_str() {
        "point" => dispatch_deploy("point-scheduler-heartbeat".to_string(), rest),
        "history" => dispatch_deploy("record-scheduler-heartbeat-history".to_string(), rest),
        "reconcile" => dispatch_deploy("scheduler-heartbeat-reconcile".to_string(), rest),
        "reconciliation-history" => dispatch_deploy(
            "record-scheduler-heartbeat-reconciliation-history".to_string(),
            rest,
        ),
        "supersede" => dispatch_deploy("scheduler-heartbeat-supersede".to_string(), rest),
        "supersession" => run_debug_scheduler_heartbeat_supersession(rest),
        "pointer" => run_debug_scheduler_heartbeat_pointer(rest),
        _ => {
            eprintln!("unknown debug deploy scheduler-heartbeat action: {action}");
            eprintln!("{}", super::usage());
            2
        }
    }
}

fn run_debug_scheduler_heartbeat_supersession<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(action) = args.next() else {
        eprintln!("missing debug deploy scheduler-heartbeat supersession action");
        eprintln!("{}", super::usage());
        return 2;
    };
    if action == "--help" || action == "-h" {
        println!("{}", super::usage());
        return 0;
    }
    match action.as_str() {
        "history" => dispatch_deploy(
            "record-scheduler-heartbeat-supersession-history".to_string(),
            args,
        ),
        "reconcile" => dispatch_deploy(
            "scheduler-heartbeat-supersession-reconcile".to_string(),
            args,
        ),
        "reconciliation-history" => dispatch_deploy(
            "record-scheduler-heartbeat-supersession-reconciliation-history".to_string(),
            args,
        ),
        _ => {
            eprintln!("unknown debug deploy scheduler-heartbeat supersession action: {action}");
            eprintln!("{}", super::usage());
            2
        }
    }
}

fn run_debug_scheduler_heartbeat_pointer<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(action) = args.next() else {
        eprintln!("missing debug deploy scheduler-heartbeat pointer action");
        eprintln!("{}", super::usage());
        return 2;
    };
    if action == "--help" || action == "-h" {
        println!("{}", super::usage());
        return 0;
    }
    match action.as_str() {
        "history" => dispatch_deploy(
            "record-scheduler-heartbeat-pointer-history".to_string(),
            args,
        ),
        "reconcile" => dispatch_deploy("reconcile-scheduler-heartbeat-pointer".to_string(), args),
        "supersede" => dispatch_deploy("scheduler-heartbeat-point-supersede".to_string(), args),
        "supersession" => run_debug_scheduler_heartbeat_pointer_supersession(args),
        "rollback" => run_debug_scheduler_heartbeat_pointer_rollback(args),
        _ => {
            eprintln!("unknown debug deploy scheduler-heartbeat pointer action: {action}");
            eprintln!("{}", super::usage());
            2
        }
    }
}

fn run_debug_scheduler_heartbeat_pointer_supersession<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(action) = args.next() else {
        eprintln!("missing debug deploy scheduler-heartbeat pointer supersession action");
        eprintln!("{}", super::usage());
        return 2;
    };
    if action == "--help" || action == "-h" {
        println!("{}", super::usage());
        return 0;
    }
    match action.as_str() {
        "history" => dispatch_deploy(
            "record-scheduler-heartbeat-pointer-supersession-history".to_string(),
            args,
        ),
        "reconcile" => dispatch_deploy(
            "reconcile-scheduler-heartbeat-pointer-supersession".to_string(),
            args,
        ),
        "reconciliation-history" => dispatch_deploy(
            "record-scheduler-heartbeat-pointer-supersession-reconciliation-history".to_string(),
            args,
        ),
        _ => {
            eprintln!(
                "unknown debug deploy scheduler-heartbeat pointer supersession action: {action}"
            );
            eprintln!("{}", super::usage());
            2
        }
    }
}

fn run_debug_scheduler_heartbeat_pointer_rollback<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(action) = args.next() else {
        return dispatch_deploy("rollback-scheduler-heartbeat-pointer".to_string(), args);
    };
    if action.starts_with('-') {
        return dispatch_deploy(
            "rollback-scheduler-heartbeat-pointer".to_string(),
            std::iter::once(action).chain(args),
        );
    }
    if action == "--help" || action == "-h" {
        println!("{}", super::usage());
        return 0;
    }
    match action.as_str() {
        "history" => dispatch_deploy(
            "record-scheduler-heartbeat-pointer-rollback-history".to_string(),
            args,
        ),
        "reconcile" => dispatch_deploy(
            "reconcile-scheduler-heartbeat-pointer-rollback".to_string(),
            args,
        ),
        "reconciliation-history" => dispatch_deploy(
            "record-scheduler-heartbeat-pointer-rollback-reconciliation-history".to_string(),
            args,
        ),
        "supersede" => dispatch_deploy(
            "scheduler-heartbeat-point-rollback-supersede".to_string(),
            args,
        ),
        "supersession" => run_debug_scheduler_heartbeat_pointer_rollback_supersession(args),
        _ => {
            eprintln!("unknown debug deploy scheduler-heartbeat pointer rollback action: {action}");
            eprintln!("{}", super::usage());
            2
        }
    }
}

fn run_debug_scheduler_heartbeat_pointer_rollback_supersession<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(action) = args.next() else {
        eprintln!("missing debug deploy scheduler-heartbeat pointer rollback supersession action");
        eprintln!("{}", super::usage());
        return 2;
    };
    if action == "--help" || action == "-h" {
        println!("{}", super::usage());
        return 0;
    }
    match action.as_str() {
        "history" => dispatch_deploy(
            "record-scheduler-heartbeat-pointer-rollback-supersession-history".to_string(),
            args,
        ),
        "reconcile" => dispatch_deploy(
            "reconcile-scheduler-heartbeat-pointer-rollback-supersession".to_string(),
            args,
        ),
        "reconciliation-history" => dispatch_deploy(
            "record-scheduler-heartbeat-pointer-rollback-supersession-reconciliation-history"
                .to_string(),
            args,
        ),
        _ => {
            eprintln!(
                "unknown debug deploy scheduler-heartbeat pointer rollback supersession action: {action}"
            );
            eprintln!("{}", super::usage());
            2
        }
    }
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
        "rollout" => crate::cmd_verify_rollout::run(args),
        "receipt" => run_verify_receipt(args),
        "bundle" => run_verify_bundle(args),
        "handoff" => run_verify_handoff(args),
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
        "current-pointer" => crate::cmd_rollback_current_pointer::run(args),
        "verification-receipt" => run_rollback_verification_receipt(args),
        "verification-bundle" => run_rollback_verification_bundle(args),
        "verification-receipt-locator" => {
            crate::cmd_deployment_verification_receipt_locator_rollback::run(args)
        }
        "verification-bundle-locator" => {
            crate::cmd_deployment_verification_bundle_locator_rollback::run(args)
        }
        _ => {
            eprintln!("unknown rollback subcommand: {subcommand}");
            eprintln!("{}", super::usage());
            2
        }
    }
}

fn run_verify_receipt<I>(args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let args = args.collect::<Vec<_>>();
    let Some(action) = args.first().cloned() else {
        return crate::cmd_deployment_verification_receipt::run(args.into_iter());
    };
    if action.starts_with('-') {
        return crate::cmd_deployment_verification_receipt::run(args.into_iter());
    }
    let rest = args.into_iter().skip(1);
    match action.as_str() {
        "history" => run_verify_receipt_history(rest),
        "reconcile" => crate::cmd_deployment_verification_receipt_reconcile::run(rest),
        "transport" => run_verify_receipt_transport(rest),
        "locator" => run_verify_receipt_locator(rest),
        _ => {
            eprintln!("unknown verify receipt action: {action}");
            eprintln!("{}", super::usage());
            2
        }
    }
}

fn run_verify_receipt_history<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(action) = args.next() else {
        eprintln!("missing verify receipt history action");
        eprintln!("{}", super::usage());
        return 2;
    };
    if action == "--help" || action == "-h" {
        println!("{}", super::usage());
        return 0;
    }
    match action.as_str() {
        "record" => crate::cmd_deployment_verification_receipt_history::run(args),
        "reconciliation" => {
            crate::cmd_deployment_verification_receipt_reconciliation_history::run(args)
        }
        _ => {
            eprintln!("unknown verify receipt history action: {action}");
            eprintln!("{}", super::usage());
            2
        }
    }
}

fn run_verify_receipt_transport<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(action) = args.next() else {
        eprintln!("missing verify receipt transport action");
        eprintln!("{}", super::usage());
        return 2;
    };
    if action == "--help" || action == "-h" {
        println!("{}", super::usage());
        return 0;
    }
    match action.as_str() {
        "point" => crate::cmd_deployment_verification_receipt_transport_locator::run(args),
        "history" => {
            crate::cmd_deployment_verification_receipt_transport_locator_history::run(args)
        }
        "reconcile" => {
            crate::cmd_deployment_verification_receipt_transport_locator_reconcile::run(args)
        }
        "reconciliation-history" => {
            crate::cmd_deployment_verification_receipt_transport_locator_reconciliation_history::run(
                args,
            )
        }
        _ => {
            eprintln!("unknown verify receipt transport action: {action}");
            eprintln!("{}", super::usage());
            2
        }
    }
}

fn run_verify_receipt_locator<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(action) = args.next() else {
        eprintln!("missing verify receipt locator action");
        eprintln!("{}", super::usage());
        return 2;
    };
    if action == "--help" || action == "-h" {
        println!("{}", super::usage());
        return 0;
    }
    match action.as_str() {
        "point" => crate::cmd_deployment_verification_receipt_locator_pointer::run(args),
        "history" => crate::cmd_deployment_verification_receipt_locator_history::run(args),
        "reconcile" => crate::cmd_deployment_verification_receipt_locator_reconcile::run(args),
        "reconciliation-history" => {
            crate::cmd_deployment_verification_receipt_locator_reconciliation_history::run(args)
        }
        _ => {
            eprintln!("unknown verify receipt locator action: {action}");
            eprintln!("{}", super::usage());
            2
        }
    }
}

fn run_verify_bundle<I>(args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let args = args.collect::<Vec<_>>();
    let Some(action) = args.first().cloned() else {
        return crate::cmd_deployment_verification_bundle::run(args.into_iter());
    };
    if action.starts_with('-') {
        return crate::cmd_deployment_verification_bundle::run(args.into_iter());
    }
    let rest = args.into_iter().skip(1);
    match action.as_str() {
        "history" => run_verify_bundle_history(rest),
        "reconcile" => crate::cmd_deployment_verification_bundle_reconcile::run(rest),
        "transport" => run_verify_bundle_transport(rest),
        "locator" => run_verify_bundle_locator(rest),
        _ => {
            eprintln!("unknown verify bundle action: {action}");
            eprintln!("{}", super::usage());
            2
        }
    }
}

fn run_verify_bundle_history<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(action) = args.next() else {
        eprintln!("missing verify bundle history action");
        eprintln!("{}", super::usage());
        return 2;
    };
    if action == "--help" || action == "-h" {
        println!("{}", super::usage());
        return 0;
    }
    match action.as_str() {
        "record" => crate::cmd_deployment_verification_bundle_history::run(args),
        "reconciliation" => {
            crate::cmd_deployment_verification_bundle_reconciliation_history::run(args)
        }
        _ => {
            eprintln!("unknown verify bundle history action: {action}");
            eprintln!("{}", super::usage());
            2
        }
    }
}

fn run_verify_bundle_transport<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(action) = args.next() else {
        eprintln!("missing verify bundle transport action");
        eprintln!("{}", super::usage());
        return 2;
    };
    if action == "--help" || action == "-h" {
        println!("{}", super::usage());
        return 0;
    }
    match action.as_str() {
        "point" => crate::cmd_deployment_verification_bundle_transport_locator::run(args),
        "history" => crate::cmd_deployment_verification_bundle_transport_locator_history::run(args),
        "reconcile" => {
            crate::cmd_deployment_verification_bundle_transport_locator_reconcile::run(args)
        }
        "reconciliation-history" => {
            crate::cmd_deployment_verification_bundle_transport_locator_reconciliation_history::run(
                args,
            )
        }
        _ => {
            eprintln!("unknown verify bundle transport action: {action}");
            eprintln!("{}", super::usage());
            2
        }
    }
}

fn run_verify_bundle_locator<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(action) = args.next() else {
        eprintln!("missing verify bundle locator action");
        eprintln!("{}", super::usage());
        return 2;
    };
    if action == "--help" || action == "-h" {
        println!("{}", super::usage());
        return 0;
    }
    match action.as_str() {
        "point" => crate::cmd_deployment_verification_bundle_locator_pointer::run(args),
        "history" => crate::cmd_deployment_verification_bundle_locator_history::run(args),
        "reconcile" => crate::cmd_deployment_verification_bundle_locator_reconcile::run(args),
        "reconciliation-history" => {
            crate::cmd_deployment_verification_bundle_locator_reconciliation_history::run(args)
        }
        _ => {
            eprintln!("unknown verify bundle locator action: {action}");
            eprintln!("{}", super::usage());
            2
        }
    }
}

fn run_verify_handoff<I>(args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let args = args.collect::<Vec<_>>();
    let Some(action) = args.first().cloned() else {
        return crate::cmd_deployment_verification_handoff::run(args.into_iter());
    };
    if action.starts_with('-') {
        return crate::cmd_deployment_verification_handoff::run(args.into_iter());
    }
    let rest = args.into_iter().skip(1);
    match action.as_str() {
        "history" => run_verify_handoff_history(rest),
        "reconcile" => crate::cmd_deployment_verification_handoff_reconcile::run(rest),
        "transport" => run_verify_handoff_transport(rest),
        _ => {
            eprintln!("unknown verify handoff action: {action}");
            eprintln!("{}", super::usage());
            2
        }
    }
}

fn run_verify_handoff_history<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(action) = args.next() else {
        eprintln!("missing verify handoff history action");
        eprintln!("{}", super::usage());
        return 2;
    };
    if action == "--help" || action == "-h" {
        println!("{}", super::usage());
        return 0;
    }
    match action.as_str() {
        "record" => crate::cmd_deployment_verification_handoff_history::run(args),
        "reconciliation" => {
            crate::cmd_deployment_verification_handoff_reconciliation_history::run(args)
        }
        _ => {
            eprintln!("unknown verify handoff history action: {action}");
            eprintln!("{}", super::usage());
            2
        }
    }
}

fn run_verify_handoff_transport<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(action) = args.next() else {
        eprintln!("missing verify handoff transport action");
        eprintln!("{}", super::usage());
        return 2;
    };
    if action == "--help" || action == "-h" {
        println!("{}", super::usage());
        return 0;
    }
    match action.as_str() {
        "point" => crate::cmd_deployment_verification_handoff_transport_locator::run(args),
        "history" => {
            crate::cmd_deployment_verification_handoff_transport_locator_history::run(args)
        }
        "reconcile" => {
            crate::cmd_deployment_verification_handoff_transport_locator_reconcile::run(args)
        }
        _ => {
            eprintln!("unknown verify handoff transport action: {action}");
            eprintln!("{}", super::usage());
            2
        }
    }
}

fn run_rollback_verification_receipt<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let Some(action) = args.next() else {
        eprintln!("missing rollback verification-receipt action");
        eprintln!("{}", super::usage());
        return 2;
    };
    if action == "--help" || action == "-h" {
        println!("{}", super::usage());
        return 0;
    }
    match action.as_str() {
        "locator" => crate::cmd_deployment_verification_receipt_locator_rollback::run(args),
        "locator-history" => crate::cmd_deployment_verification_receipt_locator_rollback_history::run(args),
        "reconcile" => crate::cmd_deployment_verification_receipt_rollback_reconcile::run(args),
        "reconciliation-history" => crate::cmd_deployment_verification_receipt_rollback_reconciliation_history::run(args),
        "supersede" => crate::cmd_deployment_verification_receipt_rollback_supersession::run(args),
        "supersession-history" => crate::cmd_deployment_verification_receipt_rollback_supersession_history::run(args),
        "supersession-reconcile" => crate::cmd_deployment_verification_receipt_rollback_supersession_reconcile::run(args),
        "supersession-reconciliation-history" => {
            crate::cmd_deployment_verification_receipt_rollback_supersession_reconciliation_history::run(args)
        }
        _ => {
            eprintln!("unknown rollback verification-receipt action: {action}");
            eprintln!("{}", super::usage());
            2
        }
    }
}

fn run_rollback_verification_bundle<I>(mut args: I) -> i32
where
    I: Iterator<Item = String>,
{
    let args = args.collect::<Vec<_>>();
    let Some(action) = args.first().cloned() else {
        eprintln!("missing rollback verification-bundle action");
        eprintln!("{}", super::usage());
        return 2;
    };
    if action == "--help" || action == "-h" {
        println!("{}", super::usage());
        return 0;
    }
    if action.starts_with('-') {
        return crate::cmd_deployment_verification_bundle_rollback::run(args.into_iter());
    }
    let rest = args.into_iter().skip(1);
    match action.as_str() {
        "locator" => crate::cmd_deployment_verification_bundle_locator_rollback::run(rest),
        "locator-history" => crate::cmd_deployment_verification_bundle_locator_rollback_history::run(rest),
        "apply" => crate::cmd_deployment_verification_bundle_rollback::run(rest),
        "history" => crate::cmd_deployment_verification_bundle_rollback_history::run(rest),
        "reconcile" => crate::cmd_deployment_verification_bundle_rollback_reconcile::run(rest),
        "reconciliation-history" => crate::cmd_deployment_verification_bundle_rollback_reconciliation_history::run(rest),
        "supersede" => crate::cmd_deployment_verification_bundle_rollback_supersession::run(rest),
        "supersession-history" => crate::cmd_deployment_verification_bundle_rollback_supersession_history::run(rest),
        "supersession-reconcile" => crate::cmd_deployment_verification_bundle_rollback_supersession_reconcile::run(rest),
        "supersession-reconciliation-history" => {
            crate::cmd_deployment_verification_bundle_rollback_supersession_reconciliation_history::run(rest)
        }
        _ => {
            eprintln!("unknown rollback verification-bundle action: {action}");
            eprintln!("{}", super::usage());
            2
        }
    }
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

fn regrouped_debug_deploy_subcommand(group: &str, action: &str) -> Option<String> {
    if let Some(legacy) = legacy_verification_subcommand(group, action) {
        return Some(legacy);
    }

    None
}

fn legacy_debug_deploy_target(subcommand: &str) -> Option<String> {
    if let Some(target) = legacy_verification_target(subcommand) {
        return Some(target);
    }

    let target = match subcommand {
        "point-scheduler-heartbeat" => "scheduler-heartbeat point",
        "record-scheduler-heartbeat-pointer-history" => "scheduler-heartbeat pointer history",
        "reconcile-scheduler-heartbeat-pointer" => "scheduler-heartbeat pointer reconcile",
        "scheduler-heartbeat-point-supersede" => "scheduler-heartbeat pointer supersede",
        "record-scheduler-heartbeat-pointer-supersession-history" => {
            "scheduler-heartbeat pointer supersession history"
        }
        "reconcile-scheduler-heartbeat-pointer-supersession" => {
            "scheduler-heartbeat pointer supersession reconcile"
        }
        "record-scheduler-heartbeat-pointer-supersession-reconciliation-history" => {
            "scheduler-heartbeat pointer supersession reconciliation-history"
        }
        "rollback-scheduler-heartbeat-pointer" => "scheduler-heartbeat pointer rollback",
        "record-scheduler-heartbeat-pointer-rollback-history" => {
            "scheduler-heartbeat pointer rollback history"
        }
        "reconcile-scheduler-heartbeat-pointer-rollback" => {
            "scheduler-heartbeat pointer rollback reconcile"
        }
        "record-scheduler-heartbeat-pointer-rollback-reconciliation-history" => {
            "scheduler-heartbeat pointer rollback reconciliation-history"
        }
        "scheduler-heartbeat-point-rollback-supersede" => {
            "scheduler-heartbeat pointer rollback supersede"
        }
        "record-scheduler-heartbeat-pointer-rollback-supersession-history" => {
            "scheduler-heartbeat pointer rollback supersession history"
        }
        "reconcile-scheduler-heartbeat-pointer-rollback-supersession" => {
            "scheduler-heartbeat pointer rollback supersession reconcile"
        }
        "record-scheduler-heartbeat-pointer-rollback-supersession-reconciliation-history" => {
            "scheduler-heartbeat pointer rollback supersession reconciliation-history"
        }
        "record-scheduler-heartbeat-history" => "scheduler-heartbeat history",
        "scheduler-heartbeat-reconcile" => "scheduler-heartbeat reconcile",
        "record-scheduler-heartbeat-reconciliation-history" => {
            "scheduler-heartbeat reconciliation-history"
        }
        "scheduler-heartbeat-supersede" => "scheduler-heartbeat supersede",
        "scheduler-heartbeat-supersession-reconcile" => {
            "scheduler-heartbeat supersession reconcile"
        }
        "record-scheduler-heartbeat-supersession-history" => {
            "scheduler-heartbeat supersession history"
        }
        "record-scheduler-heartbeat-supersession-reconciliation-history" => {
            "scheduler-heartbeat supersession reconciliation-history"
        }
        _ => return None,
    };

    Some(target.to_string())
}

fn legacy_verification_subcommand(group: &str, action: &str) -> Option<String> {
    let family = group.strip_prefix("verification-")?;
    if !matches!(family, "receipt" | "bundle" | "handoff") {
        return None;
    }
    if !matches!(
        action,
        "record-history"
            | "point-locator"
            | "record-locator-history"
            | "reconcile-locator"
            | "record-locator-reconciliation-history"
            | "record-locator-rollback-history"
            | "record-rollback-history"
            | "reconcile-rollback"
            | "record-rollback-reconciliation-history"
            | "supersede-rollback"
            | "reconcile-rollback-supersession"
            | "record-rollback-supersession-reconciliation-history"
            | "record-rollback-supersession-history"
            | "point-transport-locator"
            | "record-transport-locator-history"
            | "reconcile-transport-locator"
            | "record-transport-locator-reconciliation-history"
            | "reconcile"
            | "record-reconciliation-history"
    ) {
        return None;
    }

    Some(legacy_verification_target_parts(family, action))
}

fn legacy_verification_target(subcommand: &str) -> Option<String> {
    let (verb, rest) = subcommand.split_once("-verification-")?;
    if !matches!(verb, "record" | "point" | "reconcile" | "supersede") {
        return None;
    }
    let (family, suffix) = verification_family_and_suffix(rest)?;
    let mut target = format!("verification-{family} {verb}");
    if !suffix.is_empty() {
        target.push('-');
        target.push_str(suffix);
    }
    Some(target)
}

fn legacy_verification_target_parts(family: &str, action: &str) -> String {
    let (verb, suffix) = action.split_once('-').unwrap_or((action, ""));
    let mut subcommand = format!("{verb}-verification-{family}");
    if !suffix.is_empty() {
        subcommand.push('-');
        subcommand.push_str(suffix);
    }
    subcommand
}

fn verification_family_and_suffix(rest: &str) -> Option<(&str, &str)> {
    for family in ["receipt", "bundle", "handoff"] {
        if rest == family {
            return Some((family, ""));
        }
        if let Some(suffix) = rest.strip_prefix(&format!("{family}-")) {
            return Some((family, suffix));
        }
    }
    None
}

fn dispatch_deploy<I>(subcommand: String, args: I) -> i32
where
    I: Iterator<Item = String>,
{
    match subcommand.as_str() {
        "promote-current" => crate::cmd_deploy_promote_current::run(args),
        "rollout-check" => crate::cmd_deploy_rollout_check::run(args),
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
