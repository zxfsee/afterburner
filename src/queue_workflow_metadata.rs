#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstTodoItem {
    pub title: String,
    pub lines: Vec<String>,
}

pub fn extract_marked_section<'a>(
    text: &'a str,
    start_marker: &str,
    end_marker: &str,
) -> Result<&'a str, String> {
    text.split(start_marker)
        .nth(1)
        .and_then(|rest| rest.split(end_marker).next())
        .ok_or_else(|| format!("missing `{start_marker}`..`{end_marker}` section"))
}

pub fn normalized_marked_section_block(
    text: &str,
    start_marker: &str,
    end_marker: &str,
) -> Result<String, String> {
    let section = extract_marked_section(text, start_marker, end_marker)?;
    let normalized_lines = section.lines().map(str::trim_end).collect::<Vec<_>>();
    Ok(normalized_lines.join("\n").trim().to_string())
}

pub fn first_todo_item(section: &str) -> Result<FirstTodoItem, String> {
    let mut lines = section.lines();
    let title = lines
        .by_ref()
        .find_map(|line| {
            line.strip_prefix("- ")
                .map(|title| title.trim().to_string())
        })
        .ok_or_else(|| "TODO section does not contain any item".to_string())?;
    let mut current_lines = Vec::new();
    for line in lines {
        if line.starts_with("- ") {
            break;
        }
        current_lines.push(line.trim().to_string());
    }
    Ok(FirstTodoItem {
        title,
        lines: current_lines,
    })
}

pub fn extract_backtick_values(line: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut in_tick = false;

    for ch in line.chars() {
        if ch == '`' {
            if in_tick && !current.is_empty() {
                values.push(normalize_scope_path(&current));
                current.clear();
            }
            in_tick = !in_tick;
            continue;
        }
        if in_tick {
            current.push(ch);
        }
    }

    values
}

pub fn normalize_scope_path(path: &str) -> String {
    let mut normalized = path.replace('\\', "/");
    while normalized.starts_with("./") {
        normalized = normalized.trim_start_matches("./").to_string();
    }
    let is_dir = normalized.ends_with('/');
    let normalized = normalized.trim_matches('/').to_string();
    if is_dir && !normalized.is_empty() {
        format!("{normalized}/")
    } else {
        normalized
    }
}

pub fn normalize_candidate_path(path: &str) -> String {
    let mut normalized = path.replace('\\', "/");
    while normalized.starts_with("./") {
        normalized = normalized.trim_start_matches("./").to_string();
    }
    normalized.trim_matches('/').to_string()
}

pub fn is_allowed_path(allowed_paths: &[String], candidate: &str) -> bool {
    allowed_paths.iter().any(|allowed| {
        let allowed = normalize_scope_path(allowed);
        if allowed.ends_with('/') {
            candidate == allowed.trim_end_matches('/') || candidate.starts_with(&allowed)
        } else {
            candidate == allowed
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{
        extract_backtick_values, extract_marked_section, first_todo_item, is_allowed_path,
        normalized_marked_section_block,
    };

    #[test]
    fn marked_section_extracts_and_normalizes_block() {
        let text = "prefix\n## TODO\n\n- A\n  - Scope: `src/`\n\n## [Trunk]\nsuffix\n";
        let section = extract_marked_section(text, "## TODO", "## [Trunk]").expect("section");
        assert!(section.contains("- A"));
        let block = normalized_marked_section_block(text, "## TODO", "## [Trunk]").expect("block");
        assert_eq!(block, "- A\n  - Scope: `src/`");
    }

    #[test]
    fn first_todo_item_reads_title_and_lines() {
        let section = "\n\n- A\n  - Scope: `src/`, `tests/`\n\n- B\n  - Scope: `docs/`\n";
        let item = first_todo_item(section).expect("item");
        assert_eq!(item.title, "A");
        assert_eq!(item.lines, vec!["- Scope: `src/`, `tests/`"]);
    }

    #[test]
    fn extract_backtick_values_keeps_only_quoted_paths() {
        let values = extract_backtick_values("- Scope: `src/`, `tests/`, `docs/workflows.md`");
        assert_eq!(values, vec!["src/", "tests/", "docs/workflows.md"]);
    }

    #[test]
    fn path_allowlist_matches_files_and_directories() {
        assert!(is_allowed_path(&["Cargo.toml".into()], "Cargo.toml"));
        assert!(is_allowed_path(&["src/".into()], "src/bin/lock.rs"));
        assert!(!is_allowed_path(&["src/".into()], "tests/lock.rs"));
    }
}
