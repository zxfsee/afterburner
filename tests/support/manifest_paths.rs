use std::path::PathBuf;

pub fn fixture_model_path() -> PathBuf {
    fixture_path("model.mpk")
}

pub fn fixture_manifest_path() -> PathBuf {
    fixture_path("manifest.toml")
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}
