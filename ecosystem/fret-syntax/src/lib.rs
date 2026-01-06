mod registry;

use std::ops::Range;

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
