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
    if path.exists() && !path.is_dir() {
        return Err(format!(
            "output path exists but is not a directory: {}",
            path.display()
        ));
    }
    if path.exists() {
        let entries = std::fs::read_dir(path).map_err(|e| e.to_string())?;
        let mut blocking: Vec<String> = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let Some(name) = entry.file_name().to_str().map(|s| s.to_string()) else {
                blocking.push("<non-utf8>".to_string());
                continue;
            };
            if matches!(name.as_str(), ".git" | ".gitignore") {
                continue;
            }
            blocking.push(name);
        }
        if !blocking.is_empty() {
            blocking.sort();
            let sample = blocking.iter().take(8).cloned().collect::<Vec<_>>();
            let more = blocking.len().saturating_sub(sample.len());
            let mut msg = format!(
                "output directory is not empty: {} (choose another --path)",
                path.display()
            );
            msg.push_str(&format!("\n  found: {}", sample.join(", ")));
            if more > 0 {
                msg.push_str(&format!(" (+{more} more)"));
            }
            msg.push_str(
                "\n  note: `.git/` is allowed, but generated files (Cargo.toml, src/, README.md) must not already exist",
            );
            return Err(msg);
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

pub(super) fn write_file_if_missing(path: &Path, contents: &str) -> Result<bool, String> {
    if path.exists() {
        if path.is_dir() {
            return Err(format!(
                "refusing to write file because a directory exists at: {}",
                path.display()
            ));
        }
        return Ok(false);
    }
    std::fs::write(path, contents).map_err(|e| e.to_string())?;
    Ok(true)
}

pub(super) fn workspace_prefix_from_out_dir(
    workspace_root: &Path,
    out_dir: &Path,
) -> Result<String, String> {
    let workspace_root = workspace_root.canonicalize().map_err(|e| {
        format!(
            "failed to canonicalize workspace root `{}`: {e}",
            workspace_root.display()
        )
    })?;
    let out_dir = out_dir.canonicalize().map_err(|e| {
        format!(
            "failed to canonicalize output directory `{}`: {e}",
            out_dir.display()
        )
    })?;

    let Ok(relative) = out_dir.strip_prefix(&workspace_root) else {
        return Err(format!(
            "output directory must be inside the workspace root\n  workspace: {}\n  output:    {}",
            workspace_root.display(),
            out_dir.display()
        ));
    };

    let depth = relative.components().count();
    if depth == 0 {
        return Ok(".".to_string());
    }
    Ok(std::iter::repeat_n("..", depth)
        .collect::<Vec<_>>()
        .join("/"))
}
