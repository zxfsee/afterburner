use std::path::Path;

#[test]
fn training_exports_inference_artifact() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();

    let (train_dir, infer_dir) = afterburner::train::artifact_dirs(root);
    std::fs::create_dir_all(&train_dir).expect("create train dir");

    // Dummy “trained” artifact (we only test the contract/export mechanics here)
    let train_model = train_dir.join("model.mpk");
    std::fs::write(&train_model, b"dummy").expect("write dummy model");

    let exported = afterburner::train::export_inference_artifact(&train_model, &infer_dir)
        .expect("export inference model");

    assert!(Path::new(&train_dir).is_dir());
    assert!(Path::new(&infer_dir).is_dir());
    assert!(
        exported.exists(),
        "missing inference artifact: {}",
        exported.display()
    );
}
