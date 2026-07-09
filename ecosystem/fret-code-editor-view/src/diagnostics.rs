use std::collections::BTreeMap;
use std::ops::Range;
use std::sync::Arc;

use fret_code_editor_buffer::TextBuffer;

/// Diagnostic severity in editor-display order.
///
/// The derived ordering intentionally treats `Error` as the most severe value and `Hint` as the
/// least severe value. This mirrors the filter/order vocabulary used by editor references such as
/// Zed/LSP without taking a dependency on an LSP crate in the view layer.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticSeverity {
    #[default]
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DiagnosticSourceKind {
    Pulled,
    Pushed,
    #[default]
    Other,
}

/// A buffer-local diagnostic span expressed in UTF-8 byte indices.
///
/// v1 constraints:
/// - Ranges are buffer byte ranges, not display rows and not UTF-16 ranges.
/// - Empty ranges are valid; many language servers use point diagnostics.
/// - Ranges must be within the buffer and on UTF-8 char boundaries.
/// - Overlaps are valid because different sources/severities can annotate the same text.
/// - Presentation policy (colors, gutter icons, hover, code actions) is owned by higher layers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticSpan {
    pub range: Range<usize>,
    pub severity: DiagnosticSeverity,
    pub message: Arc<str>,
    pub source: Option<Arc<str>>,
    pub code: Option<Arc<str>>,
    pub source_kind: DiagnosticSourceKind,
    pub group_id: Option<u64>,
    pub is_primary: bool,
    pub is_unnecessary: bool,
    pub is_deprecated: bool,
    pub underline: bool,
}

impl DiagnosticSpan {
    pub fn new(
        range: Range<usize>,
        severity: DiagnosticSeverity,
        message: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            range,
            severity,
            message: message.into(),
            source: None,
            code: None,
            source_kind: DiagnosticSourceKind::Other,
            group_id: None,
            is_primary: true,
            is_unnecessary: false,
            is_deprecated: false,
            underline: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticSpanError {
    RangeStartAfterEnd,
    RangeOutOfBounds {
        start: usize,
        end: usize,
        len: usize,
    },
    RangeNotCharBoundary {
        start: usize,
        end: usize,
    },
}

/// Logical-line diagnostic aggregate for gutter and overview consumers.
///
/// v1 constraints:
/// - `line` is a `TextBuffer` logical line index, not a display row.
/// - Non-empty ranges use half-open byte-range semantics: `start..end` covers the line containing
///   the previous character boundary before `end`.
/// - This is a data projection only; it does not choose colors, icons, hover text, or actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticLineSummary {
    pub line: usize,
    pub count: usize,
    pub most_severe: DiagnosticSeverity,
    pub has_primary: bool,
    pub has_unnecessary: bool,
    pub has_deprecated: bool,
    pub has_underline: bool,
}

impl DiagnosticLineSummary {
    fn from_span(line: usize, span: &DiagnosticSpan) -> Self {
        Self {
            line,
            count: 1,
            most_severe: span.severity,
            has_primary: span.is_primary,
            has_unnecessary: span.is_unnecessary,
            has_deprecated: span.is_deprecated,
            has_underline: span.underline,
        }
    }

