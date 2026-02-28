fn display_path(base_dir: &Path, path: &Path) -> String {
    if let Ok(rel) = path.strip_prefix(base_dir) {
        return rel.to_string_lossy().to_string();
    }
    path.to_string_lossy().to_string()
}

fn sanitize_path_for_bundle(base_dir: &Path, path: &Path) -> String {
    if let Ok(rel) = path.strip_prefix(base_dir) {
        return rel.to_string_lossy().to_string();
    }
    path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn maybe_redact_string(s: &str, redact_text: bool) -> String {
    if !redact_text {
        return s.to_string();
    }
    format!("<redacted len={}>", s.len())
}

fn sanitize_label(label: &str) -> String {
    let mut out = String::with_capacity(label.len());
    for c in label.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
            out.push(c);
        } else if matches!(c, ' ' | ':' | '/' | '\\') {
            out.push('_');
        }
    }
    if out.is_empty() {
        "bundle".to_string()
    } else {
        out
    }
}

fn format_bundle_dump_note(
    label: &str,
    dump_max_snapshots: Option<usize>,
    request_id: Option<u64>,
) -> String {
    if dump_max_snapshots.is_none() && request_id.is_none() {
        return label.to_string();
    }

    let mut out = format!("label={label}");
    if let Some(n) = dump_max_snapshots {
        out.push_str(&format!(" max_snapshots={n}"));
    }
    if let Some(id) = request_id {
        out.push_str(&format!(" request_id={id}"));
    }
    out
}
