use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use afterburner::infer::manifest_path_for_weights;
use afterburner::manifest::ArtifactManifest;
use serde_json::json;

struct ProfilingSummary {
    schema_version: &'static str,
    profile_kind: &'static str,
    profiler: &'static str,
    weights_artifact: String,
    artifact_version: String,
    backend: String,
    profile_command: String,
    flamegraph_svg: String,
    total_samples: u64,
    top_hotspots: Vec<Hotspot>,
}

#[derive(Debug, Clone)]
struct Hotspot {
    symbol: String,
    samples: u64,
    percent: f64,
}

struct Args {
    input: PathBuf,
    output: PathBuf,
    weights_artifact: PathBuf,
    backend: String,
    profile_command: String,
}

fn main() -> Result<(), String> {
    let args = parse_args(std::env::args().skip(1))?;
    let svg = fs::read_to_string(&args.input)
        .map_err(|err| format!("read flamegraph svg `{}`: {err}", args.input.display()))?;
    let manifest_path = manifest_path_for_weights(&args.weights_artifact);
    let manifest = ArtifactManifest::load_from_path(&manifest_path).map_err(|err| {
        format!(
            "load artifact manifest `{}`: {err:?}",
            manifest_path.display()
        )
    })?;

    let summary = ProfilingSummary {
        schema_version: "1",
        profile_kind: "infer",
        profiler: "cargo-flamegraph",
        weights_artifact: args.weights_artifact.to_string_lossy().to_string(),
        artifact_version: manifest.artifact_version,
        backend: args.backend,
        profile_command: args.profile_command,
        flamegraph_svg: args.input.to_string_lossy().to_string(),
        total_samples: extract_total_samples(svg.as_str())?,
        top_hotspots: extract_top_hotspots(svg.as_str(), 10)?,
    };

    let output = serde_json::to_string_pretty(&summary_json(&summary))
        .map_err(|err| format!("serialize profiling summary json: {err}"))?;
    fs::write(&args.output, format!("{output}\n"))
        .map_err(|err| format!("write profiling summary `{}`: {err}", args.output.display()))?;

    Ok(())
}

fn summary_json(summary: &ProfilingSummary) -> serde_json::Value {
    json!({
        "schema_version": summary.schema_version,
        "profile_kind": summary.profile_kind,
        "profiler": summary.profiler,
        "weights_artifact": summary.weights_artifact,
        "artifact_version": summary.artifact_version,
        "backend": summary.backend,
        "profile_command": summary.profile_command,
        "flamegraph_svg": summary.flamegraph_svg,
        "total_samples": summary.total_samples,
        "top_hotspots": summary
            .top_hotspots
            .iter()
            .map(|hotspot| json!({
                "symbol": hotspot.symbol,
                "samples": hotspot.samples,
                "percent": hotspot.percent,
            }))
            .collect::<Vec<_>>(),
    })
}

fn parse_args<I>(args: I) -> Result<Args, String>
where
    I: IntoIterator<Item = String>,
{
    let mut input = None;
    let mut output = None;
    let mut weights_artifact = None;
    let mut backend = None;
    let mut profile_command = None;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = args.next().map(PathBuf::from),
            "--output" => output = args.next().map(PathBuf::from),
            "--weights-artifact" => weights_artifact = args.next().map(PathBuf::from),
            "--backend" => backend = args.next(),
            "--profile-command" => profile_command = args.next(),
            "--help" | "-h" => return Err(usage()),
            other => return Err(format!("unknown argument `{other}`\n\n{}", usage())),
        }
    }

    Ok(Args {
        input: input.ok_or_else(usage)?,
        output: output.ok_or_else(usage)?,
        weights_artifact: weights_artifact.ok_or_else(usage)?,
        backend: backend.ok_or_else(usage)?,
        profile_command: profile_command.ok_or_else(usage)?,
    })
}

fn usage() -> String {
    "usage: afterburner-profile-summary --input <flamegraph.svg> --output <summary.json> --weights-artifact <path> --backend <backend> --profile-command <command>".to_string()
}

fn extract_total_samples(svg: &str) -> Result<u64, String> {
    let marker = "total_samples=\"";
    let start = svg
        .find(marker)
        .ok_or_else(|| "missing flamegraph total_samples attribute".to_string())?
        + marker.len();
    let end = svg[start..]
        .find('"')
        .ok_or_else(|| "unterminated flamegraph total_samples attribute".to_string())?
        + start;
    svg[start..end]
        .parse::<u64>()
        .map_err(|err| format!("parse flamegraph total_samples: {err}"))
}

fn extract_top_hotspots(svg: &str, limit: usize) -> Result<Vec<Hotspot>, String> {
    let mut hotspots = HashMap::<String, Hotspot>::new();
    let mut cursor = 0usize;

    while let Some(start_rel) = svg[cursor..].find("<title>") {
        let start = cursor + start_rel + "<title>".len();
        let end = svg[start..]
            .find("</title>")
            .ok_or_else(|| "unterminated flamegraph <title> element".to_string())?
            + start;
        let title = &svg[start..end];
        cursor = end + "</title>".len();

        let Some(hotspot) = parse_hotspot(title) else {
            continue;
        };
        if should_skip_symbol(hotspot.symbol.as_str()) {
            continue;
        }

        hotspots
            .entry(hotspot.symbol.clone())
            .and_modify(|existing| {
                if hotspot.samples > existing.samples
                    || (hotspot.samples == existing.samples && hotspot.percent > existing.percent)
                {
                    *existing = hotspot.clone();
                }
            })
            .or_insert(hotspot);
    }

    let mut hotspots = hotspots.into_values().collect::<Vec<_>>();
    hotspots.sort_by(|lhs, rhs| {
        rhs.samples
            .cmp(&lhs.samples)
            .then_with(|| rhs.percent.total_cmp(&lhs.percent))
            .then_with(|| lhs.symbol.cmp(&rhs.symbol))
    });
    hotspots.truncate(limit);
    Ok(hotspots)
}

fn should_skip_symbol(symbol: &str) -> bool {
    symbol.starts_with("0x")
        || symbol == "all"
        || symbol == "start"
        || symbol == "main"
        || symbol.ends_with("::main")
        || symbol.starts_with("std::")
        || symbol.starts_with("core::")
        || symbol.starts_with("alloc::")
        || symbol.starts_with("dyld")
        || symbol.starts_with('_')
}

fn parse_hotspot(title: &str) -> Option<Hotspot> {
    let (symbol, sample_info) = title.rsplit_once(" (")?;
    let sample_info = sample_info.strip_suffix(')')?;
    let (samples, percent) = sample_info.split_once(", ")?;
    let samples = samples.split_whitespace().next()?.parse::<u64>().ok()?;
    let percent = percent.strip_suffix('%')?.parse::<f64>().ok()?;

    Some(Hotspot {
        symbol: html_unescape(symbol),
        samples,
        percent,
    })
}

fn html_unescape(value: &str) -> String {
    value
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}
