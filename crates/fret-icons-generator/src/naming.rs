use std::path::Path;

use crate::GeneratePackError;

pub(crate) fn normalize_const_name(icon_name: &str) -> String {
    let mut out = String::with_capacity(icon_name.len());
    let mut prev_was_sep = false;

    for ch in icon_name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_uppercase());
            prev_was_sep = false;
        } else if !prev_was_sep && !out.is_empty() {
            out.push('_');
            prev_was_sep = true;
        }
    }

    while out.ends_with('_') {
        out.pop();
    }

    if out.is_empty() {
        out.push_str("ICON");
    }
    if out.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        out.insert_str(0, "ICON_");
    }
    out
}

pub(crate) fn normalize_module_name(namespace: &str) -> String {
    let mut out = String::with_capacity(namespace.len());
    for ch in namespace.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    if out.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        out.insert(0, '_');
    }
    out
}

pub(crate) fn normalize_icon_name(raw: &str) -> Result<String, GeneratePackError> {
    let normalized = normalize_icon_segment(raw);
    if normalized.is_empty() {
        return Err(GeneratePackError::InvalidIconName(format!(
            "failed to derive an icon name from `{raw}`"
        )));
    }
    Ok(normalized)
}

pub(crate) fn normalize_svg_icon_name(relative_path: &Path) -> Result<String, GeneratePackError> {
    let mut segments = Vec::new();
    let mut components = relative_path.components().peekable();
    while let Some(component) = components.next() {
        let raw = if components.peek().is_none() {
            Path::new(component.as_os_str())
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or_default()
                .to_string()
        } else {
            component.as_os_str().to_string_lossy().into_owned()
        };
        let segment = normalize_icon_segment(&raw);
        if !segment.is_empty() {
            segments.push(segment);
        }
    }

    segments.retain(|segment| !segment.is_empty());
    if segments.is_empty() {
        return Err(GeneratePackError::InvalidIconName(format!(
            "failed to derive an icon name from `{}`",
            relative_path.display()
        )));
    }
    Ok(segments.join("-"))
}

fn normalize_icon_segment(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut prev_was_sep = false;
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_was_sep = false;
        } else if !prev_was_sep && !out.is_empty() {
            out.push('-');
            prev_was_sep = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{normalize_const_name, normalize_icon_name, normalize_svg_icon_name};
    use std::path::Path;

    #[test]
    fn const_names_match_existing_icon_codegen_shape() {
        assert_eq!(normalize_const_name("arrow-left"), "ARROW_LEFT");
        assert_eq!(normalize_const_name("123-alert"), "ICON_123_ALERT");
        assert_eq!(normalize_const_name(""), "ICON");
    }

    #[test]
    fn nested_svg_relative_paths_flatten_to_stable_icon_names() {
        let icon_name = normalize_svg_icon_name(Path::new("actions/search.svg"))
            .expect("icon name should normalize");
        assert_eq!(icon_name, "actions-search");

        let icon_name = normalize_svg_icon_name(Path::new("User Avatar.svg"))
            .expect("icon name should normalize");
        assert_eq!(icon_name, "user-avatar");
    }

    #[test]
    fn raw_icon_names_normalize_like_vendor_ids() {
        let icon_name = normalize_icon_name("Arrow Left").expect("icon name should normalize");
        assert_eq!(icon_name, "arrow-left");
    }
}
