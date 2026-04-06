use std::fs;
use std::path::PathBuf;

pub fn load_manifest_fixture() -> String {
    let path = fixture_path("manifest.toml");
    fs::read_to_string(path).expect("read manifest template")
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}
