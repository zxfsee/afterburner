use std::fmt::Write as _;
use std::io;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::model::{MODEL_ARCH_ID, MODEL_ARCH_VERSION};
use crate::preprocess::{MNIST_MEAN, MNIST_NORMALIZATION_NOTES, MNIST_STD};

pub const MANIFEST_FILENAME: &str = "manifest.toml";
pub const CURRENT_VERSION_FILENAME: &str = "current";
pub const DEFAULT_ARTIFACT_VERSION: &str = "0.1.0";
pub const INPUT_DTYPE: &str = "f32";
pub const INPUT_SHAPE: [usize; 3] = [1, 28, 28];
pub const WEIGHTS_DTYPE: &str = "f32";
pub const ACTIVATION_DTYPE: &str = "f32";
pub const QUANTIZATION_KIND_NONE: &str = "none";
pub const SIGNATURE_SCHEME_PLACEHOLDER: &str = "none";
pub const SIGNATURE_KEY_ID_PLACEHOLDER: &str = "unsigned";
pub const SIGNATURE_VALUE_PLACEHOLDER: &str = "";
pub const CANONICALIZATION_METHOD: &str = "toml-manifest-v1";
pub const CANONICALIZATION_NOTES: &str = "Canonical bytes use UTF-8 with LF line endings and the field order emitted by ArtifactManifest::to_toml_string().";

#[derive(Debug, Clone)]
pub struct ArtifactManifest {
    pub artifact: String,
    pub artifact_version: String,
    pub artifact_sha256: String,
    pub signature: ManifestSignature,
    pub canonicalization: ManifestCanonicalization,
    pub model: ManifestModel,
    pub input: ManifestInput,
    pub precision: ManifestPrecision,
    pub normalization: ManifestNormalization,
}

#[derive(Debug, Clone)]
pub struct ManifestModel {
    pub architecture_id: String,
    pub architecture_version: u32,
}

#[derive(Debug, Clone)]
pub struct ManifestInput {
    pub shape: [usize; 3],
    pub dtype: String,
}

#[derive(Debug, Clone)]
pub struct ManifestPrecision {
    pub weights_dtype: String,
    pub activation_dtype: String,
    pub quantization: String,
}

