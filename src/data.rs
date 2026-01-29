use std::sync::Arc;

use burn::{
    data::dataloader::batcher::Batcher,
    data::dataset::vision::{MnistDataset, MnistItem},
    prelude::*,
    tensor::TensorData,
};

use crate::preprocess::{MNIST_MEAN, MNIST_STD};

/// Batched MNIST tensors ready for a convolutional classifier.
#[derive(Clone, Debug)]
pub struct MnistBatch<B: Backend> {
    pub images: Tensor<B, 4>,
    pub targets: Tensor<B, 1, Int>,
}

#[derive(Clone, Default)]
pub struct MnistBatcher;

impl<B: Backend> Batcher<B, MnistItem, MnistBatch<B>> for MnistBatcher {
    fn batch(&self, items: Vec<MnistItem>, device: &B::Device) -> MnistBatch<B> {
        let images = items
            .iter()
            .map(|item| TensorData::from(item.image).convert::<B::FloatElem>())
            .map(|data| Tensor::<B, 2>::from_data(data, device))
            .map(|tensor| tensor.reshape([1, 28, 28]))
            .map(|tensor| ((tensor / 255.0) - MNIST_MEAN) / MNIST_STD) // standard MNIST normalization
            .collect();

        let images = Tensor::stack(images, 0);
        let targets = items
            .iter()
            .map(|item| Tensor::<B, 1, Int>::from_ints([item.label as i64], device))
            .collect::<Vec<_>>();
        let targets = Tensor::cat(targets, 0);

        MnistBatch { images, targets }
    }
}

pub fn train_loader<B: Backend>(
    batch_size: usize,
    num_workers: usize,
    seed: u64,
    device: B::Device,
) -> Arc<dyn burn::data::dataloader::DataLoader<B, MnistBatch<B>>> {
    burn::data::dataloader::DataLoaderBuilder::new(MnistBatcher)
        .batch_size(batch_size)
        .shuffle(seed)
        .num_workers(num_workers)
        .set_device(device)
        .build(MnistDataset::train())
}

pub fn test_loader<B: Backend>(
    batch_size: usize,
    num_workers: usize,
    device: B::Device,
) -> Arc<dyn burn::data::dataloader::DataLoader<B, MnistBatch<B>>> {
    burn::data::dataloader::DataLoaderBuilder::new(MnistBatcher)
        .batch_size(batch_size)
        .num_workers(num_workers)
        .set_device(device)
        .build(MnistDataset::test())
}
