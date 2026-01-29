use burn::backend::ndarray::NdArray;
use burn::prelude::*; // brings Module into scope for save_file/load_file
use burn::record::CompactRecorder;

use afterburner::manifest::{ArtifactManifest, INPUT_DTYPE, INPUT_SHAPE, MANIFEST_FILENAME};
use afterburner::model::ModelConfig;
use afterburner::model::{MODEL_ARCH_ID, MODEL_ARCH_VERSION};
use afterburner::preprocess::{MNIST_MEAN, MNIST_NORMALIZATION_NOTES, MNIST_STD};

type CpuBackend = NdArray<f32>;

#[test]
fn training_exports_inference_artifact_and_it_is_loadable() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();

    let (train_dir, infer_dir) = afterburner::train::artifact_dirs(root);
    std::fs::create_dir_all(&train_dir).expect("create train dir");

    let device = <CpuBackend as Backend>::Device::default();
    let recorder = CompactRecorder::new();

    // Contract-only “trained” artifact: we only care that the export is valid and loadable.
    let train_model = train_dir.join("model.mpk");
    let model = ModelConfig::new(10).init::<CpuBackend>(&device);

    model
        .save_file(&train_model, &recorder)
        .expect("save model");

    assert!(
        train_dir.is_dir(),
        "missing train dir: {}",
        train_dir.display()
    );

    std::fs::create_dir_all(&infer_dir).expect("create inference dir");
    let exported = afterburner::train::export_inference_artifact(&train_model, &infer_dir)
        .expect("export inference model");

    assert!(
        infer_dir.is_dir(),
        "missing inference dir: {}",
        infer_dir.display()
    );
    assert!(
        exported.exists(),
        "missing exported artifact: {}",
        exported.display()
    );

    let manifest_path = infer_dir.join(MANIFEST_FILENAME);
    assert!(
        manifest_path.exists(),
        "missing manifest: {}",
        manifest_path.display()
    );
    let manifest = ArtifactManifest::load_from_path(&manifest_path).expect("load manifest");
    assert_eq!(manifest.artifact, "model.mpk");
    assert_eq!(manifest.model.architecture_id, MODEL_ARCH_ID);
    assert_eq!(manifest.model.architecture_version, MODEL_ARCH_VERSION);
    assert_eq!(manifest.input.shape, INPUT_SHAPE);
    assert_eq!(manifest.input.dtype, INPUT_DTYPE);
    assert_eq!(manifest.normalization.dataset, "mnist");
    assert_eq!(manifest.normalization.mean, MNIST_MEAN);
    assert_eq!(manifest.normalization.std, MNIST_STD);
    assert_eq!(manifest.normalization.notes, MNIST_NORMALIZATION_NOTES);

    let _loaded = ModelConfig::new(10)
        .init::<CpuBackend>(&device)
        .load_file(&exported, &recorder, &device)
        .expect("load exported inference artifact");
}
