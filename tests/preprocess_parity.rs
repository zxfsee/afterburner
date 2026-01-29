use burn::backend::ndarray::NdArray;
use burn::prelude::*;

use afterburner::data::training_preprocess_mnist_image;
use afterburner::preprocess::mnist_image_to_tensor;

#[path = "fixture_support.rs"]
mod fixture_support;

type B = NdArray<f32>;

#[test]
fn preprocess_parity_between_training_and_infer_paths() {
    let device = <B as Backend>::Device::default();
    let image = fixture_support::load_fixture_image();

    let infer_tensor = mnist_image_to_tensor::<B>(image, &device);
    let train_tensor = training_preprocess_mnist_image::<B>(image, &device);

    let infer_data = infer_tensor.to_data();
    let train_data = train_tensor.to_data();

    let infer_vals: Vec<f32> = infer_data.iter().collect();
    let train_vals: Vec<f32> = train_data.iter().collect();

    assert_eq!(infer_vals.len(), train_vals.len());
    for (idx, (a, b)) in infer_vals.iter().zip(train_vals.iter()).enumerate() {
        let diff = (a - b).abs();
        assert!(diff <= 1e-6, "tensor mismatch at {idx}: {a} vs {b}");
    }
}
