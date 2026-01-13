use std::path::Path;

pub(super) fn sanitize_package_name(raw: &str) -> Result<String, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("package name cannot be empty".to_string());
    }

    let mut out = String::with_capacity(raw.len());
    for c in raw.chars() {
        let c = if c.is_ascii_whitespace() { '-' } else { c };
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
            out.push(c.to_ascii_lowercase());
        } else {
            return Err(format!(
                "invalid package name: `{raw}` (unsupported character: `{c}`)"
            ));
        }
    }

    Ok(out)
}

pub(super) fn ensure_dir_is_new_or_empty(path: &Path) -> Result<(), String> {
    if path.exists() {
        let mut entries = std::fs::read_dir(path).map_err(|e| e.to_string())?;
        if entries.next().is_some() {
            return Err(format!(
                "output directory is not empty: {} (choose another --path)",
                path.display()
            ));
        }
        return Ok(());
    }

    std::fs::create_dir_all(path).map_err(|e| e.to_string())
}

pub(super) fn write_new_file(path: &Path, contents: &str) -> Result<(), String> {
    if path.exists() {
        return Err(format!(
            "refusing to overwrite existing file: {}",
            path.display()
        ));
    }
    std::fs::write(path, contents).map_err(|e| e.to_string())
}
