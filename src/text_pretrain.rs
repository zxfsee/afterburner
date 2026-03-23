use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use burn::{
    config::Config,
    data::{
        dataloader::{DataLoader, DataLoaderBuilder, batcher::Batcher},
        dataset::Dataset,
    },
    nn::{
        Embedding, EmbeddingConfig, Linear, LinearConfig, PositionalEncoding,
        PositionalEncodingConfig,
        attention::generate_autoregressive_mask,
        loss::CrossEntropyLossConfig,
        transformer::{TransformerEncoder, TransformerEncoderConfig, TransformerEncoderInput},
    },
    optim::AdamConfig,
    prelude::*,
    record::CompactRecorder,
    tensor::{TensorData, backend::AutodiffBackend},
    train::{
        ClassificationOutput, InferenceStep, Learner, SupervisedTraining, TrainOutput, TrainStep,
        metric::{LossMetric, PerplexityMetric},
    },
};
use serde_json::{Value, json};

use crate::{
    data::parse_pretraining_dataset_manifest,
    observability::{append_json_line, emit_event, event_line},
};

pub const TEXT_TOKEN_CACHE_SCHEMA_VERSION: &str = "1";
pub const TEXT_PRETRAINING_RUN_SCHEMA_VERSION: &str = "1";
pub const TEXT_PRETRAINING_RUN_FILENAME: &str = "text_pretraining_run.json";
pub const TEXT_INFERENCE_PROFILE_FILENAME: &str = "text_inference_profile.json";

#[derive(Debug, Clone, PartialEq)]
pub struct TextTrainingConfig {
    pub batch_size: usize,
    pub num_epochs: usize,
    pub learning_rate: f64,
    pub num_workers: usize,
    pub seed: u64,
    pub artifacts_dir: String,
    pub artifact_version: String,
    pub dataset_manifest_path: PathBuf,
    pub tokenizer_profile_path: PathBuf,
    pub token_cache_path: PathBuf,
    pub resume_epoch: Option<usize>,
}