    fn add_span(&mut self, span: &DiagnosticSpan) {
        self.count = self.count.saturating_add(1);
        if span.severity < self.most_severe {
            self.most_severe = span.severity;
        }
        self.has_primary |= span.is_primary;
        self.has_unnecessary |= span.is_unnecessary;
        self.has_deprecated |= span.is_deprecated;
        self.has_underline |= span.underline;
    }
}

pub fn validate_diagnostic_spans(
    buf: &TextBuffer,
    spans: &[DiagnosticSpan],
) -> Result<(), DiagnosticSpanError> {
    let len = buf.len_bytes();
    for span in spans {
        let start = span.range.start;
        let end = span.range.end;
        if start > end {
            return Err(DiagnosticSpanError::RangeStartAfterEnd);
        }
        if start > len || end > len {
            return Err(DiagnosticSpanError::RangeOutOfBounds { start, end, len });
        }
        if !buf.is_char_boundary(start) || !buf.is_char_boundary(end) {
            return Err(DiagnosticSpanError::RangeNotCharBoundary { start, end });
        }
    }
    Ok(())
}

pub fn normalized_diagnostic_spans(
    buf: &TextBuffer,
    spans: &[DiagnosticSpan],
) -> Result<Vec<DiagnosticSpan>, DiagnosticSpanError> {
    validate_diagnostic_spans(buf, spans)?;

    let mut out = spans.to_vec();
    out.sort_by(|a, b| {
        a.range
            .start
            .cmp(&b.range.start)
            .then(a.range.end.cmp(&b.range.end))
            .then(a.severity.cmp(&b.severity))
            .then_with(|| a.source.as_deref().cmp(&b.source.as_deref()))
            .then_with(|| a.code.as_deref().cmp(&b.code.as_deref()))
            .then_with(|| a.message.as_ref().cmp(b.message.as_ref()))
    });
    Ok(out)
}

pub fn diagnostic_line_summaries(
    buf: &TextBuffer,
    spans: &[DiagnosticSpan],
) -> Result<Vec<DiagnosticLineSummary>, DiagnosticSpanError> {
    validate_diagnostic_spans(buf, spans)?;

    let mut summaries = BTreeMap::<usize, DiagnosticLineSummary>::new();
    for span in spans {
        let start_line = buf.line_index_at_byte(span.range.start);
        let end_line = diagnostic_span_end_line(buf, &span.range);

        for line in start_line..=end_line {
            summaries
                .entry(line)
                .and_modify(|summary| summary.add_span(span))
                .or_insert_with(|| DiagnosticLineSummary::from_span(line, span));
        }
    }

    Ok(summaries.into_values().collect())
}

fn diagnostic_span_end_line(buf: &TextBuffer, range: &Range<usize>) -> usize {
    if range.start == range.end {
        return buf.line_index_at_byte(range.start);
    }

    buf.line_index_at_byte(buf.prev_char_boundary(range.end))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_code_editor_buffer::DocId;

    fn buffer(text: &str) -> TextBuffer {
        TextBuffer::new(DocId::new(), text.to_string()).unwrap()
    }

    #[test]
    fn diagnostic_spans_allow_empty_and_overlapping_ranges() {
        let buf = buffer("abcdef");
        let spans = vec![
            DiagnosticSpan::new(1..4, DiagnosticSeverity::Warning, "warning"),
            DiagnosticSpan::new(2..2, DiagnosticSeverity::Error, "point"),
            DiagnosticSpan::new(3..5, DiagnosticSeverity::Hint, "overlap"),
        ];

        assert_eq!(validate_diagnostic_spans(&buf, &spans), Ok(()));
    }

    #[test]
    fn diagnostic_spans_reject_out_of_bounds_and_reversed_ranges() {
        let buf = buffer("abc");

        assert_eq!(
            validate_diagnostic_spans(
                &buf,
                &[DiagnosticSpan::new(
                    Range { start: 2, end: 1 },
                    DiagnosticSeverity::Error,
                    "bad"
                )]
            ),
            Err(DiagnosticSpanError::RangeStartAfterEnd)
        );
        assert_eq!(
            validate_diagnostic_spans(
                &buf,
                &[DiagnosticSpan::new(0..4, DiagnosticSeverity::Error, "bad")]
            ),
            Err(DiagnosticSpanError::RangeOutOfBounds {
                start: 0,
                end: 4,
                len: 3
            })
        );
    }

    #[test]
    fn diagnostic_spans_reject_non_char_boundaries() {
        let buf = buffer("aé");

        assert_eq!(
            validate_diagnostic_spans(
                &buf,
                &[DiagnosticSpan::new(2..3, DiagnosticSeverity::Error, "bad")]
            ),
            Err(DiagnosticSpanError::RangeNotCharBoundary { start: 2, end: 3 })
        );
    }

    #[test]
    fn normalized_diagnostic_spans_sorts_deterministically_without_dropping_overlaps() {
        let buf = buffer("abcdef");
        let spans = vec![
            DiagnosticSpan::new(3..5, DiagnosticSeverity::Hint, "c"),
            DiagnosticSpan::new(1..4, DiagnosticSeverity::Warning, "b"),
            DiagnosticSpan::new(1..4, DiagnosticSeverity::Error, "a"),
        ];

        let normalized = normalized_diagnostic_spans(&buf, &spans).unwrap();
        let keys = normalized
            .iter()
            .map(|span| (span.range.clone(), span.severity, span.message.to_string()))
            .collect::<Vec<_>>();

        assert_eq!(
            keys,
            vec![
                (1..4, DiagnosticSeverity::Error, "a".to_string()),
                (1..4, DiagnosticSeverity::Warning, "b".to_string()),
                (3..5, DiagnosticSeverity::Hint, "c".to_string()),
            ]
        );
    }

    #[test]
    fn diagnostic_line_summaries_map_point_diagnostics_to_their_line() {
        let buf = buffer("alpha\nbeta\ngamma");
        let spans = vec![DiagnosticSpan::new(
            6..6,
            DiagnosticSeverity::Warning,
            "point",
        )];

        let summaries = diagnostic_line_summaries(&buf, &spans).unwrap();

        assert_eq!(
            summaries,
            vec![DiagnosticLineSummary {
                line: 1,
                count: 1,
                most_severe: DiagnosticSeverity::Warning,
                has_primary: true,
                has_unnecessary: false,
                has_deprecated: false,
                has_underline: true,
            }]
        );
    }

    #[test]
    fn diagnostic_line_summaries_cover_all_touched_lines() {
        let buf = buffer("aa\nbb\ncc");
        let spans = vec![DiagnosticSpan::new(
            1..7,
            DiagnosticSeverity::Information,
            "multi",
        )];

        let summaries = diagnostic_line_summaries(&buf, &spans).unwrap();
        let lines = summaries
            .iter()
            .map(|summary| (summary.line, summary.count, summary.most_severe))
            .collect::<Vec<_>>();

        assert_eq!(
            lines,
            vec![
                (0, 1, DiagnosticSeverity::Information),
                (1, 1, DiagnosticSeverity::Information),
                (2, 1, DiagnosticSeverity::Information),
            ]
        );
    }

    #[test]
    fn diagnostic_line_summaries_treat_range_end_as_half_open() {
        let buf = buffer("aa\nbb\ncc");
        let spans = vec![DiagnosticSpan::new(
            0..3,
            DiagnosticSeverity::Error,
            "line0",
        )];

        let summaries = diagnostic_line_summaries(&buf, &spans).unwrap();
        let lines = summaries
            .iter()
            .map(|summary| summary.line)
            .collect::<Vec<_>>();

        assert_eq!(lines, vec![0]);
    }

    #[test]
    fn diagnostic_line_summaries_count_overlaps_and_keep_most_severe() {
        let buf = buffer("abcdef");
        let mut deprecated = DiagnosticSpan::new(1..4, DiagnosticSeverity::Hint, "deprecated");
        deprecated.is_primary = false;
        deprecated.is_deprecated = true;

        let mut unnecessary = DiagnosticSpan::new(2..3, DiagnosticSeverity::Warning, "unnecessary");
        unnecessary.is_unnecessary = true;
        unnecessary.underline = false;

        let spans = vec![
            deprecated,
            unnecessary,
            DiagnosticSpan::new(3..5, DiagnosticSeverity::Error, "error"),
        ];

        let summaries = diagnostic_line_summaries(&buf, &spans).unwrap();

        assert_eq!(
            summaries,
            vec![DiagnosticLineSummary {
                line: 0,
                count: 3,
                most_severe: DiagnosticSeverity::Error,
                has_primary: true,
                has_unnecessary: true,
                has_deprecated: true,
                has_underline: true,
            }]
        );
    }

    #[test]
    fn diagnostic_line_summaries_bubble_validation_errors() {
        let buf = buffer("aé");

        assert_eq!(
            diagnostic_line_summaries(
                &buf,
                &[DiagnosticSpan::new(2..3, DiagnosticSeverity::Error, "bad")]
            ),
            Err(DiagnosticSpanError::RangeNotCharBoundary { start: 2, end: 3 })
        );
    }
}
