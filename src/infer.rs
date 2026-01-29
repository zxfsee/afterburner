use std::env;
use std::path::{Path, PathBuf};

use crate::manifest::MANIFEST_FILENAME;

/// Default inference artifact path (ADR-002).
pub fn default_weights_path() -> PathBuf {
    PathBuf::from("artifacts/inference/model.mpk")
}

/// Parse optional CLI args into a weights path.
/// - `argv[0]` is ignored
/// - `argv[1]` (if present) is treated as the artifact path
///
/// Pure function => unit-testable without spawning a process.
pub fn parse_weights_path_from_args<I, S>(args: I) -> PathBuf
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter()
        .nth(1)
        .map(|s| PathBuf::from(s.as_ref()))
        .unwrap_or_else(default_weights_path)
}

/// Parse weights path from the actual process arguments.
pub fn parse_weights_path() -> PathBuf {
    parse_weights_path_from_args(env::args())
}

/// Derive the manifest path for a given weights file.
pub fn manifest_path_for_weights(weights_path: &Path) -> PathBuf {
    let parent = weights_path.parent().unwrap_or_else(|| Path::new("."));
    parent.join(MANIFEST_FILENAME)
}
