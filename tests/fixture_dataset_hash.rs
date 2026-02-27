use std::fmt::Write as _;
use std::path::PathBuf;

use sha2::{Digest, Sha256};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(&mut out, "{:02x}", byte);
    }
    out
}

#[test]
fn fixture_image_and_summary_hashes_are_stable() {
    let image = std::fs::read(fixture_path("mnist_0.png")).expect("read fixture image");
    let summary =
        std::fs::read(fixture_path("mnist_0.summary.toml")).expect("read fixture summary");

    assert_eq!(
        sha256_hex(&image),
        "e15505ff92348fd3315ecd7738797a44dd75e60084f8a3653b5e5840c162aec3"
    );
    assert_eq!(
        sha256_hex(&summary),
        "ea12b09ee5c31e3df30c1494a3454ec043400fe9d9a4bf9bbd0c411a5f3fd81c"
    );
}
