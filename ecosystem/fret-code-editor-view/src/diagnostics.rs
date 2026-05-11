use std::ops::Range;
use std::sync::Arc;

use fret_code_editor_buffer::TextBuffer;

/// Diagnostic severity in editor-display order.
///
/// The derived ordering intentionally treats `Error` as the most severe value and `Hint` as the
/// least severe value. This mirrors the filter/order vocabulary used by editor references such as
/// Zed/LSP without taking a dependency on an LSP crate in the view layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

impl Default for DiagnosticSeverity {
    fn default() -> Self {
        Self::Error
    }
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
                &[DiagnosticSpan::new(2..1, DiagnosticSeverity::Error, "bad")]
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
}
