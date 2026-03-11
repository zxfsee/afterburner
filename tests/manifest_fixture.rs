use afterburner::manifest::{
    ACTIVATION_DTYPE, ArtifactManifest, CANONICALIZATION_METHOD, QUANTIZATION_KIND_NONE,
    SIGNATURE_KEY_ID_PLACEHOLDER, SIGNATURE_SCHEME_PLACEHOLDER, SIGNATURE_VALUE_PLACEHOLDER,
    WEIGHTS_DTYPE,
};

#[path = "fixture_support.rs"]
mod fixture_support;

#[test]
fn manifest_fixture_parses_and_validates() {
    let manifest_path = fixture_support::fixture_manifest_path();
    let weights = fixture_support::fixture_model_path();
    let parsed = ArtifactManifest::load_from_path(&manifest_path).expect("parse manifest");
    assert_eq!(parsed.artifact_version, "0.1.0");
    assert_eq!(parsed.signature.scheme, SIGNATURE_SCHEME_PLACEHOLDER);
    assert_eq!(parsed.signature.key_id, SIGNATURE_KEY_ID_PLACEHOLDER);
    assert_eq!(parsed.signature.value, SIGNATURE_VALUE_PLACEHOLDER);
    assert_eq!(parsed.canonicalization.method, CANONICALIZATION_METHOD);
    assert_eq!(parsed.precision.weights_dtype, WEIGHTS_DTYPE);
    assert_eq!(parsed.precision.activation_dtype, ACTIVATION_DTYPE);
    assert_eq!(parsed.precision.quantization, QUANTIZATION_KIND_NONE);
    parsed
        .validate_against_current(&weights)
        .expect("validate manifest");
}
