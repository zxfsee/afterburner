use afterburner::infer::{default_weights_path, parse_weights_path_from_args};
use afterburner::manifest::is_semver;

#[test]
fn default_path_is_contract_path() {
    assert_eq!(default_weights_path().to_string_lossy(), expected_default_path());
}

#[test]
fn parse_path_prefers_cli_arg_or_defaults() {
    let p = parse_weights_path_from_args(["infer", "x.mpk"]);
    assert_eq!(p.to_string_lossy(), "x.mpk");

    let p = parse_weights_path_from_args(["infer"]);
    assert_eq!(p.to_string_lossy(), expected_default_path());
}

fn expected_default_path() -> String {
    let current_path = std::path::Path::new("artifacts/inference/current");
    let version = std::fs::read_to_string(current_path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty() && is_semver(value));

    if let Some(version) = version {
        format!("artifacts/inference/{version}/model.mpk")
    } else {
        "artifacts/inference/model.mpk".to_string()
    }
}
