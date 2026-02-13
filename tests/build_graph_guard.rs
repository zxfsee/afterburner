use std::fs;
use std::path::{Path, PathBuf};

fn collect_rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).expect("read dir");
    for entry in entries {
        let entry = entry.expect("read dir entry");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, out);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

#[test]
fn core_modules_do_not_depend_on_transport_only_crates() {
    let src_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let adapter_bin_root = src_root.join("bin");
    let mut rust_files = Vec::new();
    collect_rust_files(&src_root, &mut rust_files);

    for path in rust_files {
        if path.starts_with(&adapter_bin_root) {
            continue;
        }

        let contents = fs::read_to_string(&path).expect("read rust file");
        assert!(
            !contents.contains("tiny_http"),
            "core module must not depend on transport-only crate `tiny_http`: {}",
            path.display()
        );
        assert!(
            !contents.contains("ctrlc::"),
            "core module must not depend on transport-only crate `ctrlc`: {}",
            path.display()
        );
    }
}
