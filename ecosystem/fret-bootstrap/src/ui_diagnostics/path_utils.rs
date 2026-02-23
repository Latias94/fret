fn display_path(base_dir: &Path, path: &Path) -> String {
    if let Ok(rel) = path.strip_prefix(base_dir) {
        return rel.to_string_lossy().to_string();
    }
    path.to_string_lossy().to_string()
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
