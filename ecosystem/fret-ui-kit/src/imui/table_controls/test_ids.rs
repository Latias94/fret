use super::TableColumn;

pub(super) fn column_test_id_suffix(column: &TableColumn, index: usize) -> String {
    column
        .id()
        .map(test_id_slug)
        .filter(|slug| !slug.is_empty())
        .unwrap_or_else(|| index.to_string())
}

fn test_id_slug(s: &str) -> String {
    let mut out = String::new();
    let mut last_was_separator = false;

    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_was_separator = false;
        } else if !out.is_empty() && !last_was_separator {
            out.push('-');
            last_was_separator = true;
        }
    }

    if out.ends_with('-') {
        out.pop();
    }

    out
}
