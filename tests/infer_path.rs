use afterburner::infer::{default_weights_path, parse_weights_path_from_args};

#[test]
fn default_path_is_contract_path() {
    assert_eq!(
        default_weights_path().to_string_lossy(),
        "artifacts/inference/model.mpk"
    );
}

#[test]
fn parse_path_prefers_cli_arg_or_defaults() {
    let p = parse_weights_path_from_args(["infer", "x.mpk"]);
    assert_eq!(p.to_string_lossy(), "x.mpk");

    let p = parse_weights_path_from_args(["infer"]);
    assert_eq!(p.to_string_lossy(), "artifacts/inference/model.mpk");
}