#[derive(Debug, Clone)]
pub struct ManifestSignature {
    pub scheme: String,
    pub key_id: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct ManifestCanonicalization {
    pub method: String,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct ManifestNormalization {
    pub dataset: String,
    pub mean: f32,
    pub std: f32,
    pub notes: String,
}

#[derive(Debug)]
pub enum ManifestError {
    Io(io::Error),
    TomlParse(toml::de::Error),
    MissingField(&'static str),
    InvalidField(&'static str, String),
    Mismatch {
        field: &'static str,
        expected: String,
        actual: String,
    },
}

impl ArtifactManifest {
    pub fn for_current(
        artifact_filename: &str,
        artifact_version: &str,
        artifact_sha256: String,
    ) -> Self {
        Self {
            artifact: artifact_filename.to_string(),
            artifact_version: artifact_version.to_string(),
            artifact_sha256,
            signature: ManifestSignature {
                scheme: SIGNATURE_SCHEME_PLACEHOLDER.to_string(),
                key_id: SIGNATURE_KEY_ID_PLACEHOLDER.to_string(),
                value: SIGNATURE_VALUE_PLACEHOLDER.to_string(),
            },
            canonicalization: ManifestCanonicalization {
                method: CANONICALIZATION_METHOD.to_string(),
                notes: CANONICALIZATION_NOTES.to_string(),
            },
            model: ManifestModel {
                architecture_id: MODEL_ARCH_ID.to_string(),
                architecture_version: MODEL_ARCH_VERSION,
            },
            input: ManifestInput {
                shape: INPUT_SHAPE,
                dtype: INPUT_DTYPE.to_string(),
            },
            precision: ManifestPrecision {
                weights_dtype: WEIGHTS_DTYPE.to_string(),
                activation_dtype: ACTIVATION_DTYPE.to_string(),
                quantization: QUANTIZATION_KIND_NONE.to_string(),
            },
            normalization: ManifestNormalization {
                dataset: "mnist".to_string(),
                mean: MNIST_MEAN,
                std: MNIST_STD,
                notes: MNIST_NORMALIZATION_NOTES.to_string(),
            },
        }
    }

    pub fn to_toml_string(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(&mut out, "artifact = \"{}\"", self.artifact);
        let _ = writeln!(&mut out, "artifact_version = \"{}\"", self.artifact_version);
        let _ = writeln!(&mut out, "artifact_sha256 = \"{}\"", self.artifact_sha256);
        let _ = writeln!(&mut out);
        let _ = writeln!(&mut out, "[signature]");
        let _ = writeln!(&mut out, "scheme = \"{}\"", self.signature.scheme);
        let _ = writeln!(&mut out, "key_id = \"{}\"", self.signature.key_id);
        let _ = writeln!(&mut out, "value = \"{}\"", self.signature.value);
        let _ = writeln!(&mut out);
        let _ = writeln!(&mut out, "[canonicalization]");
        let _ = writeln!(&mut out, "method = \"{}\"", self.canonicalization.method);
        let _ = writeln!(&mut out, "notes = \"{}\"", self.canonicalization.notes);
        let _ = writeln!(&mut out);
        let _ = writeln!(&mut out, "[model]");
        let _ = writeln!(
            &mut out,
            "architecture_id = \"{}\"",
            self.model.architecture_id
        );
        let _ = writeln!(
            &mut out,
            "architecture_version = {}",
            self.model.architecture_version
        );
        let _ = writeln!(&mut out);
        let _ = writeln!(&mut out, "[input]");
        let _ = writeln!(
            &mut out,
            "shape = [{}, {}, {}]",
            self.input.shape[0], self.input.shape[1], self.input.shape[2]
        );
        let _ = writeln!(&mut out, "dtype = \"{}\"", self.input.dtype);
        let _ = writeln!(&mut out);
        let _ = writeln!(&mut out, "[precision]");
        let _ = writeln!(
            &mut out,
            "weights_dtype = \"{}\"",
            self.precision.weights_dtype
        );
        let _ = writeln!(
            &mut out,
            "activation_dtype = \"{}\"",
            self.precision.activation_dtype
        );
        let _ = writeln!(
            &mut out,
            "quantization = \"{}\"",
            self.precision.quantization
        );
        let _ = writeln!(&mut out);
        let _ = writeln!(&mut out, "[normalization]");
        let _ = writeln!(&mut out, "dataset = \"{}\"", self.normalization.dataset);
        let _ = writeln!(&mut out, "mean = {}", self.normalization.mean);
        let _ = writeln!(&mut out, "std = {}", self.normalization.std);
        let _ = writeln!(&mut out, "notes = \"{}\"", self.normalization.notes);
        out
    }

    pub fn write_to_dir(&self, dir: &Path) -> io::Result<PathBuf> {
        let path = dir.join(MANIFEST_FILENAME);
        std::fs::write(&path, self.to_toml_string())?;
        Ok(path)
    }

    pub fn load_from_path(path: &Path) -> Result<Self, ManifestError> {
        let contents = std::fs::read_to_string(path).map_err(ManifestError::Io)?;
        let value: toml::Value = toml::from_str(&contents).map_err(ManifestError::TomlParse)?;
        parse_manifest_value(&value)
    }

    pub fn validate_against_current(&self, weights_path: &Path) -> Result<(), ManifestError> {
        let expected_artifact = weights_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or(ManifestError::InvalidField(
                "artifact",
                "invalid weights filename".to_string(),
            ))?;

        if self.artifact != expected_artifact {
            return Err(ManifestError::Mismatch {
                field: "artifact",
                expected: expected_artifact.to_string(),
                actual: self.artifact.clone(),
            });
        }

        if !is_semver(&self.artifact_version) {
            return Err(ManifestError::InvalidField(
                "artifact_version",
                "expected semver (MAJOR.MINOR.PATCH)".to_string(),
            ));
        }

        if let Some(expected_version) = infer_artifact_version(weights_path)
            && self.artifact_version != expected_version
        {
            return Err(ManifestError::Mismatch {
                field: "artifact_version",
                expected: expected_version,
                actual: self.artifact_version.clone(),
            });
        }

        let expected_sha256 = compute_sha256_hex(weights_path).map_err(ManifestError::Io)?;
        if self.artifact_sha256 != expected_sha256 {
            return Err(ManifestError::Mismatch {
                field: "artifact_sha256",
                expected: expected_sha256,
                actual: self.artifact_sha256.clone(),
            });
        }

        self.validate_signature()?;

        if self.canonicalization.method != CANONICALIZATION_METHOD {
            return Err(ManifestError::Mismatch {
                field: "canonicalization.method",
                expected: CANONICALIZATION_METHOD.to_string(),
                actual: self.canonicalization.method.clone(),
            });
        }

        if self.canonicalization.notes != CANONICALIZATION_NOTES {
            return Err(ManifestError::Mismatch {
                field: "canonicalization.notes",
                expected: CANONICALIZATION_NOTES.to_string(),
                actual: self.canonicalization.notes.clone(),
            });
        }

        if self.model.architecture_id != MODEL_ARCH_ID {
            return Err(ManifestError::Mismatch {
                field: "model.architecture_id",
                expected: MODEL_ARCH_ID.to_string(),
                actual: self.model.architecture_id.clone(),
            });
        }

        if self.model.architecture_version != MODEL_ARCH_VERSION {
            return Err(ManifestError::Mismatch {
                field: "model.architecture_version",
                expected: MODEL_ARCH_VERSION.to_string(),
                actual: self.model.architecture_version.to_string(),
            });
        }

        if self.input.shape != INPUT_SHAPE {
            return Err(ManifestError::Mismatch {
                field: "input.shape",
                expected: format!("{INPUT_SHAPE:?}"),
                actual: format!("{:?}", self.input.shape),
            });
        }

        if self.input.dtype != INPUT_DTYPE {
            return Err(ManifestError::Mismatch {
                field: "input.dtype",
                expected: INPUT_DTYPE.to_string(),
                actual: self.input.dtype.clone(),
            });
        }

        if self.precision.weights_dtype != WEIGHTS_DTYPE {
            return Err(ManifestError::InvalidField(
                "precision.weights_dtype",
                format!(
                    "unsupported weights dtype `{}`",
                    self.precision.weights_dtype
                ),
            ));
        }

        if self.precision.activation_dtype != ACTIVATION_DTYPE {
            return Err(ManifestError::InvalidField(
                "precision.activation_dtype",
                format!(
                    "unsupported activation dtype `{}`",
                    self.precision.activation_dtype
                ),
            ));
        }

        if self.precision.quantization != QUANTIZATION_KIND_NONE {
            return Err(ManifestError::InvalidField(
                "precision.quantization",
                format!(
                    "unsupported quantization `{}`; current runtime requires `{}`",
                    self.precision.quantization, QUANTIZATION_KIND_NONE
                ),
            ));
        }

        if self.normalization.mean != MNIST_MEAN {
            return Err(ManifestError::Mismatch {
                field: "normalization.mean",
                expected: MNIST_MEAN.to_string(),
                actual: self.normalization.mean.to_string(),
            });
        }

        if self.normalization.std != MNIST_STD {
            return Err(ManifestError::Mismatch {
                field: "normalization.std",
                expected: MNIST_STD.to_string(),
                actual: self.normalization.std.to_string(),
            });
        }

        if self.normalization.notes != MNIST_NORMALIZATION_NOTES {
            return Err(ManifestError::Mismatch {
                field: "normalization.notes",
                expected: MNIST_NORMALIZATION_NOTES.to_string(),
                actual: self.normalization.notes.clone(),
            });
        }

        if self.normalization.dataset != "mnist" {
            return Err(ManifestError::Mismatch {
                field: "normalization.dataset",
                expected: "mnist".to_string(),
                actual: self.normalization.dataset.clone(),
            });
        }

        Ok(())
    }

    fn validate_signature(&self) -> Result<(), ManifestError> {
        if self.signature.scheme == SIGNATURE_SCHEME_PLACEHOLDER {
            if self.signature.key_id != SIGNATURE_KEY_ID_PLACEHOLDER {
                return Err(ManifestError::Mismatch {
                    field: "signature.key_id",
                    expected: SIGNATURE_KEY_ID_PLACEHOLDER.to_string(),
                    actual: self.signature.key_id.clone(),
                });
            }

            if self.signature.value != SIGNATURE_VALUE_PLACEHOLDER {
                return Err(ManifestError::Mismatch {
                    field: "signature.value",
                    expected: SIGNATURE_VALUE_PLACEHOLDER.to_string(),
                    actual: self.signature.value.clone(),
                });
            }

            return Ok(());
        }

        if self.signature.key_id.trim().is_empty() {
            return Err(ManifestError::InvalidField(
                "signature.key_id",
                "must be non-empty when signature.scheme != \"none\"".to_string(),
            ));
        }

        if self.signature.value.trim().is_empty() {
            return Err(ManifestError::InvalidField(
                "signature.value",
                "must be non-empty when signature.scheme != \"none\"".to_string(),
            ));
        }

        let digest =
            self.signature
                .value
                .strip_prefix("sha256:")
                .ok_or(ManifestError::InvalidField(
                    "signature.value",
                    "expected format sha256:<hex-digest>".to_string(),
                ))?;

        if digest.len() != 64 || !digest.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ManifestError::InvalidField(
                "signature.value",
                "expected format sha256:<hex-digest>".to_string(),
            ));
        }

        let expected_digest = self.canonicalized_manifest_digest_hex();
        if digest.to_ascii_lowercase() != expected_digest {
            return Err(ManifestError::Mismatch {
                field: "signature.value.digest",
                expected: expected_digest,
                actual: digest.to_ascii_lowercase(),
            });
        }

        Ok(())
    }

    fn canonicalized_manifest_digest_input(&self) -> String {
        let mut signing_manifest = self.clone();
        signing_manifest.signature.value = SIGNATURE_VALUE_PLACEHOLDER.to_string();
        signing_manifest.to_toml_string()
    }

    fn canonicalized_manifest_digest_hex(&self) -> String {
        let input = self.canonicalized_manifest_digest_input();
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        hex_encode(&hasher.finalize())
    }
}

fn parse_manifest_value(value: &toml::Value) -> Result<ArtifactManifest, ManifestError> {
    let artifact = read_string(value, "artifact")?;
    let artifact_version = read_string(value, "artifact_version")?;
    let artifact_sha256 = read_string(value, "artifact_sha256")?;

    let (signature_scheme, signature_key_id, signature_value) =
        if let Some(signature) = value.get("signature") {
            (
                read_string(signature, "scheme")?,
                read_string(signature, "key_id")?,
                read_string(signature, "value")?,
            )
        } else {
            (
                SIGNATURE_SCHEME_PLACEHOLDER.to_string(),
                SIGNATURE_KEY_ID_PLACEHOLDER.to_string(),
                SIGNATURE_VALUE_PLACEHOLDER.to_string(),
            )
        };

    let (canonicalization_method, canonicalization_notes) =
        if let Some(canonicalization) = value.get("canonicalization") {
            (
                read_string(canonicalization, "method")?,
                read_string(canonicalization, "notes")?,
            )
        } else {
            (
                CANONICALIZATION_METHOD.to_string(),
                CANONICALIZATION_NOTES.to_string(),
            )
        };

    let model = value
        .get("model")
        .ok_or(ManifestError::MissingField("model"))?;
    let architecture_id = read_string(model, "architecture_id")?;
    let architecture_version = read_u32(model, "architecture_version")?;

    let input = value
        .get("input")
        .ok_or(ManifestError::MissingField("input"))?;
    let shape = read_shape(input, "shape")?;
    let dtype = read_string(input, "dtype")?;

    let precision = value.get("precision");
    let weights_dtype = precision
        .map(|precision| read_string(precision, "weights_dtype"))
        .transpose()?
        .unwrap_or_else(|| WEIGHTS_DTYPE.to_string());
    let activation_dtype = precision
        .map(|precision| read_string(precision, "activation_dtype"))
        .transpose()?
        .unwrap_or_else(|| ACTIVATION_DTYPE.to_string());
    let quantization = precision
        .map(|precision| read_string(precision, "quantization"))
        .transpose()?
        .unwrap_or_else(|| QUANTIZATION_KIND_NONE.to_string());

    let normalization = value
        .get("normalization")
        .ok_or(ManifestError::MissingField("normalization"))?;
    let dataset = read_string(normalization, "dataset")?;
    let mean = read_f32(normalization, "mean")?;
    let std = read_f32(normalization, "std")?;
    let notes = read_string(normalization, "notes")?;

    Ok(ArtifactManifest {
        artifact,
        artifact_version,
        artifact_sha256,
        signature: ManifestSignature {
            scheme: signature_scheme,
            key_id: signature_key_id,
            value: signature_value,
        },
        canonicalization: ManifestCanonicalization {
            method: canonicalization_method,
            notes: canonicalization_notes,
        },
        model: ManifestModel {
            architecture_id,
            architecture_version,
        },
        input: ManifestInput { shape, dtype },
        precision: ManifestPrecision {
            weights_dtype,
            activation_dtype,
            quantization,
        },
        normalization: ManifestNormalization {
            dataset,
            mean,
            std,
            notes,
        },
    })
}

pub fn compute_sha256_hex(path: &Path) -> io::Result<String> {
    let bytes = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    Ok(hex_encode(&digest))
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut out, "{:02x}", byte);
    }
    out
}

