use burn::{
    nn::{
        conv::{Conv2d, Conv2dConfig},
        pool::{AdaptiveAvgPool2d, AdaptiveAvgPool2dConfig, MaxPool2d, MaxPool2dConfig},
        Dropout, DropoutConfig, Linear, LinearConfig, PaddingConfig2d, Relu,
    },
    prelude::*,
    train::ClassificationOutput,
};

#[derive(Module, Debug)]
pub struct Model<B: Backend> {
    stem: Conv2d<B>,
    block1: ResidualBlock<B>,
    down1: MaxPool2d,
    block2: ResidualBlock<B>,
    down2: MaxPool2d,
    projector: Conv2d<B>,
    head_pool: AdaptiveAvgPool2d,
    head_dropout: Dropout,
    head_hidden: Linear<B>,
    head_out: Linear<B>,
    activation: Relu,
}

#[derive(Config, Debug)]
pub struct ModelConfig {
    num_classes: usize,
    #[config(default = 192)]
    hidden_size: usize,
    #[config(default = 0.3)]
    dropout: f64,
}

impl ModelConfig {
    pub fn init<B: Backend>(&self, device: &B::Device) -> Model<B> {
        Model {
            stem: Conv2dConfig::new([1, 32], [5, 5]).init(device),
            block1: ResidualBlock::new(32, 32, 1, self.dropout, device),
            down1: MaxPool2dConfig::new([2, 2]).init(),
            block2: ResidualBlock::new(32, 64, 1, self.dropout, device),
            down2: MaxPool2dConfig::new([2, 2]).init(),
            projector: Conv2dConfig::new([64, 96], [1, 1]).init(device),
            head_pool: AdaptiveAvgPool2dConfig::new([1, 1]).init(),
            head_dropout: DropoutConfig::new(self.dropout).init(),
            head_hidden: LinearConfig::new(96, self.hidden_size).init(device),
            head_out: LinearConfig::new(self.hidden_size, self.num_classes).init(device),
            activation: Relu::new(),
        }
    }
}

#[derive(Module, Debug)]
struct ResidualBlock<B: Backend> {
    conv1: Conv2d<B>,
    conv2: Conv2d<B>,
    shortcut: Option<Conv2d<B>>,
    activation: Relu,
    dropout: Dropout,
}

impl<B: Backend> ResidualBlock<B> {
    fn new(
        in_channels: usize,
        out_channels: usize,
        stride: usize,
        dropout: f64,
        device: &B::Device,
    ) -> Self {
        let needs_projection = in_channels != out_channels || stride > 1;
        Self {
            conv1: Conv2dConfig::new([in_channels, out_channels], [3, 3])
                .with_stride([stride, stride])
                .with_padding(PaddingConfig2d::Same)
                .init(device),
            conv2: Conv2dConfig::new([out_channels, out_channels], [3, 3])
                .with_padding(PaddingConfig2d::Same)
                .init(device),
            shortcut: needs_projection.then(|| {
                Conv2dConfig::new([in_channels, out_channels], [1, 1])
                    .with_stride([stride, stride])
                    .init(device)
            }),
            activation: Relu::new(),
            dropout: DropoutConfig::new(dropout).init(),
        }
    }

    fn forward(&self, input: Tensor<B, 4>) -> Tensor<B, 4> {
        let residual = self
            .shortcut
            .as_ref()
            .map(|proj| proj.forward(input.clone()))
            .unwrap_or(input.clone());

        let x = self.activation.forward(self.conv1.forward(input));
        let x = self.dropout.forward(x);
        let x = self.conv2.forward(x);
        self.activation.forward(x + residual)
    }
}

impl<B: Backend> Model<B> {
    /// Forward for logits.
    pub fn forward(&self, images: Tensor<B, 4>) -> Tensor<B, 2> {
        let x = self.activation.forward(self.stem.forward(images));
        let x = self.block1.forward(x);
        let x = self.down1.forward(x);

        let x = self.block2.forward(x);
        let x = self.down2.forward(x);

        let x = self.activation.forward(self.projector.forward(x));
        let x = self.head_pool.forward(x);
        let batch = x.dims()[0];
        let x = x.reshape([batch, 96]);

        let x = self.head_dropout.forward(x);
        let x = self.activation.forward(self.head_hidden.forward(x));
        self.head_out.forward(x)
    }

    /// Convenience helper returning loss and predictions for classification.
    pub fn forward_classification(
        &self,
        images: Tensor<B, 4>,
        targets: Tensor<B, 1, Int>,
    ) -> ClassificationOutput<B> {
        let logits = self.forward(images);
        let loss = burn::nn::loss::CrossEntropyLossConfig::new()
            .init(&logits.device())
            .forward(logits.clone(), targets.clone());

        ClassificationOutput::new(loss, logits, targets)
    }
}
