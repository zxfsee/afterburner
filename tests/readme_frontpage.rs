use std::fs;
use std::path::PathBuf;

#[test]
fn readme_frontpage_is_sharp_and_scannable() {
    let readme = fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("README.md"))
        .unwrap_or_else(|err| panic!("read README.md: {err}"));

    let first_block = readme.lines().take(80).collect::<Vec<_>>().join("\n");
    for heading in [
        "## Quick start",
        "## At a glance",
        "## System shape",
        "## Read next",
    ] {
        assert!(
            first_block.contains(heading),
            "README frontpage must include `{heading}` near the top"
        );
    }

    assert!(
        first_block.contains("The canonical workflow surface is `just`"),
        "README frontpage must state the canonical workflow surface near the top"
    );
    assert!(
        first_block.contains("[Architecture](./ARCHITECTURE.md)")
            && first_block.contains("[ADRs](./docs/adr/)")
            && first_block.contains("[Changelog](./CHANGELOG.md)"),
        "README frontpage must point readers to deeper repo docs near the top"
    );
}
