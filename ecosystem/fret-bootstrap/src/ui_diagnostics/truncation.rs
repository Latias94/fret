fn truncate_string_bytes(s: &mut String, max_bytes: usize) {
    if s.len() <= max_bytes {
        return;
    }
    if max_bytes == 0 {
        s.clear();
        return;
    }

    let suffix = "...";
    if max_bytes <= suffix.len() {
        let mut idx = max_bytes;
        while idx > 0 && !s.is_char_boundary(idx) {
            idx -= 1;
        }
        s.truncate(idx);
        return;
    }

    let mut idx = max_bytes - suffix.len();
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    s.truncate(idx);
    s.push_str(suffix);
}

fn truncate_opt_string_bytes(s: &mut Option<String>, max_bytes: usize) {
    let Some(v) = s.as_mut() else {
        return;
    };
    truncate_string_bytes(v, max_bytes);
}

fn truncate_vec_string_bytes(items: &mut Vec<String>, max_bytes: usize) {
    for s in items {
        truncate_string_bytes(s, max_bytes);
    }
}
