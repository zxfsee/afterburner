use std::fs;
use std::io::BufWriter;
use std::path::PathBuf;

use png::{BitDepth, ColorType, Encoder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::current_dir()?;
    let fixtures_dir = root.join("fixtures");
    fs::create_dir_all(&fixtures_dir)?;

    let image = build_fixture_image();
    write_png(&fixtures_dir.join("mnist_0.png"), &image)?;
    write_summary(&fixtures_dir.join("mnist_0.summary.toml"), &image)?;
    let weights_path = fixtures_dir.join("model.mpk");
    write_fixture_weights(&weights_path)?;
    let checksum = afterburner::manifest::compute_sha256_hex(&weights_path)?;
    write_manifest(&fixtures_dir.join("manifest.toml"), &checksum)?;

    Ok(())
}

fn build_fixture_image() -> [[f32; 28]; 28] {
    let mut image = [[0.0f32; 28]; 28];
    for (y, row) in image.iter_mut().enumerate() {
        for (x, cell) in row.iter_mut().enumerate() {
            let value = ((x * 9 + y * 7) % 256) as f32;
            *cell = value;
        }
    }
    image
}

fn write_png(path: &PathBuf, image: &[[f32; 28]; 28]) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::create(path)?;
    let w = BufWriter::new(file);
    let mut encoder = Encoder::new(w, 28, 28);
    encoder.set_color(ColorType::Grayscale);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder.write_header()?;

    let mut buf = Vec::with_capacity(28 * 28);
    for row in image {
        for &value in row {
            let clamped = value.round().clamp(0.0, 255.0) as u8;
            buf.push(clamped);
        }
    }
    writer.write_image_data(&buf)?;
    Ok(())
}

fn write_summary(
    path: &PathBuf,
    image: &[[f32; 28]; 28],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut values = Vec::with_capacity(28 * 28);
    for row in image {
        for &value in row {
            let normalized = ((value / 255.0) - afterburner::preprocess::MNIST_MEAN)
                / afterburner::preprocess::MNIST_STD;
            values.push(normalized);
        }
    }

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
    let min = values.iter().copied().reduce(f32::min).unwrap_or(0.0);
    let max = values.iter().copied().reduce(f32::max).unwrap_or(0.0);

    let summary = format!(
        "mean = {:.8}\nstd = {:.8}\nmin = {:.8}\nmax = {:.8}\n",
        mean, std, min, max
    );
    fs::write(path, summary)?;
    Ok(())
}

fn write_fixture_weights(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(path, b"fixture-model-weights")?;
    Ok(())
}

fn write_manifest(path: &PathBuf, checksum: &str) -> Result<(), Box<dyn std::error::Error>> {
    let version = "0.1.0";
    let template = format!(
        "artifact = \"model.mpk\"\n\
artifact_version = \"{}\"\n\
artifact_sha256 = \"{}\"\n\
\n\
[signature]\n\
scheme = \"{}\"\n\
key_id = \"{}\"\n\
value = \"{}\"\n\
\n\
[canonicalization]\n\
method = \"{}\"\n\
notes = \"{}\"\n\
\n\
[model]\n\
architecture_id = \"{}\"\n\
architecture_version = {}\n\
\n\
[input]\n\
shape = [1, 28, 28]\n\
dtype = \"f32\"\n\
\n\
[precision]\n\
weights_dtype = \"f32\"\n\
activation_dtype = \"f32\"\n\
quantization = \"none\"\n\
\n\
[normalization]\n\
dataset = \"mnist\"\n\
mean = {}\n\
std = {}\n\
notes = \"{}\"\n",
        version,
        checksum,
        afterburner::manifest::SIGNATURE_SCHEME_PLACEHOLDER,
        afterburner::manifest::SIGNATURE_KEY_ID_PLACEHOLDER,
        afterburner::manifest::SIGNATURE_VALUE_PLACEHOLDER,
        afterburner::manifest::CANONICALIZATION_METHOD,
        afterburner::manifest::CANONICALIZATION_NOTES,
        afterburner::model::MODEL_ARCH_ID,
        afterburner::model::MODEL_ARCH_VERSION,
        afterburner::preprocess::MNIST_MEAN,
        afterburner::preprocess::MNIST_STD,
        afterburner::preprocess::MNIST_NORMALIZATION_NOTES
    );
    fs::write(path, template)?;
    Ok(())
}
