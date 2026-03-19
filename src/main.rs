mod cmd_drift_receipt;
mod cmd_eval;
mod cmd_infer;
mod cmd_train;
mod cmd_upload;

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(subcommand) = args.next() else {
        eprintln!("{}", usage());
        std::process::exit(2);
    };

    let code = match subcommand.as_str() {
        "train" => cmd_train::run(args),
        "infer" => cmd_infer::run(args),
        "eval" => cmd_eval::run(args),
        "drift-receipt" => cmd_drift_receipt::run(args),
        "upload" => cmd_upload::run(args),
        "--help" | "-h" | "help" => {
            println!("{}", usage());
            0
        }
        _ => {
            eprintln!("unknown subcommand: {subcommand}");
            eprintln!("{}", usage());
            2
        }
    };

    if code != 0 {
        std::process::exit(code);
    }
}

fn usage() -> &'static str {
    "usage: afterburner <train|infer|eval|drift-receipt|upload> [args]"
}
