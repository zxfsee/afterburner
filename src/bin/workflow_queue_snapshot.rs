use afterburner::workflow_queue_snapshot_cli::parse_args;
use afterburner::workflow_queue_snapshot_core::run;

fn main() {
    let code = match parse_args(std::env::args().skip(1)) {
        Ok(command) => match run(command) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("{err}");
                2
            }
        },
        Err(err) => {
            eprintln!("{err}");
            2
        }
    };
    if code != 0 {
        std::process::exit(code);
    }
}
