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
            current_recipe = Some(recipe);
            continue;
        }

        if line.trim_start().starts_with("if ") && let Some(recipe) = current_recipe.clone() {
            branching_recipes.insert(recipe);
        }
    }

    let allowed_branching = BTreeSet::from(["queue-execute-preflight repair=''".to_string()]);
    assert_eq!(
        branching_recipes, allowed_branching,
        "justfile control-flow branching must stay limited to the explicit queue preflight recovery path"
    );

    let allowed_action_args = BTreeSet::from([
        "scheduler-heartbeat-supersession action +args".to_string(),
        "scheduler-heartbeat-pointer action +args".to_string(),
        "scheduler-heartbeat-pointer-supersession action +args".to_string(),
        "scheduler-heartbeat-pointer-rollback action +args".to_string(),
        "scheduler-heartbeat-pointer-rollback-supersession action +args".to_string(),
        "deployment-verification-bundle-history action +args".to_string(),
        "deployment-verification-bundle-transport action +args".to_string(),
        "deployment-verification-bundle-locator action +args".to_string(),
        "deployment-verification-bundle-rollback action +args".to_string(),
        "deployment-verification-handoff-history action +args".to_string(),
        "deployment-verification-handoff-transport action +args".to_string(),
        "deployment-verification-receipt-history action +args".to_string(),
        "deployment-verification-receipt-transport action +args".to_string(),
        "deployment-verification-receipt-locator action +args".to_string(),
        "deployment-verification-receipt-rollback action +args".to_string(),
        "distributed-shard-lineage-handoff-history action +args".to_string(),
        "distributed-shard-lineage-locator action +args".to_string(),
        "distributed-shard-lineage-locator-transport action +args".to_string(),
        "pretraining-source-provenance action +args".to_string(),
    ]);
    assert_eq!(
        action_arg_recipes, allowed_action_args,
        "generic `action +args` buckets must stay bounded to the explicit allowlist until their families are removed end to end"
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
    ] {
        assert!(
            !justfile.contains(forbidden),
            "justfile must not regress to retired generic bucket `{forbidden}`"
        );
    }
}
