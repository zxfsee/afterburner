use std::collections::BTreeMap;
use std::fs;
use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};
use std::time::Duration;

use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::border::Set as BorderSet;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use serde_json::{Map, Value};

const ASCII_BORDER: BorderSet = BorderSet {
    top_left: "+",
    top_right: "+",
    bottom_left: "+",
    bottom_right: "+",
    vertical_left: "|",
    vertical_right: "|",
    horizontal_top: "-",
    horizontal_bottom: "-",
};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{}", usage());
        return;
    }

    let exit_code = match run(args.into_iter()) {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("{err}");
            2
        }
    };

    if exit_code != 0 {
        std::process::exit(exit_code);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DashboardArgs {
    input: PathBuf,
    snapshot: bool,
    width: u16,
    height: u16,
    refresh_ms: u64,
}

#[derive(Debug, Clone)]
struct DashboardEvent {
    ts_ms: u64,
    level: String,
    source: String,
    event: String,
    fields: Map<String, Value>,
}

#[derive(Debug, Default, Clone)]
struct DashboardState {
    input_label: String,
    total_events: usize,
    error_events: usize,
    counts: BTreeMap<String, usize>,
    last_event: Option<DashboardEvent>,
    latest_train: Option<DashboardEvent>,
    latest_eval: Option<DashboardEvent>,
    latest_infer: Option<DashboardEvent>,
}

fn run<I>(args: I) -> Result<(), String>
where
    I: Iterator<Item = String>,
{
    let args = parse_args(args)?;
    if args.snapshot {
        let state = load_dashboard_state(args.input.as_path())?;
        let snapshot = render_snapshot(&state, args.width, args.height)
            .map_err(|err| format!("render dashboard snapshot: {err}"))?;
        print!("{snapshot}");
        return Ok(());
    }

    if !io::stdout().is_terminal() {
        return Err(
            "dashboard live mode requires a terminal; use --snapshot for CI or piping".into(),
        );
    }

    run_live(args)
}

fn run_live(args: DashboardArgs) -> Result<(), String> {
    let mut terminal =
        ratatui::try_init().map_err(|err| format!("initialize dashboard terminal: {err}"))?;
    let _restore = RestoreTerminal;

    loop {
        let state = load_dashboard_state(args.input.as_path())?;
        terminal
            .draw(|frame| render_dashboard(frame, &state))
            .map_err(|err| format!("draw dashboard frame: {err}"))?;

        if event::poll(Duration::from_millis(args.refresh_ms))
            .map_err(|err| format!("poll dashboard input: {err}"))?
        {
            let input = event::read().map_err(|err| format!("read dashboard input: {err}"))?;
            if should_exit(input) {
                break;
            }
        }
    }

    Ok(())
}

fn should_exit(input: Event) -> bool {
    matches!(
        input,
        Event::Key(key)
            if key.kind == KeyEventKind::Press
                && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
    )
}

struct RestoreTerminal;

impl Drop for RestoreTerminal {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

fn usage() -> &'static str {
    "usage: afterburner-dashboard [--input PATH] [--snapshot] [--width N] [--height N] [--refresh-ms N]"
}

fn parse_args<I>(args: I) -> Result<DashboardArgs, String>
where
    I: Iterator<Item = String>,
{
    let mut args = args.peekable();
    let mut parsed = DashboardArgs {
        input: PathBuf::from("artifacts/train/observability.jsonl"),
        snapshot: false,
        width: 80,
        height: 18,
        refresh_ms: 1000,
    };

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => {
                let value: String = parse_value(&mut args, "--input")?;
                parsed.input = PathBuf::from(value);
            }
            "--snapshot" => parsed.snapshot = true,
            "--width" => parsed.width = parse_value(&mut args, "--width")?,
            "--height" => parsed.height = parse_value(&mut args, "--height")?,
            "--refresh-ms" => parsed.refresh_ms = parse_value(&mut args, "--refresh-ms")?,
            _ if arg.starts_with("--input=") => {
                parsed.input = PathBuf::from(arg.trim_start_matches("--input="))
            }
            _ if arg.starts_with("--width=") => {
                parsed.width = parse_inline_value(arg.trim_start_matches("--width="), "--width")?
            }
            _ if arg.starts_with("--height=") => {
                parsed.height = parse_inline_value(arg.trim_start_matches("--height="), "--height")?
            }
            _ if arg.starts_with("--refresh-ms=") => {
                parsed.refresh_ms =
                    parse_inline_value(arg.trim_start_matches("--refresh-ms="), "--refresh-ms")?
            }
            _ => {
                return Err(format!(
                    "unknown argument for afterburner-dashboard: {arg}\n{}",
                    usage()
                ));
            }
        }
    }

    if parsed.width < 40 {
        return Err(format!("--width must be >= 40\n{}", usage()));
    }
    if parsed.height < 12 {
        return Err(format!("--height must be >= 12\n{}", usage()));
    }
    if parsed.refresh_ms == 0 {
        return Err(format!("--refresh-ms must be > 0\n{}", usage()));
    }

    Ok(parsed)
}