fn read_string(value: &toml::Value, field: &'static str) -> Result<String, ManifestError> {
    match value.get(field) {
        Some(v) => v
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| ManifestError::InvalidField(field, "expected string".to_string())),
        None => Err(ManifestError::MissingField(field)),
    }
}

fn read_u32(value: &toml::Value, field: &'static str) -> Result<u32, ManifestError> {
    match value.get(field) {
        Some(v) => v
            .as_integer()
            .and_then(|v| u32::try_from(v).ok())
            .ok_or_else(|| ManifestError::InvalidField(field, "expected u32".to_string())),
        None => Err(ManifestError::MissingField(field)),
    }
}

fn read_f32(value: &toml::Value, field: &'static str) -> Result<f32, ManifestError> {
    match value.get(field) {
        Some(v) => v
            .as_float()
            .or_else(|| v.as_integer().map(|i| i as f64))
            .map(|v| v as f32)
            .ok_or_else(|| ManifestError::InvalidField(field, "expected float".to_string())),
        None => Err(ManifestError::MissingField(field)),
    }
}

fn read_shape(value: &toml::Value, field: &'static str) -> Result<[usize; 3], ManifestError> {
    let arr = match value.get(field) {
        Some(v) => v
            .as_array()
            .ok_or_else(|| ManifestError::InvalidField(field, "expected array".to_string()))?,
        None => return Err(ManifestError::MissingField(field)),
    };
    if arr.len() != 3 {
        return Err(ManifestError::InvalidField(
            field,
            "expected 3 items".to_string(),
        ));
    }
    let mut out = [0usize; 3];
    for (idx, item) in arr.iter().enumerate() {
        let v = item
            .as_integer()
            .and_then(|v| usize::try_from(v).ok())
            .ok_or_else(|| ManifestError::InvalidField(field, "expected usize".to_string()))?;
        out[idx] = v;
    }
    Ok(out)
}

