use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn optimized_model_local_profile_schema_and_workflow_are_explicit() {
    let schema_text = fs::read_to_string(fixture_path("optimized_model_local_profile.schema.json"))
        .expect("read optimized model local profile schema");
    let schema: Value =
        serde_json::from_str(&schema_text).expect("parse optimized model local profile schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://afterburner.local/schemas/optimized-model-local-profile/v1")
    );
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("schema.required must be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("schema.required items must be strings")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    let expected = [
        "schema_version",
        "package_contract_path",
        "optimization_profile_path",
        "input_artifact_version",
        "target_environment",
        "optimization_steps",
        "runtime_precision_contract",
        "optimized_artifact_path",
        "export_format",
        "quality",
        "latency",
        "memory",
        "package_size",
        "passed",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    assert_eq!(required, expected);

    let justfile = repo_file("justfile");
    assert!(
        justfile.contains(
            "optimized-model-local-profile package_contract quality_metric quality_value minimum_quality_value observed_latency_ms_p99 observed_max_memory_bytes observed_package_bytes:"
        ),
        "justfile must expose the optimized-model-local-profile workflow"
    );

    let readme = repo_file("README.md");
    for needle in [
        "optimized model local profile",
        "afterburner profile optimized-model-local-profile",
        "optimized_model_local_profile.json",
    ] {
        assert!(
            readme.contains(needle),
            "README must mention `{needle}` for the local profile contract"
        );
    }
}

#[test]
fn optimized_model_local_profile_command_writes_profile_and_event() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let optimization_profile_path = tmp.path().join("model_optimization_profile.json");
    fs::copy(
        fixture_path("model_optimization_profile.example.json"),
        &optimization_profile_path,
    )
    .expect("copy optimization profile");

    let package_contract_path = tmp.path().join("optimized_model_package_contract.json");
    let mut package_contract: Value = serde_json::from_str(
        &fs::read_to_string(fixture_path(
            "optimized_model_package_contract.example.json",
        ))
        .expect("read package contract fixture"),
    )
    .expect("parse package contract fixture");
    package_contract["packaging_inputs"]["optimization_profile_path"] =
        Value::from(optimization_profile_path.display().to_string());
    fs::write(
        &package_contract_path,
        serde_json::to_string_pretty(&package_contract).expect("serialize package contract"),
    )
    .expect("write package contract");

    let out = tmp.path().join("optimized_model_local_profile.json");
    let mut cmd = cargo_bin_cmd!("afterburner");
    cmd.arg("profile")
        .arg("optimized-model-local-profile")
        .arg("--package-contract")
        .arg(&package_contract_path)
        .arg("--quality-metric")
        .arg("accuracy")
        .arg("--quality-value")
        .arg("0.991")
        .arg("--minimum-quality-value")
        .arg("0.98925781")
        .arg("--observed-latency-ms-p99")
        .arg("210.0")
        .arg("--observed-max-memory-bytes")
        .arg("3221225472")
        .arg("--observed-package-bytes")
        .arg("1610612736")
        .arg("--out")
        .arg(&out);
    let assert = cmd.assert().success();

    let text = fs::read_to_string(&out).expect("read local profile");
    let mut profile: Value = serde_json::from_str(&text).expect("parse local profile");
    profile["package_contract_path"] = Value::from("<package-contract>");
    profile["optimization_profile_path"] = Value::from("<optimization-profile>");
    assert_eq!(
        profile,
        serde_json::json!({
            "schema_version": "1",
            "package_contract_path": "<package-contract>",
            "optimization_profile_path": "<optimization-profile>",
            "input_artifact_version": "0.1.0",
            "target_environment": "macbook-local",
            "optimization_steps": ["quantization", "compression", "packaging"],
            "runtime_precision_contract": {
                "weights_dtype": "f32",
                "activation_dtype": "f32",
                "quantization": "none"
            },
            "optimized_artifact_path": "artifacts/inference/0.1.0/model.optimized.mpk",
            "export_format": "burn-mpk",
            "quality": {
                "metric": "accuracy",
                "observed": 0.991,
                "minimum_accepted": 0.98925781,
                "passed": true
            },
            "latency": {
                "observed_ms_p99": 210.0,
                "budget_ms_p99": 250.0,
                "passed": true
            },
            "memory": {
                "observed_bytes": 3221225472_u64,
                "budget_bytes": 4294967296_u64,
                "passed": true
            },
            "package_size": {
                "observed_bytes": 1610612736_u64,
                "budget_bytes": 2147483648_u64,
                "passed": true
            },
            "passed": true
        })
    );

    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf8 stderr");
    let event = stderr
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|event| {
            event.get("event").and_then(Value::as_str)
                == Some("optimized_model_local_profile_written")
        })
        .unwrap_or_else(|| {
            panic!("missing optimized model local profile event in stderr: {stderr}")
        });
    let mut normalized_event = event.clone();
    let object = normalized_event
        .as_object_mut()
        .expect("event must be object");
    object.insert("ts_ms".to_string(), Value::from(0));
    let fields = object
        .get_mut("fields")
        .and_then(Value::as_object_mut)
        .expect("event fields must be object");
    fields.insert("profile_path".to_string(), Value::from("<profile>"));
    fields.insert(
        "package_contract_path".to_string(),
        Value::from("<package-contract>"),
    );
    fields.insert(
        "optimization_profile_path".to_string(),
        Value::from("<optimization-profile>"),
    );
    let event_profile = fields
        .get_mut("profile")
        .and_then(Value::as_object_mut)
        .expect("event profile must be object");
    event_profile.insert(
        "package_contract_path".to_string(),
        Value::from("<package-contract>"),
    );
    event_profile.insert(
        "optimization_profile_path".to_string(),
        Value::from("<optimization-profile>"),
    );
    assert_eq!(
        normalized_event,
        serde_json::json!({
            "ts_ms": 0,
            "level": "info",
            "source": "profile_cli",
            "event": "optimized_model_local_profile_written",
            "fields": {
                "profile_path": "<profile>",
                "profile": profile,
                "package_contract_path": "<package-contract>",
                "optimization_profile_path": "<optimization-profile>"
            }
        })
    );
}
