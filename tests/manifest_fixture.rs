use afterburner::manifest::ArtifactManifest;

#[path = "fixture_support.rs"]
mod fixture_support;

#[test]
fn manifest_fixture_parses_and_validates() {
    let manifest_path = fixture_support::fixture_manifest_path();
    let weights = fixture_support::fixture_model_path();
    let parsed = ArtifactManifest::load_from_path(&manifest_path).expect("parse manifest");
    parsed
        .validate_against_current(&weights)
        .expect("validate manifest");
}
