use std::env;
use std::path::PathBuf;

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
