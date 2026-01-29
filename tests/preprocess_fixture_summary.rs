use burn::backend::ndarray::NdArray;
use burn::prelude::*;

use afterburner::preprocess::mnist_image_to_tensor;

#[path = "fixture_support.rs"]
mod fixture_support;

type B = NdArray<f32>;

#[test]
fn mnist_fixture_normalization_summary_matches() {
    let device = <B as Backend>::Device::default();
    let image = fixture_support::load_fixture_image();
    let expected = fixture_support::load_fixture_summary();

    let tensor = mnist_image_to_tensor::<B>(image, &device);
    let flat = tensor.reshape([784]);
    let data = flat.to_data();
    let values: Vec<f32> = data.iter().collect();

    let n = values.len() as f32;
    let mean = values.iter().sum::<f32>() / n;
    let var = values
        .iter()
        .map(|v| {
            let diff = v - mean;
            diff * diff
        })
        .sum::<f32>()
        / n;
    let std = var.sqrt();
    let min = values.iter().copied().fold(f32::INFINITY, f32::min);
    let max = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);

    assert!((mean - expected.mean).abs() <= 1e-6);
    assert!((std - expected.std).abs() <= 1e-6);
    assert!((min - expected.min).abs() <= 1e-6);
    assert!((max - expected.max).abs() <= 1e-6);
}
