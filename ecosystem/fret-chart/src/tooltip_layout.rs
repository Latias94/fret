pub(crate) fn split_tooltip_text_for_columns(text: &str) -> Option<(&str, &str)> {
    let (left, right) = text.split_once(": ")?;
    if left.is_empty() || right.is_empty() {
        return None;
    }
    Some((left, right))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tooltip_column_splitter_requires_non_empty_columns() {
        assert_eq!(split_tooltip_text_for_columns("x: 1"), Some(("x", "1")));
        assert_eq!(split_tooltip_text_for_columns("x: "), None);
        assert_eq!(split_tooltip_text_for_columns(": 1"), None);
        assert_eq!(split_tooltip_text_for_columns("no delimiter"), None);
    }
}
