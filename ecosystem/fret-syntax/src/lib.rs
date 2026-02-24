//! Syntax highlighting helpers for Fret.
//!
//! This crate vendors a set of Tree-sitter highlight queries and provides a small API to map a
//! source string to highlight spans.
//!
//! The output is intentionally backend-agnostic so higher layers can decide how to paint tokens.
//!
//! Note: for `wasm32` targets we currently disable Tree-sitter highlighting to avoid requiring a
//! WebAssembly-capable C toolchain (Tree-sitter's C core requires `clang` with a wasm backend).

#[cfg(not(target_arch = "wasm32"))]
mod registry;

use std::ops::Range;

pub const HIGHLIGHT_NAMES: &[&str] = &[
    "attribute",
    "boolean",
    "comment",
    "comment.doc",
    "constant",
    "constructor",
    "embedded",
    "emphasis",
    "emphasis.strong",
    "enum",
    "function",
    "hint",
    "keyword",
    "label",
    "link_text",
    "link_uri",
    "number",
    "operator",
    "predictive",
    "preproc",
    "primary",
    "property",
    "punctuation",
    "punctuation.bracket",
    "punctuation.delimiter",
    "punctuation.list_marker",
    "punctuation.special",
    "string",
    "string.escape",
    "string.regex",
    "string.special",
    "string.special.symbol",
    "tag",
    "tag.doctype",
    "text.literal",
    "title",
    "type",
    "variable",
    "variable.special",
    "variant",
    "character",
    "character.special",
    "comment.documentation",
    "comment.unused",
    "constant.builtin",
    "constant.macro",
    "diff.minus",
    "diff.plus",
    "error",
    "field",
    "float",
    "function.builtin",
    "function.call",
    "function.macro",
    "function.method",
    "function.method.call",
    "function.special",
    "import",
    "include",
    "keyword.conditional",
    "keyword.conditional.ternary",
    "keyword.coroutine",
    "keyword.directive",
    "keyword.exception",
    "keyword.function",
    "keyword.import",
    "keyword.modifier",
    "keyword.operator",
    "keyword.repeat",
    "keyword.return",
    "keyword.type",
    "method",
    "method.call",
    "module",
    "module.builtin",
    "namespace",
    "number.float",
    "parameter",
    "property.definition",
    "storageclass",
    "string.documentation",
    "string.regexp",
    "string.special.path",
    "tag.error",
    "text.danger",
    "text.note",
    "text.reference",
    "text.uri",
    "text.warning",
    "type.builtin",
    "type.definition",
    "type.qualifier",
    "variable.builtin",
    "variable.member",
    "variable.parameter",
    "variable.parameter.builtin",
    "none",
];

#[cfg(not(target_arch = "wasm32"))]
pub fn supported_languages() -> &'static [&'static str] {
    registry::supported_languages()
}

#[cfg(target_arch = "wasm32")]
pub fn supported_languages() -> &'static [&'static str] {
    &[]
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// A half-open byte range in the source and an optional highlight name.
pub struct HighlightSpan {
    pub range: Range<usize>,
    pub highlight: Option<&'static str>,
}

#[derive(Debug, thiserror::Error)]
/// Highlighting error.
pub enum HighlightError {
    #[error("unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[cfg(target_arch = "wasm32")]
    #[error("syntax highlighting is not supported on wasm targets")]
    Unavailable,
    #[error("tree-sitter highlight error: {0}")]
    #[cfg(not(target_arch = "wasm32"))]
    Highlight(#[from] tree_sitter_highlight::Error),
}

/// Highlights a source string using the vendored Tree-sitter highlight queries.
///
/// `language` is a short identifier from [`supported_languages`].
pub fn highlight(source: &str, language: &str) -> Result<Vec<HighlightSpan>, HighlightError> {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (source, language);
        return Err(HighlightError::Unavailable);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
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
        let supported: HashSet<&'static str> = crate::HIGHLIGHT_NAMES.iter().copied().collect();

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
