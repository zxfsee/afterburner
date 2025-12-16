use burn::prelude::*;
use burn::tensor::TensorData;

/// MNIST normalization constants used by training.
/// Keeping them named makes the contract auditable.
pub const MNIST_MEAN: f32 = 0.1307;
pub const MNIST_STD: f32 = 0.3081;

/// Convert a single MNIST image to a `[1,1,28,28]` tensor and normalize.
pub fn mnist_image_to_tensor<B: Backend>(
    image: [[f32; 28]; 28],
    device: &B::Device,
) -> Tensor<B, 4> {
    let data = TensorData::from(image).convert::<B::FloatElem>();
    let t = ((Tensor::<B, 2>::from_data(data, device).reshape([1, 28, 28]) / 255.0) - MNIST_MEAN)
        / MNIST_STD;

    Tensor::stack(vec![t], 0)
}