#[derive(Debug)]
pub enum TextTrainingError {
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for TextTrainingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Parse(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for TextTrainingError {}

impl From<std::io::Error> for TextTrainingError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TextTokenizerProfile {
    tokenizer_id: String,
    tokenizer_revision: String,
    vocab_size: usize,
    context_length: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TextTokenCache {
    source: String,
    source_revision: String,
    tokenizer_id: String,
    tokenizer_revision: String,
    context_length: usize,
    vocab_size: usize,
    train_sequences: Vec<Vec<i64>>,
    validation_sequences: Vec<Vec<i64>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TextSequenceItem {
    tokens: Vec<i64>,
}

#[derive(Clone, Debug)]
struct TextSequenceDataset {
    items: Vec<TextSequenceItem>,
}

impl Dataset<TextSequenceItem> for TextSequenceDataset {
    fn get(&self, index: usize) -> Option<TextSequenceItem> {
        self.items.get(index).cloned()
    }

    fn len(&self) -> usize {
        self.items.len()
    }
}

#[derive(Clone, Debug)]
struct TextBatcher {
    sequence_length: usize,
}

#[derive(Clone, Debug)]
pub struct TextBatch<B: Backend> {
    pub input_tokens: Tensor<B, 2, Int>,
    pub targets: Tensor<B, 1, Int>,
}

impl<B: Backend> Batcher<B, TextSequenceItem, TextBatch<B>> for TextBatcher {
    fn batch(&self, items: Vec<TextSequenceItem>, device: &B::Device) -> TextBatch<B> {
        let batch_size = items.len();
        let mut input_tokens = Vec::with_capacity(batch_size * self.sequence_length);
        let mut targets = Vec::with_capacity(batch_size * self.sequence_length);

        for item in items {
            let input = &item.tokens[..self.sequence_length];
            let target = &item.tokens[1..];
            input_tokens.extend_from_slice(input);
            targets.extend_from_slice(target);
        }

        let input_tokens = Tensor::<B, 2, Int>::from_data(
            TensorData::new(input_tokens, [batch_size, self.sequence_length]),
            device,
        );
        let targets = Tensor::<B, 1, Int>::from_data(
            TensorData::new(targets, [batch_size * self.sequence_length]),
            device,
        );

        TextBatch {
            input_tokens,
            targets,
        }
    }
}

#[derive(Config, Debug)]
pub struct TextModelConfig {
    pub vocab_size: usize,
    pub context_length: usize,
    #[config(default = 32)]
    pub d_model: usize,
    #[config(default = 64)]
    pub d_ff: usize,
    #[config(default = 4)]
    pub n_heads: usize,
    #[config(default = 2)]
    pub n_layers: usize,
    #[config(default = 0.1)]
    pub dropout: f64,
}

#[derive(Module, Debug)]
pub struct TextModel<B: Backend> {
    token_embedding: Embedding<B>,
    position_encoding: PositionalEncoding<B>,
    transformer: TransformerEncoder<B>,
    lm_head: Linear<B>,
}

impl TextModelConfig {
    pub fn init<B: Backend>(&self, device: &B::Device) -> TextModel<B> {
        TextModel {
            token_embedding: EmbeddingConfig::new(self.vocab_size, self.d_model).init(device),
            position_encoding: PositionalEncodingConfig::new(self.d_model)
                .with_max_sequence_size(self.context_length)
                .init(device),
            transformer: TransformerEncoderConfig::new(
                self.d_model,
                self.d_ff,
                self.n_heads,
                self.n_layers,
            )
            .with_dropout(self.dropout)
            .init(device),
            lm_head: LinearConfig::new(self.d_model, self.vocab_size).init(device),
        }
    }
}

impl<B: Backend> TextModel<B> {
    fn forward(&self, input_tokens: Tensor<B, 2, Int>) -> Tensor<B, 3> {
        let x = self.token_embedding.forward(input_tokens);
        let x = self.position_encoding.forward(x);
        let [batch_size, seq_length, _] = x.dims();
        let mask_attn = generate_autoregressive_mask(batch_size, seq_length, &x.device());
        let x = self
            .transformer
            .forward(TransformerEncoderInput::new(x).mask_attn(mask_attn));
        self.lm_head.forward(x)
    }

    fn forward_classification(
        &self,
        input_tokens: Tensor<B, 2, Int>,
        targets: Tensor<B, 1, Int>,
    ) -> ClassificationOutput<B> {
        let logits = self.forward(input_tokens);
        let [batch_size, seq_length, vocab_size] = logits.dims();
        let logits = logits.reshape([batch_size * seq_length, vocab_size]);
        let loss = CrossEntropyLossConfig::new()
            .init(&logits.device())
            .forward(logits.clone(), targets.clone());

        ClassificationOutput::new(loss, logits, targets)
    }
}

impl<B: AutodiffBackend> TrainStep for TextModel<B> {
    type Input = TextBatch<B>;
    type Output = ClassificationOutput<B>;

    fn step(&self, batch: Self::Input) -> TrainOutput<Self::Output> {
        let item = self.forward_classification(batch.input_tokens, batch.targets);
        let grads = item.loss.backward();
        TrainOutput::new(self, grads, item)
    }
}

impl<B: Backend> InferenceStep for TextModel<B> {
    type Input = TextBatch<B>;
    type Output = ClassificationOutput<B>;

    fn step(&self, batch: Self::Input) -> Self::Output {
        self.forward_classification(batch.input_tokens, batch.targets)
    }
}

pub fn train_text<B: AutodiffBackend>(
    config: TextTrainingConfig,
    device: B::Device,
) -> Result<(), TextTrainingError> {
    B::seed(&device, config.seed);

    let dataset_manifest = load_dataset_manifest(&config.dataset_manifest_path)?;
    let tokenizer_profile = load_tokenizer_profile(&config.tokenizer_profile_path)?;
    let token_cache = load_token_cache(&config.token_cache_path)?;

    validate_token_cache(
        &token_cache,
        &dataset_manifest,
        &tokenizer_profile,
        &config.token_cache_path,
    )?;

    let root = Path::new(&config.artifacts_dir);
    let train_dir = root.join("train").join("text");
    let inference_dir = root.join("text_inference").join(&config.artifact_version);
    fs::create_dir_all(&train_dir)?;
    fs::create_dir_all(&inference_dir)?;

    let sequence_length = token_cache.context_length;
    let train_loader = text_loader::<B>(
        token_cache.train_sequences.clone(),
        config.batch_size,
        config.num_workers,
        config.seed,
        sequence_length,
        device.clone(),
        true,
    );
    let valid_loader = text_loader::<B::InnerBackend>(
        token_cache.validation_sequences.clone(),
        config.batch_size,
        config.num_workers,
        config.seed,
        sequence_length,
        device.clone(),
        false,
    );

    let optim = AdamConfig::new().init::<B, TextModel<B>>();
    let learner = Learner::new(
        TextModelConfig::new(token_cache.vocab_size, token_cache.context_length).init::<B>(&device),
        optim,
        config.learning_rate,
    );

    let mut training = SupervisedTraining::new(&train_dir, train_loader, valid_loader)
        .metric_train_numeric(LossMetric::new())
        .metric_train_numeric(PerplexityMetric::new())
        .metric_valid_numeric(LossMetric::new())
        .metric_valid_numeric(PerplexityMetric::new())
        .with_file_checkpointer(CompactRecorder::new())
        .num_epochs(config.num_epochs);
    if let Some(epoch) = config.resume_epoch {
        training = training.checkpoint(epoch);
    }

    let result = training.launch(learner);

    let text_model_path = inference_dir.join("model.mpk");
    result
        .model
        .save_file(&text_model_path, &CompactRecorder::new())
        .map_err(|err| TextTrainingError::Parse(format!("save text model: {err}")))?;

    let text_inference_profile = text_inference_profile_value(&config.tokenizer_profile_path);
    let text_inference_profile_path = inference_dir.join(TEXT_INFERENCE_PROFILE_FILENAME);
    write_json_file(&text_inference_profile_path, &text_inference_profile)?;

    let run_artifact = text_pretraining_run_value(
        &config,
        &token_cache,
        &train_dir,
        &text_model_path,
        &text_inference_profile_path,
    );
    let run_path = root.join("train").join(TEXT_PRETRAINING_RUN_FILENAME);
    write_json_file(&run_path, &run_artifact)?;
    write_text_train_done_event(&train_dir, &run_artifact)?;

    Ok(())
}

fn text_loader<B: Backend>(
    sequences: Vec<Vec<i64>>,
    batch_size: usize,
    num_workers: usize,
    seed: u64,
    sequence_length: usize,
    device: B::Device,
    shuffle: bool,
) -> Arc<dyn DataLoader<B, TextBatch<B>>> {
    let dataset = Arc::new(TextSequenceDataset {
        items: sequences
            .into_iter()
            .map(|tokens| TextSequenceItem { tokens })
            .collect(),
    });
    let batcher = TextBatcher { sequence_length };
    let builder = DataLoaderBuilder::new(batcher)
        .batch_size(batch_size)
        .num_workers(num_workers)
        .set_device(device);
    if shuffle {
        builder.shuffle(seed).build(dataset)
    } else {
        builder.build(dataset)
    }
}

fn load_dataset_manifest(
    path: &Path,
) -> Result<crate::data::PretrainingDatasetManifest, TextTrainingError> {
    let text = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&text).map_err(|err| {
        TextTrainingError::Parse(format!(
            "parse dataset manifest at {}: {err}",
            path.display()
        ))
    })?;
    parse_pretraining_dataset_manifest(&value).map_err(|err| {
        TextTrainingError::Parse(format!(
            "validate dataset manifest at {}: {err:?}",
            path.display()
        ))
    })
}

fn load_tokenizer_profile(path: &Path) -> Result<TextTokenizerProfile, TextTrainingError> {
    let value = load_json_value(path)?;
    let object = value.as_object().ok_or_else(|| {
        TextTrainingError::Parse(format!(
            "tokenizer profile at {} must be an object",
            path.display()
        ))
    })?;

    let schema_version = required_string(object, "schema_version", path)?;
    if schema_version != "1" {
        return Err(TextTrainingError::Parse(format!(
            "tokenizer profile at {} must have schema_version=`1`",
            path.display()
        )));
    }

    Ok(TextTokenizerProfile {
        tokenizer_id: required_string(object, "tokenizer_id", path)?,
        tokenizer_revision: required_string(object, "tokenizer_revision", path)?,
        vocab_size: required_u64(object, "vocab_size", path)? as usize,
        context_length: required_u64(object, "context_length", path)? as usize,
    })
}

fn load_token_cache(path: &Path) -> Result<TextTokenCache, TextTrainingError> {
    let value = load_json_value(path)?;
    let object = value.as_object().ok_or_else(|| {
        TextTrainingError::Parse(format!(
            "text token cache at {} must be an object",
            path.display()
        ))
    })?;

    let schema_version = required_string(object, "schema_version", path)?;
    if schema_version != TEXT_TOKEN_CACHE_SCHEMA_VERSION {
        return Err(TextTrainingError::Parse(format!(
            "text token cache at {} must have schema_version=`{TEXT_TOKEN_CACHE_SCHEMA_VERSION}`",
            path.display()
        )));
    }

    Ok(TextTokenCache {
        source: required_string(object, "source", path)?,
        source_revision: required_string(object, "source_revision", path)?,
        tokenizer_id: required_string(object, "tokenizer_id", path)?,
        tokenizer_revision: required_string(object, "tokenizer_revision", path)?,
        context_length: required_u64(object, "context_length", path)? as usize,
        vocab_size: required_u64(object, "vocab_size", path)? as usize,
        train_sequences: required_sequences(object, "train_sequences", path)?,
        validation_sequences: required_sequences(object, "validation_sequences", path)?,
    })
}

fn validate_token_cache(
    token_cache: &TextTokenCache,
    dataset_manifest: &crate::data::PretrainingDatasetManifest,
    tokenizer_profile: &TextTokenizerProfile,
    path: &Path,
) -> Result<(), TextTrainingError> {
    if token_cache.source != dataset_manifest.source
        || token_cache.source_revision != dataset_manifest.source_revision
    {
        return Err(TextTrainingError::Parse(format!(
            "text token cache at {} does not match dataset manifest source identity",
            path.display()
        )));
    }

    if token_cache.tokenizer_id != tokenizer_profile.tokenizer_id
        || token_cache.tokenizer_revision != tokenizer_profile.tokenizer_revision
    {
        return Err(TextTrainingError::Parse(format!(
            "text token cache at {} does not match tokenizer profile identity",
            path.display()
        )));
    }

    if token_cache.context_length > tokenizer_profile.context_length {
        return Err(TextTrainingError::Parse(format!(
            "text token cache at {} exceeds tokenizer profile context_length",
            path.display()
        )));
    }

    if token_cache.vocab_size > tokenizer_profile.vocab_size {
        return Err(TextTrainingError::Parse(format!(
            "text token cache at {} exceeds tokenizer profile vocab_size",
            path.display()
        )));
    }

    for (label, sequences) in [
        ("train_sequences", token_cache.train_sequences.as_slice()),
        (
            "validation_sequences",
            token_cache.validation_sequences.as_slice(),
        ),
    ] {
        for (index, sequence) in sequences.iter().enumerate() {
            if sequence.len() != token_cache.context_length + 1 {
                return Err(TextTrainingError::Parse(format!(
                    "text token cache at {} has {label}[{index}] length {} but expected {}",
                    path.display(),
                    sequence.len(),
                    token_cache.context_length + 1
                )));
            }
            if sequence
                .iter()
                .any(|token| *token < 0 || (*token as usize) >= token_cache.vocab_size)
            {
                return Err(TextTrainingError::Parse(format!(
                    "text token cache at {} has out-of-range token ids in {label}[{index}]",
                    path.display()
                )));
            }
        }
    }

    Ok(())
}

fn text_inference_profile_value(tokenizer_profile_path: &Path) -> Value {
    json!({
        "schema_version": "1",
        "task": "causal-lm",
        "tokenizer_profile": tokenizer_profile_path.display().to_string(),
        "max_context_tokens": 1024,
        "sampling_defaults": {
            "max_new_tokens": 128,
            "temperature": 0.8,
            "top_k": 40,
            "top_p": 0.95
        }
    })
}

fn text_pretraining_run_value(
    config: &TextTrainingConfig,
    token_cache: &TextTokenCache,
    train_dir: &Path,
    text_model_path: &Path,
    text_inference_profile_path: &Path,
) -> Value {
    json!({
        "schema_version": TEXT_PRETRAINING_RUN_SCHEMA_VERSION,
        "task": "causal-lm",
        "artifact_version": config.artifact_version,
        "backend": std::env::var("BACKEND").unwrap_or_else(|_| "wgpu".to_string()).to_ascii_lowercase(),
        "dataset_manifest": config.dataset_manifest_path.display().to_string(),
        "tokenizer_profile": config.tokenizer_profile_path.display().to_string(),
        "token_cache": config.token_cache_path.display().to_string(),
        "source": token_cache.source,
        "source_revision": token_cache.source_revision,
        "context_length": token_cache.context_length,
        "vocab_size": token_cache.vocab_size,
        "train_sequence_count": token_cache.train_sequences.len(),
        "validation_sequence_count": token_cache.validation_sequences.len(),
        "num_epochs": config.num_epochs,
        "batch_size": config.batch_size,
        "checkpoint_root": train_dir.join("checkpoint").display().to_string(),
        "resume_epoch": config.resume_epoch,
        "text_model": text_model_path.display().to_string(),
        "text_inference_profile": text_inference_profile_path.display().to_string()
    })
}

fn write_text_train_done_event(
    train_dir: &Path,
    run_artifact: &Value,
) -> Result<(), TextTrainingError> {
    let path = train_dir.join("observability.jsonl");
    let line = event_line("info", "train", "text_train_done", run_artifact.clone());
    append_json_line(&path, &line)?;
    emit_event("info", "train", "text_train_done", run_artifact.clone());
    Ok(())
}

fn write_json_file(path: &Path, value: &Value) -> Result<(), TextTrainingError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(value).map_err(|err| {
        TextTrainingError::Parse(format!("serialize json for {}: {err}", path.display()))
    })?;
    fs::write(path, text)?;
    Ok(())
}

fn load_json_value(path: &Path) -> Result<Value, TextTrainingError> {
    let text = fs::read_to_string(path)?;
    serde_json::from_str(&text)
        .map_err(|err| TextTrainingError::Parse(format!("parse json at {}: {err}", path.display())))
}

fn required_string(
    object: &serde_json::Map<String, Value>,
    key: &str,
    path: &Path,
) -> Result<String, TextTrainingError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            TextTrainingError::Parse(format!(
                "required string field `{key}` missing in {}",
                path.display()
            ))
        })
}

