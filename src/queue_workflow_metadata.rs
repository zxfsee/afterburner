#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueWorkflowMetadataError {
    MissingMarkedSection {
        start_marker: String,
        end_marker: String,
    },
    MissingMarker {
        marker: String,
    },
    MissingItem,
    MissingItemTitle,
}

impl std::fmt::Display for QueueWorkflowMetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingMarkedSection {
                start_marker,
                end_marker,
            } => write!(f, "missing `{start_marker}`..`{end_marker}` section"),
            Self::MissingMarker { marker } => write!(f, "missing `{marker}` marker"),
            Self::MissingItem => write!(f, "TODO section does not contain any item"),
            Self::MissingItemTitle => write!(f, "TODO item missing title"),
        }
    }
}

impl std::error::Error for QueueWorkflowMetadataError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstTodoItem {
    pub title: String,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueTextItem {
    pub title: String,
    pub lines: Vec<String>,
    pub blocked_by: Option<String>,
}

pub fn extract_marked_section<'a>(
    text: &'a str,
    start_marker: &str,
    end_marker: &str,
) -> Result<&'a str, QueueWorkflowMetadataError> {
    text.split(start_marker)
        .nth(1)
        .and_then(|rest| rest.split(end_marker).next())
        .ok_or_else(|| QueueWorkflowMetadataError::MissingMarkedSection {
            start_marker: start_marker.to_string(),
            end_marker: end_marker.to_string(),
        })
}

pub fn normalized_marked_section_block(
    text: &str,
    start_marker: &str,
    end_marker: &str,
) -> Result<String, QueueWorkflowMetadataError> {
    let section = extract_marked_section(text, start_marker, end_marker)?;
    let normalized_lines = section.lines().map(str::trim_end).collect::<Vec<_>>();
    Ok(normalized_lines.join("\n").trim().to_string())
}

pub fn first_todo_item(section: &str) -> Result<FirstTodoItem, QueueWorkflowMetadataError> {
    let mut lines = section.lines();
    let title = lines
        .by_ref()
        .find_map(|line| {
            line.strip_prefix("- ")
                .map(|title| title.trim().to_string())
        })
        .ok_or(QueueWorkflowMetadataError::MissingItem)?;
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

pub fn split_marked_items(
    text: &str,
    start_marker: &str,
    end_marker: &str,
) -> Result<(String, String, Vec<QueueTextItem>), QueueWorkflowMetadataError> {
    let start = text.find(start_marker).ok_or_else(|| {
        QueueWorkflowMetadataError::MissingMarkedSection {
            start_marker: start_marker.to_string(),
            end_marker: end_marker.to_string(),
        }
    })?;
    let suffix_start = text[start..]
        .find(end_marker)
        .map(|offset| start + offset)
        .ok_or_else(|| QueueWorkflowMetadataError::MissingMarker {
            marker: end_marker.to_string(),
        })?;
    let prefix_end = start + start_marker.len();
    let prefix = text[..prefix_end].to_string();
    let section = text[prefix_end..suffix_start].to_string();
    let suffix = text[suffix_start..].to_string();
    Ok((prefix, suffix, parse_item_section(&section, true)?))
}

pub fn split_items_after_marker(
    text: &str,
    start_marker: &str,
) -> Result<(String, String, Vec<QueueTextItem>), QueueWorkflowMetadataError> {
    let start =
        text.find(start_marker)
            .ok_or_else(|| QueueWorkflowMetadataError::MissingMarker {
                marker: start_marker.to_string(),
            })?;
    let prefix_end = start + start_marker.len();
    let prefix = text[..prefix_end].to_string();
    let section = text[prefix_end..].to_string();
    Ok((prefix, String::new(), parse_item_section(&section, false)?))
}

pub fn parse_item_section(
    section: &str,
    require_non_empty: bool,
) -> Result<Vec<QueueTextItem>, QueueWorkflowMetadataError> {
    let mut items = Vec::new();
    let mut current = Vec::<String>::new();

    for line in section.lines() {
        if line.starts_with("- ") {
            if !current.is_empty() {
                items.push(parse_text_item(&current)?);
            }
            current = vec![line.to_string()];
        } else if !current.is_empty() {
            current.push(line.to_string());
        }
    }

    if !current.is_empty() {
        items.push(parse_text_item(&current)?);
    }

    if require_non_empty && items.is_empty() {
        return Err(QueueWorkflowMetadataError::MissingItem);
    }
    Ok(items)
}

pub fn rebuild_item_block(prefix: &str, items: &[QueueTextItem], suffix: &str) -> String {
    let mut out = String::new();
    out.push_str(prefix);
    out.push('\n');
    out.push('\n');
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        for line in &item.lines {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !suffix.starts_with('\n') {
        out.push('\n');
    }
    out.push_str(suffix);
    out
}

fn parse_text_item(lines: &[String]) -> Result<QueueTextItem, QueueWorkflowMetadataError> {
    let title = lines[0]
        .strip_prefix("- ")
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .ok_or(QueueWorkflowMetadataError::MissingItemTitle)?
        .to_string();
    let blocked_by = lines.iter().find_map(|line| {
        line.trim()
            .strip_prefix("- Blocked-by:")
            .map(|value| value.trim().to_string())
    });
    Ok(QueueTextItem {
        title,
        lines: lines.to_vec(),
        blocked_by,
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
        normalized_marked_section_block, parse_item_section, rebuild_item_block,
        split_items_after_marker, split_marked_items,
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

    #[test]
    fn split_marked_items_extracts_prefix_suffix_and_items() {
        let text = "prefix\n## TODO\n\n- A\n  - Scope: `src/`\n\n- B\n  - Scope: `docs/`\n\n## [Trunk]\nsuffix\n";
        let (prefix, suffix, items) =
            split_marked_items(text, "## TODO", "## [Trunk]").expect("items");
        assert_eq!(prefix, "prefix\n## TODO");
        assert_eq!(suffix, "## [Trunk]\nsuffix\n");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "A");
        assert_eq!(items[1].title, "B");
    }

    #[test]
    fn split_items_after_marker_extracts_backlog_items() {
        let text = "prefix\n## Items\n\n- A\n  - Goal: x\n";
        let (prefix, suffix, items) = split_items_after_marker(text, "## Items").expect("items");
        assert_eq!(prefix, "prefix\n## Items");
        assert!(suffix.is_empty());
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "A");
    }

    #[test]
    fn parse_and_rebuild_item_section_round_trips() {
        let section = "\n\n- A\n  - Goal: x\n  - Blocked-by: wait\n\n- B\n  - Goal: y\n";
        let items = parse_item_section(section, true).expect("items");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].blocked_by.as_deref(), Some("wait"));
        let rebuilt = rebuild_item_block("## TODO", &items, "## [Trunk]");
        assert!(rebuilt.contains("- A"));
        assert!(rebuilt.contains("- B"));
        assert!(rebuilt.contains("## [Trunk]"));
    }
}