pub fn is_semver(version: &str) -> bool {
    let mut parts = version.split('.');
    let (Some(major), Some(minor), Some(patch), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    [major, minor, patch]
        .iter()
        .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
}

fn infer_artifact_version(weights_path: &Path) -> Option<String> {
    let parent = weights_path.parent()?;
    let version = parent.file_name()?.to_str()?;
    if !is_semver(version) {
        return None;
    }
    let root = parent.parent()?;
    if root.file_name()?.to_str()? != "inference" {
        return None;
    }
    Some(version.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        ACTIVATION_DTYPE, CANONICALIZATION_METHOD, CANONICALIZATION_NOTES, ManifestError,
        QUANTIZATION_KIND_NONE, SIGNATURE_KEY_ID_PLACEHOLDER, SIGNATURE_SCHEME_PLACEHOLDER,
        WEIGHTS_DTYPE, parse_manifest_value,
    };

    #[test]
    fn parse_manifest_backfills_signing_fields_when_absent() {
        let manifest = r#"
artifact = "model.mpk"
artifact_version = "0.1.0"
artifact_sha256 = "abc"

[model]
architecture_id = "afterburner.mnist.residual_v1"
architecture_version = 1

[input]
shape = [1, 28, 28]
dtype = "f32"

[normalization]
dataset = "mnist"
mean = 0.1307
std = 0.3081
notes = "((x / 255.0) - 0.1307) / 0.3081"
"#;
        let value: toml::Value = toml::from_str(manifest).expect("valid toml");
        let parsed = parse_manifest_value(&value).expect("parse manifest");
        assert_eq!(parsed.signature.scheme, SIGNATURE_SCHEME_PLACEHOLDER);
        assert_eq!(parsed.signature.key_id, SIGNATURE_KEY_ID_PLACEHOLDER);
        assert_eq!(parsed.canonicalization.method, CANONICALIZATION_METHOD);
        assert_eq!(parsed.canonicalization.notes, CANONICALIZATION_NOTES);
        assert_eq!(parsed.precision.weights_dtype, WEIGHTS_DTYPE);
        assert_eq!(parsed.precision.activation_dtype, ACTIVATION_DTYPE);
        assert_eq!(parsed.precision.quantization, QUANTIZATION_KIND_NONE);
    }

    #[test]
    fn signed_manifest_requires_non_empty_key_id() {
        let manifest = r#"
artifact = "model.mpk"
artifact_version = "0.1.0"
artifact_sha256 = "abc"

[signature]
scheme = "ed25519"
key_id = ""
value = "sha256:8f2f4d6f8f2f4d6f8f2f4d6f8f2f4d6f8f2f4d6f8f2f4d6f8f2f4d6f8f2f4d6f"

[canonicalization]
method = "toml-manifest-v1"
notes = "Canonical bytes use UTF-8 with LF line endings and the field order emitted by ArtifactManifest::to_toml_string()."

[model]
architecture_id = "afterburner.mnist.residual_v1"
architecture_version = 1

[input]
shape = [1, 28, 28]
dtype = "f32"

[normalization]
dataset = "mnist"
mean = 0.1307
std = 0.3081
notes = "((x / 255.0) - 0.1307) / 0.3081"
"#;
        let value: toml::Value = toml::from_str(manifest).expect("valid toml");
        let parsed = parse_manifest_value(&value).expect("parse manifest");
        let err = parsed
            .validate_signature()
            .expect_err("empty key id must fail");
        match err {
            ManifestError::InvalidField(field, detail) => {
                assert_eq!(field, "signature.key_id");
                assert!(detail.contains("must be non-empty"));
            }
            _ => panic!("expected invalid field error for signature.key_id"),
        }
    }

    #[test]
    fn signed_manifest_digest_must_match_canonical_input() {
        let manifest = r#"
artifact = "model.mpk"
artifact_version = "0.1.0"
artifact_sha256 = "abc"

[signature]
scheme = "ed25519"
key_id = "unit-test-key"
value = "sha256:0000000000000000000000000000000000000000000000000000000000000000"

[canonicalization]
method = "toml-manifest-v1"
notes = "Canonical bytes use UTF-8 with LF line endings and the field order emitted by ArtifactManifest::to_toml_string()."

[model]
architecture_id = "afterburner.mnist.residual_v1"
architecture_version = 1

[input]
shape = [1, 28, 28]
dtype = "f32"

[normalization]
dataset = "mnist"
mean = 0.1307
std = 0.3081
notes = "((x / 255.0) - 0.1307) / 0.3081"
"#;
        let value: toml::Value = toml::from_str(manifest).expect("valid toml");
        let mut parsed = parse_manifest_value(&value).expect("parse manifest");

        let err = parsed
            .validate_signature()
            .expect_err("digest mismatch must fail");
        match err {
            ManifestError::Mismatch {
                field,
                expected,
                actual,
            } => {
                assert_eq!(field, "signature.value.digest");
                assert_eq!(
                    actual,
                    "0000000000000000000000000000000000000000000000000000000000000000"
                );
                assert_eq!(expected.len(), 64);
            }
            _ => panic!("expected mismatch for signature.value.digest"),
        }

        let digest = parsed.canonicalized_manifest_digest_hex();
        parsed.signature.value = format!("sha256:{digest}");
        parsed
            .validate_signature()
            .expect("matching canonical digest should pass");
    }
}
