use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::Range;
use std::sync::Arc;

#[derive(Default)]
pub(crate) struct CodeBlockPreparedState {
    key: CodeBlockKey,
    pub(crate) prepared: Arc<PreparedCodeBlock>,
}

impl CodeBlockPreparedState {
    pub(crate) fn prepare(
        &mut self,
        code: &str,
        language: Option<&str>,
        show_line_numbers: bool,
        mode: CodeBlockPrepareMode,
    ) {
        let key = CodeBlockKey::new(code, language, show_line_numbers, mode);
        if self.key == key {
            return;
        }
        self.key = key;
        self.prepared = Arc::new(prepare_code_block(
            code,
            language,
            show_line_numbers,
            key.revision(),
            mode,
        ));
    }

    pub(crate) fn prepare_arc(
        &mut self,
        code: &Arc<str>,
        language: Option<&str>,
        show_line_numbers: bool,
        mode: CodeBlockPrepareMode,
    ) {
        let key = CodeBlockKey::new_arc(code, language, show_line_numbers, mode);
        if self.key == key {
            return;
        }
        self.key = key;
        self.prepared = Arc::new(prepare_code_block(
            code.as_ref(),
            language,
            show_line_numbers,
            key.revision(),
            mode,
        ));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct CodeBlockKey {
    code: CodeBlockCodeKey,
    language_hash: u64,
    language_len: usize,
    show_line_numbers: bool,
    mode: CodeBlockPrepareMode,
}

impl CodeBlockKey {
    fn new(
        code: &str,
        language: Option<&str>,
        show_line_numbers: bool,
        mode: CodeBlockPrepareMode,
    ) -> Self {
        let language = language.unwrap_or("");
        Self {
            code: CodeBlockCodeKey::Borrowed {
                hash: hash_value(code),
                len: code.len(),
            },
            language_hash: hash_value(language),
            language_len: language.len(),
            show_line_numbers,
            mode,
        }
    }

    fn new_arc(
        code: &Arc<str>,
        language: Option<&str>,
        show_line_numbers: bool,
        mode: CodeBlockPrepareMode,
    ) -> Self {
        let language = language.unwrap_or("");
        Self {
            code: CodeBlockCodeKey::Shared {
                ptr: Arc::as_ptr(code) as *const () as usize,
                len: code.len(),
            },
            language_hash: hash_value(language),
            language_len: language.len(),
            show_line_numbers,
            mode,
        }
    }

    fn revision(&self) -> u64 {
        let mut h = DefaultHasher::new();
        self.hash(&mut h);
        h.finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum CodeBlockCodeKey {
    Borrowed { hash: u64, len: usize },
    Shared { ptr: usize, len: usize },
}

impl Default for CodeBlockCodeKey {
    fn default() -> Self {
        Self::Borrowed { hash: 0, len: 0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) enum CodeBlockPrepareMode {
    #[default]
    Full,
    LineIndexed,
}

#[derive(Debug, Clone)]
pub(crate) struct PreparedCodeBlock {
    pub(crate) revision: u64,
    pub(crate) show_line_numbers: bool,
    pub(crate) line_number_width: usize,
    pub(crate) max_line_columns: usize,
    pub(crate) syntax_highlights: Vec<&'static str>,
    pub(crate) lines: Vec<PreparedLine>,
}

impl Default for PreparedCodeBlock {
    fn default() -> Self {
        Self {
            revision: 0,
            show_line_numbers: false,
            line_number_width: 0,
            max_line_columns: 0,
            syntax_highlights: Vec::new(),
            lines: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct PreparedLine {
    pub(crate) segments: Vec<PreparedSegment>,
}

#[derive(Debug, Clone)]
pub(crate) struct PreparedSegment {
    pub(crate) text: Arc<str>,
    pub(crate) highlight: Option<&'static str>,
}

fn prepare_code_block(
    code: &str,
    language: Option<&str>,
    show_line_numbers: bool,
    revision: u64,
    mode: CodeBlockPrepareMode,
) -> PreparedCodeBlock {
    let mut lines = split_lines(code);
    let line_number_width = line_number_width(lines.len());

    let max_line_columns = lines
        .iter()
        .map(|line| estimated_line_columns(line.text))
        .max()
        .unwrap_or(0);
    let mut prepared_lines = Vec::with_capacity(lines.len());
    let mut syntax_highlights: Vec<&'static str> = Vec::new();
    let mut span_i = 0usize;

    let spans = if mode == CodeBlockPrepareMode::Full {
        match language {
            Some(language) => normalize_spans_to_char_boundaries(
                code,
                fret_syntax::highlight(code, language).unwrap_or_default(),
            ),
            None => Vec::new(),
        }
    } else {
        Vec::new()
    };

    for line in &mut lines {
        let global_range = line.range.clone();

        if mode == CodeBlockPrepareMode::LineIndexed {
            prepared_lines.push(plain_prepared_line(line.text));
            continue;
        }

        let line_text = line.text;

        while span_i < spans.len() && spans[span_i].range.end <= global_range.start {
            span_i += 1;
        }

        let mut segments: Vec<(String, Option<&'static str>)> = Vec::new();
        let mut cursor = global_range.start;
        let mut j = span_i;
        while j < spans.len() {
            let span = &spans[j];
            if span.range.start >= global_range.end {
                break;
            }
            let start = span.range.start.max(global_range.start);
            let end = span.range.end.min(global_range.end);
            if cursor < start {
                let rel = cursor - global_range.start;
                let rel_end = start - global_range.start;
                segments.push((safe_slice(line_text, rel, rel_end), None));
            }
            let rel = start - global_range.start;
            let rel_end = end - global_range.start;
            if let Some(highlight) = span.highlight
                && !syntax_highlights.contains(&highlight)
            {
                syntax_highlights.push(highlight);
            }
            segments.push((safe_slice(line_text, rel, rel_end), span.highlight));
            cursor = end;
            j += 1;
        }
        if cursor < global_range.end {
            let rel = cursor - global_range.start;
            let rel_end = global_range.end - global_range.start;
            segments.push((safe_slice(line_text, rel, rel_end), None));
        }

        if segments.is_empty() {
            segments.push((line_text.to_string(), None));
        }

        let segments = coalesce_segments(segments)
            .into_iter()
            .map(|(text, highlight)| PreparedSegment {
                text: Arc::<str>::from(text),
                highlight,
            })
            .collect();

        prepared_lines.push(PreparedLine { segments });
    }

    PreparedCodeBlock {
        revision,
        show_line_numbers,
        line_number_width,
        max_line_columns,
        syntax_highlights,
        lines: prepared_lines,
    }
}

fn estimated_line_columns(text: &str) -> usize {
    text.chars()
        .map(|ch| {
            if ch == '\t' {
                4
            } else if ch.is_ascii() {
                1
            } else {
                2
            }
        })
        .sum()
}

fn coalesce_segments(
    segments: Vec<(String, Option<&'static str>)>,
) -> Vec<(String, Option<&'static str>)> {
    let mut out: Vec<(String, Option<&'static str>)> = Vec::with_capacity(segments.len());
    for (text, highlight) in segments {
        if text.is_empty() {
            continue;
        }
        if let Some((last_text, last_highlight)) = out.last_mut()
            && *last_highlight == highlight
        {
            last_text.push_str(&text);
            continue;
        }
        out.push((text, highlight));
    }
    out
}

fn hash_value(value: &str) -> u64 {
    let mut h = DefaultHasher::new();
    value.hash(&mut h);
    h.finish()
}

fn safe_slice(text: &str, start: usize, end: usize) -> String {
    if start >= end {
        return String::new();
    }
    if start >= text.len() {
        return String::new();
    }
    let end = end.min(text.len());
    match text.get(start..end) {
        Some(s) => s.to_string(),
        None => {
            debug_assert!(false, "code slice is not aligned to UTF-8 boundaries");
            String::from_utf8_lossy(&text.as_bytes()[start..end]).into_owned()
        }
    }
}

fn plain_prepared_line(line_text: &str) -> PreparedLine {
    PreparedLine {
        segments: vec![PreparedSegment {
            text: Arc::<str>::from(line_text),
            highlight: None,
        }],
    }
}

fn clamp_down_to_char_boundary(text: &str, mut index: usize) -> usize {
    index = index.min(text.len());
    while index > 0 && !text.is_char_boundary(index) {
        index = index.saturating_sub(1);
    }
    index
}

fn clamp_up_to_char_boundary(text: &str, mut index: usize) -> usize {
    index = index.min(text.len());
    while index < text.len() && !text.is_char_boundary(index) {
        index = index.saturating_add(1);
    }
    index
}

fn normalize_spans_to_char_boundaries(
    text: &str,
    spans: Vec<fret_syntax::HighlightSpan>,
) -> Vec<fret_syntax::HighlightSpan> {
    let mut out = Vec::with_capacity(spans.len());
    let mut cursor = 0usize;

    for span in spans {
        let mut start = clamp_down_to_char_boundary(text, span.range.start);
        let end = clamp_up_to_char_boundary(text, span.range.end);

        if start < cursor {
            start = cursor;
        }
        if end <= start {
            continue;
        }

        cursor = end;
        out.push(fret_syntax::HighlightSpan {
            range: start..end,
            highlight: span.highlight,
        });
    }

    out
}

#[derive(Debug, Clone)]
struct LineSlice<'a> {
    range: Range<usize>,
    text: &'a str,
}

fn split_lines(text: &str) -> Vec<LineSlice<'_>> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            let mut end = i;
            if end > start && bytes[end - 1] == b'\r' {
                end -= 1;
            }
            out.push(LineSlice {
                range: start..end,
                text: &text[start..end],
            });
            start = i + 1;
        }
        i += 1;
    }
    out.push(LineSlice {
        range: start..text.len(),
        text: &text[start..],
    });
    out
}

fn line_number_width(lines: usize) -> usize {
    let mut n = lines.max(1);
    let mut digits = 0usize;
    while n > 0 {
        digits += 1;
        n /= 10;
    }
    digits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coalesces_adjacent_segments() {
        let segments = vec![
            ("a".to_string(), None),
            ("b".to_string(), None),
            ("c".to_string(), Some("keyword")),
            ("d".to_string(), Some("keyword")),
            ("".to_string(), Some("keyword")),
            ("e".to_string(), None),
        ];
        let out = coalesce_segments(segments);
        assert_eq!(
            out,
            vec![
                ("ab".to_string(), None),
                ("cd".to_string(), Some("keyword")),
                ("e".to_string(), None)
            ]
        );
    }

    #[test]
    fn splits_crlf_lines_without_carriage_returns() {
        let lines = split_lines("a\r\nb\r\n");
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].text, "a");
        assert_eq!(lines[1].text, "b");
        assert_eq!(lines[2].text, "");
    }

    #[test]
    fn normalizes_spans_to_utf8_boundaries() {
        let text = "a🦀b";
        let spans = vec![fret_syntax::HighlightSpan {
            range: 2..3,
            highlight: Some("keyword"),
        }];
        let out = normalize_spans_to_char_boundaries(text, spans);
        assert_eq!(out.len(), 1);
        assert!(text.is_char_boundary(out[0].range.start));
        assert!(text.is_char_boundary(out[0].range.end));
    }

    #[test]
    fn prepared_state_is_idempotent_for_identical_inputs() {
        let mut state = CodeBlockPreparedState::default();
        state.prepare(
            "fn main() {}",
            Some("rust"),
            true,
            CodeBlockPrepareMode::Full,
        );
        let first = state.prepared.clone();
        let first_revision = first.revision;

        state.prepare(
            "fn main() {}",
            Some("rust"),
            true,
            CodeBlockPrepareMode::Full,
        );

        assert!(
            Arc::ptr_eq(&state.prepared, &first),
            "identical code block inputs must not rebuild prepared state"
        );
        assert_eq!(state.prepared.revision, first_revision);
    }

    #[test]
    fn prepared_state_rebuilds_when_inputs_change() {
        let mut state = CodeBlockPreparedState::default();
        state.prepare(
            "fn main() {}",
            Some("rust"),
            true,
            CodeBlockPrepareMode::Full,
        );
        let first = state.prepared.clone();

        state.prepare(
            "fn main() {}",
            Some("rust"),
            false,
            CodeBlockPrepareMode::Full,
        );

        assert!(
            !Arc::ptr_eq(&state.prepared, &first),
            "changed code block inputs must rebuild prepared state"
        );
        assert_ne!(state.prepared.revision, first.revision);
    }

    #[test]
    fn line_indexed_prepare_keeps_plain_line_index_for_windowed_rows() {
        let mut state = CodeBlockPreparedState::default();
        state.prepare(
            "fn main() {}\nlet x = 1;",
            Some("rust"),
            true,
            CodeBlockPrepareMode::LineIndexed,
        );

        let prepared = state.prepared;
        assert_eq!(prepared.lines.len(), 2);
        assert_eq!(prepared.lines[0].segments.len(), 1);
        assert_eq!(prepared.lines[0].segments[0].text.as_ref(), "fn main() {}");
        assert_eq!(prepared.lines[0].segments[0].highlight, None);
        assert!(prepared.syntax_highlights.is_empty());
    }

    #[test]
    fn prepare_records_max_line_columns_for_windowed_extent_estimates() {
        let mut state = CodeBlockPreparedState::default();
        state.prepare(
            "a\nlet wide = 12345;\n\t中\n",
            Some("rust"),
            true,
            CodeBlockPrepareMode::LineIndexed,
        );

        assert_eq!(
            state.prepared.max_line_columns,
            "let wide = 12345;".chars().count()
        );
    }

    #[test]
    fn prepare_arc_reuses_the_shared_source_identity() {
        let code = Arc::<str>::from("fn main() {}\nlet x = 1;");
        let mut state = CodeBlockPreparedState::default();
        state.prepare_arc(&code, Some("rust"), true, CodeBlockPrepareMode::LineIndexed);
        let first = state.prepared.clone();

        state.prepare_arc(&code, Some("rust"), true, CodeBlockPrepareMode::LineIndexed);

        assert!(
            Arc::ptr_eq(&state.prepared, &first),
            "shared Arc input should short-circuit repeated preparation"
        );
    }
}