fn parse_value<T, I>(args: &mut I, flag: &str) -> Result<T, String>
where
    T: std::str::FromStr,
    I: Iterator<Item = String>,
{
    let value = args
        .next()
        .ok_or_else(|| format!("missing value for {flag}\n{}", usage()))?;
    parse_inline_value(value.as_str(), flag)
}

fn parse_inline_value<T>(value: &str, flag: &str) -> Result<T, String>
where
    T: std::str::FromStr,
{
    value
        .parse::<T>()
        .map_err(|_| format!("invalid value for {flag}: {value}\n{}", usage()))
}

fn load_dashboard_state(path: &Path) -> Result<DashboardState, String> {
    let contents = fs::read_to_string(path)
        .map_err(|err| format!("read dashboard input {}: {err}", path.display()))?;
    let mut state = DashboardState {
        input_label: path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string()),
        ..DashboardState::default()
    };

    for (line_index, line) in contents.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let event = parse_event_line(line)
            .map_err(|err| format!("parse dashboard input line {}: {err}", line_index + 1))?;
        ingest_event(&mut state, event);
    }

    Ok(state)
}

fn parse_event_line(line: &str) -> Result<DashboardEvent, String> {
    let value: Value =
        serde_json::from_str(line).map_err(|err| format!("invalid json event line: {err}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "dashboard input line must be a json object".to_string())?;

    if let Some(fields) = object.get("fields").and_then(Value::as_object) {
        return Ok(DashboardEvent {
            ts_ms: object
                .get("ts_ms")
                .and_then(Value::as_u64)
                .unwrap_or_default(),
            level: object
                .get("level")
                .and_then(Value::as_str)
                .unwrap_or("info")
                .to_string(),
            source: object
                .get("source")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
            event: object
                .get("event")
                .and_then(Value::as_str)
                .ok_or_else(|| "event envelope missing string `event`".to_string())?
                .to_string(),
            fields: fields.clone(),
        });
    }

    let event = object
        .get("event")
        .and_then(Value::as_str)
        .ok_or_else(|| "legacy dashboard line missing string `event`".to_string())?;
    let mut fields = Map::new();
    for (key, value) in object {
        if matches!(key.as_str(), "ts_ms" | "level" | "source" | "event") {
            continue;
        }
        fields.insert(key.clone(), value.clone());
    }

    Ok(DashboardEvent {
        ts_ms: object
            .get("ts_ms")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        level: object
            .get("level")
            .and_then(Value::as_str)
            .unwrap_or("info")
            .to_string(),
        source: object
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("legacy")
            .to_string(),
        event: event.to_string(),
        fields,
    })
}

fn ingest_event(state: &mut DashboardState, event: DashboardEvent) {
    state.total_events += 1;
    if event.level == "error" {
        state.error_events += 1;
    }
    *state.counts.entry(event.event.clone()).or_insert(0) += 1;

    match event.event.as_str() {
        "train_start"
        | "train_done"
        | "artifact_exported"
        | "distributed_shard_metadata_invalid" => {
            state.latest_train = Some(event.clone());
        }
        "eval_done" | "eval_error" | "mnist_eval_gate_failed" | "mnist_eval_gate_passed" => {
            state.latest_eval = Some(event.clone());
        }
        "infer_done" | "calibration_metadata_invalid" => {
            state.latest_infer = Some(event.clone());
        }
        _ => {}
    }

    state.last_event = Some(event);
}

fn render_snapshot(state: &DashboardState, width: u16, height: u16) -> io::Result<String> {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|frame| render_dashboard(frame, state))?;
    Ok(buffer_to_string(terminal.backend().buffer()))
}

fn buffer_to_string(buffer: &ratatui::buffer::Buffer) -> String {
    let mut out = String::new();
    for y in 0..buffer.area.height {
        let mut line = String::new();
        for x in 0..buffer.area.width {
            line.push_str(buffer[(x, y)].symbol());
        }
        out.push_str(line.trim_end_matches(' '));
        out.push('\n');
    }
    out
}

fn render_dashboard(frame: &mut Frame<'_>, state: &DashboardState) {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(6),
            Constraint::Length(5),
            Constraint::Min(6),
        ])
        .split(frame.area());

    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(vertical[3]);

    render_block(
        frame,
        vertical[0],
        "Afterburner Dashboard",
        header_lines(state),
        Color::Cyan,
    );
    render_block(
        frame,
        vertical[1],
        "Train",
        train_lines(state),
        Color::LightRed,
    );
    render_block(
        frame,
        vertical[2],
        "Eval",
        eval_lines(state),
        Color::LightBlue,
    );
    render_block(
        frame,
        bottom[0],
        "Infer",
        infer_lines(state),
        Color::LightGreen,
    );
    render_block(
        frame,
        bottom[1],
        "Event Counts",
        count_lines(state),
        Color::Yellow,
    );
}

