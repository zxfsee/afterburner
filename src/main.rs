mod cmd_drift_baseline;
mod cmd_drift_baseline_approval;
mod cmd_drift_baseline_bundle;
mod cmd_drift_baseline_checkpoint;
mod cmd_drift_baseline_handoff;
mod cmd_drift_baseline_history;
mod cmd_drift_baseline_pointer;
mod cmd_drift_baseline_refresh;
mod cmd_drift_baseline_rollback;
mod cmd_drift_baseline_supersession;
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
        "drift-baseline" => cmd_drift_baseline::run(args),
        "drift-approve-baseline" => cmd_drift_baseline_approval::run(args),
        "drift-export-baseline-bundle" => cmd_drift_baseline_bundle::run(args),
        "drift-export-baseline-handoff" => cmd_drift_baseline_handoff::run(args),
        "drift-checkpoint-baseline" => cmd_drift_baseline_checkpoint::run(args),
        "drift-record-approved-baseline-history" => cmd_drift_baseline_history::run(args),
        "drift-point-approved-baseline" => cmd_drift_baseline_pointer::run(args),
        "drift-rollback-approved-baseline" => cmd_drift_baseline_rollback::run(args),
        "drift-refresh-baseline" => cmd_drift_baseline_refresh::run(args),
        "drift-supersede-baseline-approval" => cmd_drift_baseline_supersession::run(args),
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
    "usage: afterburner <train|infer|eval|drift-baseline|drift-approve-baseline|drift-checkpoint-baseline|drift-export-baseline-bundle|drift-export-baseline-handoff|drift-record-approved-baseline-history|drift-point-approved-baseline|drift-refresh-baseline|drift-rollback-approved-baseline|drift-supersede-baseline-approval|drift-receipt|upload> [args]"
}
