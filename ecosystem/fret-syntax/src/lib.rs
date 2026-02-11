mod registry;

use std::ops::Range;

pub use registry::HIGHLIGHT_NAMES;
pub use registry::supported_languages;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightSpan {
    pub range: Range<usize>,
    pub highlight: Option<&'static str>,
}

#[derive(Debug, thiserror::Error)]
pub enum HighlightError {
    #[error("unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("tree-sitter highlight error: {0}")]
    Highlight(#[from] tree_sitter_highlight::Error),
}

pub fn highlight(source: &str, language: &str) -> Result<Vec<HighlightSpan>, HighlightError> {
    let Some(config) = registry::config_for(language) else {
        return Err(HighlightError::UnsupportedLanguage(language.to_string()));
    };

    registry::reset_active_highlights();

    let mut highlighter = tree_sitter_highlight::Highlighter::new();
    let mut spans = Vec::new();

    let events = highlighter.highlight(config, source.as_bytes(), None, |language| {
        registry::config_for(language)
    })?;
    for event in events {
        match event? {
            tree_sitter_highlight::HighlightEvent::Source { start, end } => {
                if start >= end {
                    continue;
                }
                let highlight = registry::active_highlight_name();
                if matches!(highlight, Some("none")) {
                    continue;
                }
                if highlight.is_some() {
                    spans.push(HighlightSpan {
                        range: start..end,
                        highlight,
                    });
                }
            }
            tree_sitter_highlight::HighlightEvent::HighlightStart(h) => {
                registry::push_active_highlight(h);
            }
            tree_sitter_highlight::HighlightEvent::HighlightEnd => {
                registry::pop_active_highlight();
            }
        }
    }

    Ok(spans)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashSet};
    use std::fs;
    use std::path::{Path, PathBuf};

    fn strip_comments_and_strings(line: &str) -> String {
        let mut out = String::with_capacity(line.len());
        let mut in_string = false;
        let mut escape = false;

        for ch in line.chars() {
            if in_string {
                if escape {
                    escape = false;
                    out.push(' ');
                    continue;
                }
                if ch == '\\' {
                    escape = true;
                    out.push(' ');
                    continue;
                }
                if ch == '"' {
                    in_string = false;
                }
                out.push(' ');
                continue;
            }

            if ch == '"' {
                in_string = true;
                out.push(' ');
                continue;
            }

            if ch == ';' {
                break;
            }

            out.push(ch);
        }

        out
    }

    fn is_capture_char(b: u8) -> bool {
        b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-')
    }

    fn extract_captures(line: &str, out: &mut BTreeSet<String>) {
        let bytes = line.as_bytes();
        let mut i = 0usize;
        while i < bytes.len() {
            if bytes[i] != b'@' {
                i += 1;
                continue;
            }

            let start = i + 1;
            let mut end = start;
            while end < bytes.len() && is_capture_char(bytes[end]) {
                end += 1;
            }

            if start < end {
                out.insert(String::from_utf8_lossy(&bytes[start..end]).into_owned());
            }

            i = end;
        }
    }

    fn visit_dir(dir: &Path, files: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                visit_dir(&path, files);
            } else if path
                .file_name()
                .is_some_and(|name| name == "highlights.scm")
            {
                files.push(path);
            }
        }
    }

    #[test]
    fn highlight_query_captures_are_supported() {
        let supported: HashSet<&'static str> =
            crate::registry::HIGHLIGHT_NAMES.iter().copied().collect();

        let languages_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("languages");
        let mut files = Vec::new();
        visit_dir(&languages_dir, &mut files);

        let mut captures = BTreeSet::new();
        for file in files {
            let Ok(contents) = fs::read_to_string(&file) else {
                continue;
            };
            for line in contents.lines() {
                let line = strip_comments_and_strings(line);
                extract_captures(&line, &mut captures);
            }
        }

        let mut missing = BTreeSet::new();
        for capture in captures {
            if capture.starts_with('_') {
                continue;
            }
            if capture == "spell" || capture == "nospell" {
                continue;
            }
            if capture.starts_with("injection.") {
                continue;
            }
            if !supported.contains(capture.as_str()) {
                missing.insert(capture);
            }
        }

        assert!(
            missing.is_empty(),
            "unsupported highlight captures in vendored queries: {missing:?}"
        );
    }
}
