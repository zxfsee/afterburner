use burn::backend::ndarray::NdArray;
use burn::prelude::*;

type B = NdArray<f32>;

#[test]
fn preprocess_shape_and_finiteness() {
    let device = <B as Backend>::Device::default();
    let image = [[0.0f32; 28]; 28];

    let x = afterburner::preprocess::mnist_image_to_tensor::<B>(image, &device);

    assert_eq!(x.dims(), [1, 1, 28, 28]);

    let flat = x.reshape([784]);
    let data = flat.to_data();
    assert!(data.iter::<f32>().all(|v| v.is_finite()));
}
