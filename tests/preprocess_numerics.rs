use burn::backend::ndarray::NdArray;
use burn::prelude::*;

use afterburner::preprocess::{MNIST_MEAN, MNIST_STD, mnist_image_to_tensor};

#[path = "fixture_support.rs"]
mod fixture_support;

type B = NdArray<f32>;

#[test]
fn preprocess_values_are_finite_and_denormalize_to_unit_interval() {
    let device = <B as Backend>::Device::default();
    let image = fixture_support::load_fixture_image();
    let tensor = mnist_image_to_tensor::<B>(image, &device);
    let values: Vec<f32> = tensor.to_data().iter().collect();

    for (idx, value) in values.iter().copied().enumerate() {
        assert!(
            value.is_finite(),
            "non-finite normalized value at index {idx}"
        );

        // Reverse ((x/255)-mean)/std back to x/255 and enforce [0,1] bounds.
        let unit = (value * MNIST_STD) + MNIST_MEAN;
        assert!(
            (0.0..=1.0).contains(&unit),
            "denormalized unit value out of [0,1] at index {idx}: {unit}"
        );
    }
}