fn render_block(frame: &mut Frame<'_>, area: Rect, title: &str, lines: Vec<String>, color: Color) {
    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_set(ASCII_BORDER);
    let paragraph = Paragraph::new(lines.into_iter().map(Line::from).collect::<Vec<_>>())
        .block(block)
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn header_lines(state: &DashboardState) -> Vec<String> {
    let mut lines = vec![
        format!("Input: {}", state.input_label),
        format!(
            "Events: {}  Errors: {}",
            state.total_events, state.error_events
        ),
    ];

    if let Some(event) = state.last_event.as_ref() {
        lines.push(format!(
            "Last: {} from {} at {}",
            event.event, event.source, event.ts_ms
        ));
        lines.push(format!("Level: {}", event.level));
    } else {
        lines.push("Last: none".to_string());
        lines.push("Level: n/a".to_string());
    }

    lines
}

fn train_lines(state: &DashboardState) -> Vec<String> {
    let Some(event) = state.latest_train.as_ref() else {
        return vec!["No training events loaded.".to_string()];
    };

    vec![
        format!("Event: {}", event.event),
        format!(
            "Backend: {}  Artifact: {}",
            field_string(&event.fields, "backend"),
            field_string(&event.fields, "artifact_version")
        ),
        format!(
            "Batch: {}  Workers: {}  Epochs: {}",
            field_string(&event.fields, "batch_size"),
            field_string(&event.fields, "worker_parallelism"),
            field_string(&event.fields, "num_epochs")
        ),
        format!(
            "Planned: {}  Throughput: {}/s",
            field_string(&event.fields, "planned_samples"),
            field_string(&event.fields, "throughput_samples_per_sec")
        ),
        format!("Elapsed: {} ms", field_string(&event.fields, "elapsed_ms")),
    ]
}

fn eval_lines(state: &DashboardState) -> Vec<String> {
    let Some(event) = state.latest_eval.as_ref() else {
        return vec!["No eval events loaded.".to_string()];
    };

    vec![
        format!("Event: {}", event.event),
        format!(
            "Accuracy: {}  Samples: {}  Batch: {}",
            field_string(&event.fields, "accuracy"),
            field_string(&event.fields, "samples"),
            field_string(&event.fields, "batch_size")
        ),
        format!(
            "Rate: {}/s  Duration: {} ms",
            field_string(&event.fields, "rate_samples_per_sec"),
            field_string(&event.fields, "duration_ms")
        ),
        format!(
            "Seed: {}  Errors: {}",
            field_string(&event.fields, "seed"),
            field_string(&event.fields, "error_count")
        ),
    ]
}

fn infer_lines(state: &DashboardState) -> Vec<String> {
    let Some(event) = state.latest_infer.as_ref() else {
        return vec!["No infer events loaded.".to_string()];
    };

    vec![
        format!("Event: {}", event.event),
        format!("Backend: {}", field_string(&event.fields, "backend")),
        format!(
            "Artifact: {}",
            field_string(&event.fields, "artifact_version")
        ),
        format!("Batch: {}", field_string(&event.fields, "batch_size")),
        format!("Elapsed: {} ms", field_string(&event.fields, "elapsed_ms")),
    ]
}

fn count_lines(state: &DashboardState) -> Vec<String> {
    if state.counts.is_empty() {
        return vec!["No event counts available.".to_string()];
    }

    state
        .counts
        .iter()
        .map(|(event, count)| format!("{event} x{count}"))
        .collect()
}

fn field_string(fields: &Map<String, Value>, key: &str) -> String {
    match fields.get(key) {
        Some(Value::String(value)) => value.clone(),
        Some(Value::Number(value)) => value.to_string(),
        Some(Value::Bool(value)) => value.to_string(),
        Some(Value::Null) | None => "n/a".to_string(),
        Some(other) => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{field_string, parse_args, parse_event_line};

    #[test]
    fn parse_args_accepts_snapshot_controls() {
        let args = vec![
            "--input".to_string(),
            "custom.jsonl".to_string(),
            "--snapshot".to_string(),
            "--width=100".to_string(),
            "--height".to_string(),
            "20".to_string(),
            "--refresh-ms".to_string(),
            "250".to_string(),
        ];

        let parsed = parse_args(args.into_iter()).expect("parse dashboard args");
        assert_eq!(parsed.input.to_string_lossy(), "custom.jsonl");
        assert!(parsed.snapshot);
        assert_eq!(parsed.width, 100);
        assert_eq!(parsed.height, 20);
        assert_eq!(parsed.refresh_ms, 250);
    }

    #[test]
    fn parse_event_line_normalizes_legacy_top_level_fields() {
        let line =
            r#"{"event":"train_done","backend":"cpu","artifact_version":"0.1.0","elapsed_ms":14}"#;
        let event = parse_event_line(line).expect("parse legacy event line");
        assert_eq!(event.source, "legacy");
        assert_eq!(event.event, "train_done");
        assert_eq!(field_string(&event.fields, "backend"), "cpu");
        assert_eq!(field_string(&event.fields, "artifact_version"), "0.1.0");
        assert_eq!(field_string(&event.fields, "elapsed_ms"), "14");
    }
}
