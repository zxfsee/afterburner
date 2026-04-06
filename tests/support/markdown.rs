pub fn markdown_section<'a>(text: &'a str, heading: &str) -> &'a str {
    let start = text
        .find(heading)
        .unwrap_or_else(|| panic!("missing heading `{heading}`"));
    let rest = &text[start + heading.len()..];
    let end = rest.find("\n## ").unwrap_or(rest.len());
    &rest[..end]
}

pub fn markdown_contains(text: &str, needle: &str) -> bool {
    normalize_markdown(text).contains(&normalize_markdown(needle))
}

fn normalize_markdown(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
