pub(super) fn normalize(input: &str) -> String {
    input.trim().to_ascii_lowercase()
}

pub(super) fn split_category_label(label: &str) -> (Option<&str>, &str) {
    label
        .rsplit_once('/')
        .map(|(category, name)| (Some(category), name))
        .unwrap_or((None, label))
}

pub(super) fn score_candidate(query: &str, label: &str, kind: &str) -> Option<(u8, usize)> {
    if query.is_empty() {
        return Some((0, 0));
    }
    let label_lc = label.to_ascii_lowercase();
    if let Some(index) = label_lc.find(query) {
        let bucket = if index == 0 { 0 } else { 1 };
        return Some((bucket, index));
    }
    let kind_lc = kind.to_ascii_lowercase();
    kind_lc.find(query).map(|index| (2, index))
}
