use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

mod support;

use support::{fixture_path, repo_file};

#[test]
fn single_node_scheduler_contracts_are_explicit() {
    let job_schema_text = fs::read_to_string(fixture_path("gpu_scheduler_job_request.schema.json"))
        .expect("read job request schema");
    let job_schema: Value =
        serde_json::from_str(&job_schema_text).expect("parse job request schema");
    assert_eq!(
        job_schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/gpu-scheduler-job-request/v1")
    );

    let inventory_schema_text = fs::read_to_string(fixture_path("gpu_inventory.schema.json"))
        .expect("read inventory schema");
    let inventory_schema: Value =
        serde_json::from_str(&inventory_schema_text).expect("parse inventory schema");
    assert_eq!(
        inventory_schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/gpu-inventory/v1")
    );

    let lease_schema_text = fs::read_to_string(fixture_path("gpu_scheduler_lease.schema.json"))
        .expect("read lease schema");
    let lease_schema: Value = serde_json::from_str(&lease_schema_text).expect("parse lease schema");
    assert_eq!(
        lease_schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/gpu-scheduler-lease/v1")
    );

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains("deploy single-node-scheduler"),
        "justfile must expose the single-node scheduler workflow"
    );

    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-046"),
        "architecture decisions index must link ADR-046"
    );
}

#[test]
fn single_node_scheduler_writes_lease_and_unit() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let lease_path = tmp.path().join("gpu_scheduler_lease.json");
    let unit_path = tmp.path().join("afterburner-job-0001.service");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("deploy")
        .arg("single-node-scheduler")
        .arg("--job")
        .arg(fixture_path("gpu_scheduler_job_request.example.json"))
        .arg("--inventory")
        .arg(fixture_path("gpu_inventory.example.json"))
        .arg("--out-lease")
        .arg(&lease_path)
        .arg("--out-unit")
        .arg(&unit_path);
    let assert = cmd.assert().success();

    let lease_text = fs::read_to_string(&lease_path).expect("read lease");
    let lease: Value = serde_json::from_str(&lease_text).expect("parse lease");
    let expected_lease_text = fs::read_to_string(fixture_path("gpu_scheduler_lease.example.json"))
        .expect("read expected lease");
    let expected_lease: Value =
        serde_json::from_str(&expected_lease_text).expect("parse expected lease");
    assert_eq!(lease, expected_lease);

    let unit_text = fs::read_to_string(&unit_path).expect("read unit");
    let expected_unit = fs::read_to_string(fixture_path("afterburner-job-0001.service"))
        .expect("read unit fixture");
    assert_eq!(unit_text, expected_unit);

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str) == Some("single_node_gpu_lease_written")
        })
        .expect("missing single_node_gpu_lease_written event");
    let fields = event
        .get("fields")
        .and_then(Value::as_object)
        .expect("event fields must be object");
    assert_eq!(
        fields.get("lease_path").and_then(Value::as_str),
        Some(lease_path.display().to_string().as_str())
    );
    assert_eq!(
        fields.get("unit_path").and_then(Value::as_str),
        Some(unit_path.display().to_string().as_str())
    );
}

#[test]
fn single_node_scheduler_rejects_multi_gpu_request() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let job_path = tmp.path().join("job.json");
    fs::write(
        &job_path,
        r#"{"schema_version":"1","job_id":"job-0002","priority":10,"gpu_count":2,"checkpoint_mode":"cooperative","working_directory":"/srv/afterburner","exec_start":"./bin/afterburner train --num-epochs 1"}"#,
    )
    .expect("write job request");

    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("deploy")
        .arg("single-node-scheduler")
        .arg("--job")
        .arg(&job_path)
        .arg("--inventory")
        .arg(fixture_path("gpu_inventory.example.json"));
    cmd.assert().failure().code(2);
}