fn required_u64(
    object: &serde_json::Map<String, Value>,
    key: &str,
    path: &Path,
) -> Result<u64, TextTrainingError> {
    object.get(key).and_then(Value::as_u64).ok_or_else(|| {
        TextTrainingError::Parse(format!(
            "required integer field `{key}` missing in {}",
            path.display()
        ))
    })
}

fn required_sequences(
    object: &serde_json::Map<String, Value>,
    key: &str,
    path: &Path,
) -> Result<Vec<Vec<i64>>, TextTrainingError> {
    let values = object.get(key).and_then(Value::as_array).ok_or_else(|| {
        TextTrainingError::Parse(format!(
            "required sequence array `{key}` missing in {}",
            path.display()
        ))
    })?;

    let mut sequences = Vec::with_capacity(values.len());
    for (index, value) in values.iter().enumerate() {
        let sequence = value.as_array().ok_or_else(|| {
            TextTrainingError::Parse(format!(
                "{key}[{index}] in {} must be an array",
                path.display()
            ))
        })?;
        if sequence.len() < 2 {
            return Err(TextTrainingError::Parse(format!(
                "{key}[{index}] in {} must contain at least two tokens",
                path.display()
            )));
        }
        let mut tokens = Vec::with_capacity(sequence.len());
        for token in sequence {
            let token = token.as_u64().ok_or_else(|| {
                TextTrainingError::Parse(format!(
                    "{key}[{index}] in {} must contain only non-negative integers",
                    path.display()
                ))
            })?;
            tokens.push(i64::try_from(token).map_err(|_| {
                TextTrainingError::Parse(format!(
                    "{key}[{index}] in {} contains token id too large for i64",
                    path.display()
                ))
            })?);
        }
        sequences.push(tokens);
    }

    Ok(sequences)
}
