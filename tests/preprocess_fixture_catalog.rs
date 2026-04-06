use std::fmt::Write as _;

use burn::backend::ndarray::NdArray;
use burn::prelude::*;
use sha2::{Digest, Sha256};

use afterburner::data::training_preprocess_mnist_image;
use afterburner::preprocess::{MNIST_MEAN, MNIST_STD, mnist_image_to_tensor};

#[path = "support/fixture.rs"]
mod fixture_test_support;
#[path = "support/mnist_fixture.rs"]
mod mnist_fixture;

use fixture_test_support::fixture_path;

type B = NdArray<f32>;

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(&mut out, "{:02x}", byte);
    }
    out
}

#[test]
fn preprocess_shape_and_finiteness() {
    let device = <B as Backend>::Device::default();
    let image = [[0.0f32; 28]; 28];

    let x = afterburner::preprocess::mnist_image_to_tensor::<B>(image, &device);

    assert_eq!(x.dims(), [1, 28, 28]);

    let flat = x.reshape([784]);
    let data = flat.to_data();
    assert!(data.iter::<f32>().all(|v| v.is_finite()));
}

#[test]
fn preprocess_values_are_finite_and_denormalize_to_unit_interval() {
    let device = <B as Backend>::Device::default();
    let image = mnist_fixture::load_fixture_image();
    let tensor = mnist_image_to_tensor::<B>(image, &device);
    let values: Vec<f32> = tensor.to_data().iter().collect();

    for (idx, value) in values.iter().copied().enumerate() {
        assert!(
            value.is_finite(),
            "non-finite normalized value at index {idx}"
        );

        let unit = (value * MNIST_STD) + MNIST_MEAN;
        assert!(
            (0.0..=1.0).contains(&unit),
            "denormalized unit value out of [0,1] at index {idx}: {unit}"
        );
    }
}

#[test]
fn preprocess_parity_between_training_and_infer_paths() {
    let device = <B as Backend>::Device::default();
    let image = mnist_fixture::load_fixture_image();

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

#[test]
fn mnist_fixture_normalization_summary_matches() {
    let device = <B as Backend>::Device::default();
    let image = mnist_fixture::load_fixture_image();
    let expected = mnist_fixture::load_fixture_summary();

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
