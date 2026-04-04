use std::path::PathBuf;

struct Args {
    input: PathBuf,
    output: PathBuf,
    weights_artifact: PathBuf,
    backend: String,
    profile_command: String,
}

fn main() -> Result<(), String> {
    let args = parse_args(std::env::args().skip(1))?;
    afterburner::profiling_summary::write_infer_hotspot_summary(
        &args.input,
        &args.output,
        &args.weights_artifact,
        &args.backend,
        &args.profile_command,
    )
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
