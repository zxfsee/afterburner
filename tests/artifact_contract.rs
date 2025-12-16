use burn::backend::ndarray::NdArray;
use burn::prelude::*; // brings Module into scope for save_file/load_file
use burn::record::CompactRecorder;

use afterburner::model::ModelConfig;

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

    let exported = afterburner::train::export_inference_artifact(&train_model, &infer_dir)
        .expect("export inference model");

    let _loaded = ModelConfig::new(10)
        .init::<CpuBackend>(&device)
        .load_file(&exported, &recorder, &device)
        .expect("load exported inference artifact");
}
