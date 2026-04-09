use std::path::Path;

use crate::GeneratePackError;

pub(crate) fn sanitize_package_name(raw: &str) -> Result<String, GeneratePackError> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(GeneratePackError::InvalidPackageName(
            "package name cannot be empty".to_string(),
        ));
    }

    let mut out = String::with_capacity(raw.len());
    for c in raw.chars() {
        let c = if c.is_ascii_whitespace() { '-' } else { c };
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
            out.push(c.to_ascii_lowercase());
        } else {
            return Err(GeneratePackError::InvalidPackageName(format!(
                "invalid package name `{raw}` (unsupported character `{c}`)"
            )));
        }
    }

    Ok(out)
}

pub(crate) fn validate_vendor_namespace(raw: &str) -> Result<String, GeneratePackError> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(GeneratePackError::InvalidVendorNamespace(
            "vendor namespace cannot be empty".to_string(),
        ));
    }

    let mut out = String::with_capacity(raw.len());
    for (index, c) in raw.chars().enumerate() {
        let c = c.to_ascii_lowercase();
        let valid = c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_';
        if !valid {
            return Err(GeneratePackError::InvalidVendorNamespace(format!(
                "invalid vendor namespace `{raw}` (unsupported character `{c}`)"
            )));
        }
        if index == 0 && !c.is_ascii_lowercase() {
            return Err(GeneratePackError::InvalidVendorNamespace(format!(
                "invalid vendor namespace `{raw}` (must start with an ASCII letter)"
            )));
        }
        out.push(c);
    }

    Ok(out)
}

pub(crate) fn crate_module_name(package_name: &str) -> String {
    package_name.replace('-', "_")
}

pub(crate) fn ensure_dir_is_new_or_empty(path: &Path) -> Result<(), GeneratePackError> {
    if path.exists() && !path.is_dir() {
        return Err(GeneratePackError::OutputPathNotDirectory(
            path.display().to_string(),
        ));
    }
    if path.exists() {
        let entries = std::fs::read_dir(path)?;
        let mut blocking: Vec<String> = Vec::new();
        for entry in entries {
            let entry = entry?;
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
            let mut message = format!(
                "output directory is not empty: {} (choose another output path)",
                path.display()
            );
            message.push_str(&format!("\n  found: {}", sample.join(", ")));
            if more > 0 {
                message.push_str(&format!(" (+{more} more)"));
            }
            return Err(GeneratePackError::OutputDirectoryNotEmpty(message));
        }
        return Ok(());
    }

    std::fs::create_dir_all(path)?;
    Ok(())
}

pub(crate) fn write_new_file(path: &Path, contents: &str) -> Result<(), GeneratePackError> {
    if path.exists() {
        return Err(GeneratePackError::RefusingToOverwrite(
            path.display().to_string(),
        ));
    }
    std::fs::write(path, contents)?;
    Ok(())
}

pub(crate) fn write_new_bytes(path: &Path, contents: &[u8]) -> Result<(), GeneratePackError> {
    if path.exists() {
        return Err(GeneratePackError::RefusingToOverwrite(
            path.display().to_string(),
        ));
    }
    std::fs::write(path, contents)?;
    Ok(())
}

pub(crate) fn path_label(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}
