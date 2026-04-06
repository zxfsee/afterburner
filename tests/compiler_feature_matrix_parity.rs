use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use afterburner::RUNTIME_SUPPORTED_BACKENDS;

fn cargo_manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
}

#[test]
fn compiler_feature_matrix_fixture_matches_effective_burn_features() {
    let cargo_toml_text = fs::read_to_string(cargo_manifest_path()).expect("read Cargo.toml");
    let cargo_toml: toml::Value = toml::from_str(&cargo_toml_text).expect("parse Cargo.toml");

    let matrix = cargo_toml
        .get("package")
        .and_then(|v| v.get("metadata"))
        .and_then(|v| v.get("afterburner"))
        .and_then(|v| v.get("compiler_feature_matrix"))
        .expect("package.metadata.afterburner.compiler_feature_matrix must exist");

    assert_eq!(
        matrix.get("schema_version").and_then(|v| v.as_str()),
        Some("1")
    );

    let entries = matrix
        .get("backend_feature_matrix")
        .and_then(|v| v.as_array())
        .expect("backend_feature_matrix must be an array");
    assert!(
        !entries.is_empty(),
        "backend_feature_matrix must contain at least one backend"
    );

    let burn_features = effective_burn_features();
    let mut backends = BTreeSet::new();

    for entry in entries {
        let backend = entry
            .get("backend")
            .and_then(|v| v.as_str())
            .expect("backend must be a string");
        assert!(
            backends.insert(backend.to_string()),
            "duplicate backend in matrix: {backend}"
        );

        let enabled_features = entry
            .get("enabled_features")
            .and_then(|v| v.as_array())
            .expect("enabled_features must be an array");
        assert!(
            !enabled_features.is_empty(),
            "enabled_features must not be empty for backend: {backend}"
        );

        for feature in enabled_features {
            let feature = feature
                .as_str()
                .expect("enabled_features values must be strings");
            assert!(
                burn_features.contains(feature),
                "backend `{backend}` requires feature `{feature}` missing from effective burn features: {burn_features:?}"
            );
        }
    }

    let expected_backends = RUNTIME_SUPPORTED_BACKENDS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let declared_backends = backends.iter().map(String::as_str).collect::<BTreeSet<_>>();
    assert_eq!(
        declared_backends, expected_backends,
        "matrix must declare all supported backends"
    );
}

fn effective_burn_features() -> BTreeSet<String> {
    let metadata = cargo_metadata_json();
    let resolve = metadata
        .get("resolve")
        .expect("cargo metadata missing resolve section");
    let root_id = resolve
        .get("root")
        .and_then(|v| v.as_str())
        .expect("cargo metadata missing resolve.root");
    let nodes = resolve
        .get("nodes")
        .and_then(|v| v.as_array())
        .expect("cargo metadata missing resolve.nodes array");

    let root_node = nodes
        .iter()
        .find(|node| node.get("id").and_then(|v| v.as_str()) == Some(root_id))
        .expect("resolve node for root package missing");
    let burn_pkg_id = root_node
        .get("deps")
        .and_then(|v| v.as_array())
        .and_then(|deps| {
            deps.iter().find_map(|dep| {
                let is_burn = dep.get("name").and_then(|v| v.as_str()) == Some("burn");
                if !is_burn {
                    return None;
                }
                dep.get("pkg").and_then(|v| v.as_str())
            })
        })
        .expect("root package dependency `burn` missing from cargo metadata");

    let burn_node = nodes
        .iter()
        .find(|node| node.get("id").and_then(|v| v.as_str()) == Some(burn_pkg_id))
        .expect("resolve node for `burn` missing");
    burn_node
        .get("features")
        .and_then(|v| v.as_array())
        .expect("burn resolve node missing features array")
        .iter()
        .map(|feature| {
            feature
                .as_str()
                .expect("burn feature values must be strings")
                .to_string()
        })
        .collect()
}

fn cargo_metadata_json() -> serde_json::Value {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--format-version")
        .arg("1")
        .arg("--locked")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run `cargo metadata`");

    assert!(
        output.status.success(),
        "`cargo metadata` failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("parse cargo metadata json")
}
