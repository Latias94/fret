use std::sync::Arc;

use crate::clamp_to_char_boundary;

/// A line-local inlay span expressed as a UTF-8 byte offset into the line text.
///
/// v1 constraints (enforced by `validate_inlay_spans`):
/// - Offsets must be within the line text and on UTF-8 char boundaries.
/// - Offsets must be sorted and unique.
/// - This is a view-layer contract only (no policy): interaction and edit behavior are owned by
///   the surface layer (ADR 0185).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlaySpan {
    pub byte: usize,
    pub text: Arc<str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlaySpanError {
    OffsetOutOfBounds { byte: usize, len: usize },
    OffsetNotCharBoundary { byte: usize },
    NotSortedOrOverlapping,
}

pub fn validate_inlay_spans(text: &str, spans: &[InlaySpan]) -> Result<(), InlaySpanError> {
    let len = text.len();
    let mut prev = None::<usize>;
    for span in spans {
        let byte = span.byte;
        if byte > len {
            return Err(InlaySpanError::OffsetOutOfBounds { byte, len });
        }
        let clamped = clamp_to_char_boundary(text, byte);
        if byte != clamped {
            return Err(InlaySpanError::OffsetNotCharBoundary { byte });
        }
        if prev.is_some_and(|p| byte <= p) {
            return Err(InlaySpanError::NotSortedOrOverlapping);
        }
        prev = Some(byte);
    }
    Ok(())
}

/// Apply inlay spans to a line of text, inserting each span's text at its byte offset.
///
/// This is a pure helper (no buffer access). Callers should validate inlay spans first if they
/// need a structured error.
pub fn apply_inlay_spans(text: &str, spans: &[InlaySpan]) -> Result<String, InlaySpanError> {
    validate_inlay_spans(text, spans)?;

    let mut added = 0usize;
    for span in spans {
        added = added.saturating_add(span.text.len());
    }

    let cap = text.len().saturating_add(added).max(1);
    let mut out = String::with_capacity(cap);

    let mut cursor = 0usize;
    for span in spans {
        let byte = span.byte.min(text.len());
        if cursor < byte {
            out.push_str(&text[cursor..byte]);
        }
        out.push_str(span.text.as_ref());
        cursor = byte;
    }
    if cursor < text.len() {
        out.push_str(&text[cursor..]);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_inlay_spans_inserts_text() {
        let text = "abcdef";
        let spans = vec![InlaySpan {
            byte: 1,
            text: Arc::<str>::from("<in>"),
        }];
        assert_eq!(apply_inlay_spans(text, &spans).unwrap(), "a<in>bcdef");
    }

    #[test]
    fn validate_inlay_spans_rejects_unsorted_or_duplicate_offsets() {
        let text = "abcdef";
        let spans = vec![
            InlaySpan {
                byte: 3,
                text: Arc::<str>::from("x"),
            },
            InlaySpan {
                byte: 3,
                text: Arc::<str>::from("y"),
            },
        ];
        assert_eq!(
            validate_inlay_spans(text, &spans),
            Err(InlaySpanError::NotSortedOrOverlapping)
        );
    }
}
