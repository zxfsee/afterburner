use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn justfile_thinness_debt_stays_bounded() {
    let justfile = repo_file("justfile");
    let mut current_recipe = None::<String>;
    let mut branching_recipes = BTreeSet::new();
    let mut action_arg_recipes = BTreeSet::new();
    let mut expanded_recipes = BTreeSet::new();

    for line in justfile.lines() {
        let trimmed = line.trim_end();
        if !trimmed.is_empty()
            && !line.starts_with(' ')
            && !line.starts_with('\t')
            && trimmed.ends_with(':')
        {
            let recipe = trimmed.trim_end_matches(':').to_string();
            if recipe.ends_with(" action +args") {
                action_arg_recipes.insert(recipe.clone());
            }
            if !recipe.ends_with(" action +args")
                && !recipe.starts_with("rollout-")
                && (recipe.contains("-record-")
                    || recipe.contains("-history-")
                    || recipe.contains("-history ")
                    || recipe.contains("-reconcile")
                    || recipe.contains("reconciliation-history")
                    || recipe.contains("-point-")
                    || recipe.contains("-point ")
                    || recipe.starts_with("deploy-launch-transport-locator ")
                    || recipe.starts_with("deploy-launch-locator ")
                    || recipe.starts_with("distributed-shard-lineage-locator-pointer ")
                    || recipe.starts_with("distributed-shard-lineage-transport-locator ")
                    || recipe.starts_with("deployment-verification-handoff-transport-locator ")
                    || recipe.starts_with("deployment-verification-receipt-transport-locator ")
                    || recipe.starts_with("deployment-verification-receipt-locator-pointer ")
                    || recipe.starts_with("deployment-verification-bundle-transport-locator ")
                    || recipe.starts_with("deployment-verification-bundle-locator-pointer ")
                    || recipe.contains("-rollback")
                    || recipe.contains("-supersede")
                    || recipe.starts_with("kube-rs-lease-"))
            {
                expanded_recipes.insert(recipe.clone());
            }
            current_recipe = Some(recipe);
            continue;
        }

        if line.trim_start().starts_with("if ")
            && let Some(recipe) = current_recipe.clone()
        {
            branching_recipes.insert(recipe);
        }
    }

    let allowed_branching = BTreeSet::new();
    assert_eq!(
        branching_recipes, allowed_branching,
        "justfile control-flow branching must stay out of the canonical workflow surface"
    );

    let allowed_action_args = BTreeSet::new();
    assert_eq!(
        action_arg_recipes, allowed_action_args,
        "generic `action +args` buckets must stay bounded to the explicit allowlist until their families are removed end to end"
    );

    let allowed_expanded = BTreeSet::from([
        "deploy-launch-bundle-reconcile receipt bundle".to_string(),
        "deploy-launch-bundle-reconciliation-history reconciliation event recorded_at_unix_ms".to_string(),
        "deploy-launch-handoff-reconcile bundle handoff".to_string(),
        "deploy-launch-handoff-history handoff event recorded_at_unix_ms".to_string(),
        "deploy-launch-handoff-reconciliation-history reconciliation event recorded_at_unix_ms".to_string(),
        "deploy-launch-transport-locator handoff".to_string(),
        "deploy-launch-transport-locator-history locator event recorded_at_unix_ms".to_string(),
        "deploy-launch-transport-locator-reconciliation-history reconciliation event recorded_at_unix_ms".to_string(),
        "deploy-launch-transport-locator-reconcile handoff locator".to_string(),
        "deploy-launch-locator locator".to_string(),
        "deploy-launch-locator-reconcile plan pointer".to_string(),
        "deploy-launch-locator-history pointer event recorded_at_unix_ms".to_string(),
        "deploy-launch-locator-reconciliation-history reconciliation event recorded_at_unix_ms".to_string(),
        "kube-rs-lease-reconcile lease namespace resource_name".to_string(),
        "kube-rs-lease-pointer reconciliation".to_string(),
        "kube-rs-lease-history pointer event recorded_at_unix_ms".to_string(),
        "kube-rs-lease-reconciliation-history reconciliation event recorded_at_unix_ms".to_string(),
        "scheduler-heartbeat-history heartbeat event recorded_at_unix_ms".to_string(),
        "scheduler-heartbeat-reconcile heartbeat current_heartbeat".to_string(),
        "scheduler-heartbeat-reconciliation-history reconciliation event recorded_at_unix_ms".to_string(),
        "scheduler-heartbeat-supersede previous_heartbeat next_heartbeat superseded_at_unix_ms".to_string(),
        "scheduler-heartbeat-point heartbeat".to_string(),
        "scheduler-heartbeat-supersession-history supersession event recorded_at_unix_ms".to_string(),
        "scheduler-heartbeat-supersession-reconcile supersession current_supersession".to_string(),
        "scheduler-heartbeat-supersession-reconciliation-history reconciliation event recorded_at_unix_ms".to_string(),
        "scheduler-heartbeat-pointer-history pointer event recorded_at_unix_ms".to_string(),
        "scheduler-heartbeat-pointer-reconcile pointer current_pointer".to_string(),
        "scheduler-heartbeat-pointer-supersede previous_pointer next_pointer superseded_at_unix_ms".to_string(),
        "scheduler-heartbeat-pointer-supersession-history supersession event recorded_at_unix_ms".to_string(),
        "scheduler-heartbeat-pointer-supersession-reconcile supersession current_supersession".to_string(),
        "scheduler-heartbeat-pointer-supersession-reconciliation-history reconciliation event recorded_at_unix_ms".to_string(),
        "scheduler-heartbeat-pointer-rollback current_pointer restored_pointer rolled_back_at_unix_ms".to_string(),
        "scheduler-heartbeat-pointer-rollback-history rollback event recorded_at_unix_ms".to_string(),
        "scheduler-heartbeat-pointer-rollback-reconcile rollback current_rollback".to_string(),
        "scheduler-heartbeat-pointer-rollback-reconciliation-history reconciliation event recorded_at_unix_ms".to_string(),
        "scheduler-heartbeat-pointer-rollback-supersede previous_rollback next_rollback superseded_at_unix_ms".to_string(),
        "scheduler-heartbeat-pointer-rollback-supersession-history supersession event recorded_at_unix_ms".to_string(),
        "scheduler-heartbeat-pointer-rollback-supersession-reconcile supersession current_supersession".to_string(),
        "scheduler-heartbeat-pointer-rollback-supersession-reconciliation-history reconciliation event recorded_at_unix_ms".to_string(),
        "deployment-verification-bundle-history bundle event recorded_at_unix_ms".to_string(),
        "deployment-verification-bundle-reconciliation-history reconciliation event recorded_at_unix_ms".to_string(),
        "deployment-verification-bundle-reconcile receipt bundle".to_string(),
        "deployment-verification-bundle-transport-locator bundle".to_string(),
        "deployment-verification-bundle-transport-locator-history locator event recorded_at_unix_ms".to_string(),
        "deployment-verification-bundle-transport-locator-reconcile bundle locator".to_string(),
        "deployment-verification-bundle-transport-reconciliation-history reconciliation event recorded_at_unix_ms".to_string(),
        "deployment-verification-bundle-locator-pointer locator".to_string(),
        "deployment-verification-bundle-locator-history pointer event recorded_at_unix_ms".to_string(),
        "deployment-verification-bundle-locator-reconcile locator pointer".to_string(),
        "deployment-verification-bundle-locator-reconciliation-history reconciliation event recorded_at_unix_ms".to_string(),
        "deployment-verification-bundle-rollback-pointer current_pointer restored_pointer rolled_back_at_unix_ms".to_string(),
        "deployment-verification-bundle-rollback-pointer-history rollback event recorded_at_unix_ms".to_string(),
        "deployment-verification-bundle-rollback-apply current_bundle restored_bundle rolled_back_at_unix_ms".to_string(),
        "deployment-verification-bundle-rollback-apply-history rollback event recorded_at_unix_ms".to_string(),
        "deployment-verification-bundle-rollback-apply-reconcile rollback current_rollback".to_string(),
        "deployment-verification-bundle-rollback-apply-reconciliation-history reconciliation event recorded_at_unix_ms".to_string(),
        "deployment-verification-bundle-rollback-apply-supersede previous_rollback next_rollback superseded_at_unix_ms".to_string(),
        "deployment-verification-bundle-rollback-apply-supersession-history supersession event recorded_at_unix_ms".to_string(),
        "deployment-verification-bundle-rollback-apply-supersession-reconcile supersession current_supersession".to_string(),
        "deployment-verification-bundle-rollback-apply-supersession-reconciliation-history reconciliation event recorded_at_unix_ms".to_string(),
        "deployment-verification-handoff-history handoff event recorded_at_unix_ms".to_string(),
        "deployment-verification-handoff-reconciliation-history reconciliation event recorded_at_unix_ms".to_string(),
        "deployment-verification-handoff-reconcile bundle handoff".to_string(),
        "deployment-verification-handoff-transport-locator handoff".to_string(),
        "deployment-verification-handoff-transport-locator-history locator event recorded_at_unix_ms".to_string(),
        "deployment-verification-handoff-transport-locator-reconcile handoff locator".to_string(),
        "deployment-verification-receipt-history receipt event recorded_at_unix_ms".to_string(),
        "deployment-verification-receipt-reconciliation-history reconciliation event recorded_at_unix_ms".to_string(),
        "deployment-verification-receipt-reconcile receipt artifact_version profile_name verification_status verified_at_unix_ms evidence evidence_source_1 evidence_source_2 +evidence_sources".to_string(),
        "deployment-verification-receipt-transport-locator receipt".to_string(),
        "deployment-verification-receipt-transport-locator-history locator event recorded_at_unix_ms".to_string(),
        "deployment-verification-receipt-transport-locator-reconcile receipt locator".to_string(),
        "deployment-verification-receipt-transport-reconciliation-history reconciliation event recorded_at_unix_ms".to_string(),
        "deployment-verification-receipt-locator-pointer locator".to_string(),
        "deployment-verification-receipt-locator-history pointer event recorded_at_unix_ms".to_string(),
        "deployment-verification-receipt-locator-reconcile locator pointer".to_string(),
        "deployment-verification-receipt-locator-reconciliation-history reconciliation event recorded_at_unix_ms".to_string(),
        "deployment-verification-receipt-rollback-pointer current_pointer restored_pointer rolled_back_at_unix_ms".to_string(),
        "deployment-verification-receipt-rollback-pointer-history rollback event recorded_at_unix_ms".to_string(),
        "deployment-verification-receipt-rollback-reconcile rollback current_rollback".to_string(),
        "deployment-verification-receipt-rollback-reconciliation-history reconciliation event recorded_at_unix_ms".to_string(),
        "deployment-verification-receipt-rollback-supersede previous_rollback next_rollback superseded_at_unix_ms".to_string(),
        "deployment-verification-receipt-rollback-supersession-history supersession event recorded_at_unix_ms".to_string(),
        "deployment-verification-receipt-rollback-supersession-reconcile supersession current_supersession".to_string(),
        "deployment-verification-receipt-rollback-supersession-reconciliation-history reconciliation event recorded_at_unix_ms".to_string(),
        "distributed-shard-lineage-bundle-history reconciliation event recorded_at_unix_ms".to_string(),
        "distributed-shard-lineage-bundle-reconcile receipt bundle".to_string(),
        "distributed-shard-lineage-handoff-history handoff event recorded_at_unix_ms".to_string(),
        "distributed-shard-lineage-handoff-reconciliation-history reconciliation event recorded_at_unix_ms".to_string(),
        "distributed-shard-lineage-handoff-reconcile bundle handoff".to_string(),
        "distributed-shard-lineage-locator-pointer locator".to_string(),
        "distributed-shard-lineage-locator-history pointer event recorded_at_unix_ms".to_string(),
        "distributed-shard-lineage-transport-locator handoff".to_string(),
        "distributed-shard-lineage-transport-locator-reconcile handoff locator".to_string(),
        "distributed-shard-lineage-transport-locator-history reconciliation event recorded_at_unix_ms".to_string(),
        "drift-approved-baseline-history pointer event recorded_at_unix_ms".to_string(),
        "drift-approved-baseline-rollback current_pointer restored_approval rolled_back_at_unix_ms".to_string(),
        "drift-approved-baseline-supersede previous_approval next_approval superseded_at_unix_ms".to_string(),
    ]);
    assert_eq!(
        expanded_recipes, allowed_expanded,
        "artifact-family expansion in justfile must stay bounded to the explicit current allowlist until those families are removed end to end"
    );

    for forbidden in [
        "-manage action +args:",
        "deployment-verification-receipt-manage action +args:",
        "deployment-verification-bundle-manage action +args:",
        "deployment-verification-handoff-manage action +args:",
        "distributed-shard-lineage-bundle-manage action +args:",
        "distributed-shard-lineage-handoff-manage action +args:",
        "distributed-shard-lineage-locator-manage action +args:",
        "scheduler-heartbeat-manage action +args:",
        "scheduler-heartbeat-supersession action +args:",
        "scheduler-heartbeat-pointer action +args:",
        "scheduler-heartbeat-pointer-supersession action +args:",
        "scheduler-heartbeat-pointer-rollback action +args:",
        "scheduler-heartbeat-pointer-rollback-supersession action +args:",
    ] {
        assert!(
            !justfile.contains(forbidden),
            "justfile must not regress to retired generic bucket `{forbidden}`"
        );
    }
}
