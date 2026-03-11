use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
struct TodoItem {
    title: String,
    block: String,
}

fn todo_items() -> Vec<TodoItem> {
    let cargo_toml =
        fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .expect("read Cargo.toml");

    let todo_section = cargo_toml
        .split("## TODO")
        .nth(1)
        .and_then(|rest| rest.split("## [Trunk]").next())
        .expect("Cargo.toml changelog template must contain TODO section");

    let mut items = Vec::new();
    let mut current_title = None;
    let mut current_lines = Vec::new();

    for line in todo_section.lines() {
        if let Some(title) = line.strip_prefix("- ") {
            if let Some(previous_title) = current_title.take() {
                items.push(TodoItem {
                    title: previous_title,
                    block: current_lines.join("\n"),
                });
                current_lines.clear();
            }
            current_title = Some(title.trim().to_string());
        } else if current_title.is_some() {
            current_lines.push(line.to_string());
        }
    }

    if let Some(last_title) = current_title {
        items.push(TodoItem {
            title: last_title,
            block: current_lines.join("\n"),
        });
    }

    items
}

#[test]
fn active_todo_horizon_stays_populated_and_metadata_consistent() {
    let items = todo_items();
    assert!(
        items.len() >= 6,
        "active TODO horizon must keep at least 6 items; found {}",
        items.len()
    );

    for item in items {
        for required in ["Goal:", "Kind:", "Boundary:", "Contracts:", "Scope:"] {
            assert!(
                item.block.contains(required),
                "TODO `{}` must include `{required}` metadata",
                item.title
            );
        }

        let lower_title = item.title.to_ascii_lowercase();
        if item.block.contains("Kind: `gate`") {
            assert!(
                lower_title.contains("gate")
                    || lower_title.contains("fixture")
                    || lower_title.contains("ci"),
                "gate TODO `{}` must advertise gate/fixture/CI in the title",
                item.title
            );
        }

        if item.block.contains("Kind: `behavior`") || item.block.contains("Kind: `mixed`") {
            assert!(
                !lower_title.contains("gate"),
                "behavior/mixed TODO `{}` must not advertise gate in the title",
                item.title
            );
        }
    }
}
