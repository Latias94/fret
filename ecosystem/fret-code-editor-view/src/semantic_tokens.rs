use std::ops::Range;
use std::sync::Arc;

use fret_code_editor_buffer::TextBuffer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticToken {
    pub range: Range<usize>,
    pub class: Arc<str>,
    pub modifiers: Vec<Arc<str>>,
}

impl SemanticToken {
    pub fn new(range: Range<usize>, class: impl Into<Arc<str>>) -> Self {
        Self {
            range,
            class: class.into(),
            modifiers: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticTokenError {
    RangeStartAfterEnd,
    EmptyRange {
        at: usize,
    },
    RangeOutOfBounds {
        start: usize,
        end: usize,
        len: usize,
    },
    RangeNotCharBoundary {
        start: usize,
        end: usize,
    },
    EmptyClass,
    EmptyModifier,
}

pub fn validate_semantic_tokens(
    buf: &TextBuffer,
    tokens: &[SemanticToken],
) -> Result<(), SemanticTokenError> {
    let len = buf.len_bytes();
    for token in tokens {
        let start = token.range.start;
        let end = token.range.end;
        if start > end {
            return Err(SemanticTokenError::RangeStartAfterEnd);
        }
        if start == end {
            return Err(SemanticTokenError::EmptyRange { at: start });
        }
        if start > len || end > len {
            return Err(SemanticTokenError::RangeOutOfBounds { start, end, len });
        }
        if !buf.is_char_boundary(start) || !buf.is_char_boundary(end) {
            return Err(SemanticTokenError::RangeNotCharBoundary { start, end });
        }
        if token.class.is_empty() {
            return Err(SemanticTokenError::EmptyClass);
        }
        if token.modifiers.iter().any(|modifier| modifier.is_empty()) {
            return Err(SemanticTokenError::EmptyModifier);
        }
    }

    Ok(())
}

pub fn normalized_semantic_tokens(
    buf: &TextBuffer,
    tokens: &[SemanticToken],
) -> Result<Vec<SemanticToken>, SemanticTokenError> {
    validate_semantic_tokens(buf, tokens)?;

    let mut out = tokens.to_vec();
    for token in &mut out {
        token.modifiers.sort();
        token.modifiers.dedup();
    }
    out.sort_by(|a, b| {
        a.range
            .start
            .cmp(&b.range.start)
            .then(a.range.end.cmp(&b.range.end))
            .then(a.class.cmp(&b.class))
            .then(a.modifiers.cmp(&b.modifiers))
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
    fn semantic_tokens_allow_overlaps_with_semantic_classes_and_modifiers() {
        let buf = buffer("abcdef");
        let mut token = SemanticToken::new(1..4, "function");
        token.modifiers = vec!["declaration".into(), "async".into()];
        let tokens = vec![token, SemanticToken::new(3..5, "variable")];

        assert_eq!(validate_semantic_tokens(&buf, &tokens), Ok(()));
    }

    #[test]
    fn semantic_tokens_reject_reversed_empty_and_out_of_bounds_ranges() {
        let buf = buffer("abc");

        assert_eq!(
            validate_semantic_tokens(
                &buf,
                &[SemanticToken::new(Range { start: 2, end: 1 }, "bad")]
            ),
            Err(SemanticTokenError::RangeStartAfterEnd)
        );
        assert_eq!(
            validate_semantic_tokens(&buf, &[SemanticToken::new(1..1, "bad")]),
            Err(SemanticTokenError::EmptyRange { at: 1 })
        );
        assert_eq!(
            validate_semantic_tokens(&buf, &[SemanticToken::new(0..4, "bad")]),
            Err(SemanticTokenError::RangeOutOfBounds {
                start: 0,
                end: 4,
                len: 3
            })
        );
    }

    #[test]
    fn semantic_tokens_reject_non_char_boundaries() {
        let buf = buffer("aé");

        assert_eq!(
            validate_semantic_tokens(&buf, &[SemanticToken::new(2..3, "bad")]),
            Err(SemanticTokenError::RangeNotCharBoundary { start: 2, end: 3 })
        );
    }

    #[test]
    fn semantic_tokens_reject_empty_class_and_modifier() {
        let buf = buffer("abc");
        let mut token = SemanticToken::new(0..1, "keyword");
        token.modifiers = vec!["".into()];

        assert_eq!(
            validate_semantic_tokens(&buf, &[SemanticToken::new(0..1, "")]),
            Err(SemanticTokenError::EmptyClass)
        );
        assert_eq!(
            validate_semantic_tokens(&buf, &[token]),
            Err(SemanticTokenError::EmptyModifier)
        );
    }

    #[test]
    fn normalized_semantic_tokens_sort_tokens_and_dedupe_modifiers() {
        let buf = buffer("abcdef");
        let mut keyword = SemanticToken::new(0..2, "keyword");
        keyword.modifiers = vec!["unsafe".into(), "unsafe".into(), "declaration".into()];

        let mut function = SemanticToken::new(0..2, "function");
        function.modifiers = vec!["async".into()];

        let normalized =
            normalized_semantic_tokens(&buf, &[function.clone(), keyword.clone()]).unwrap();

        let mut expected_keyword = keyword;
        expected_keyword.modifiers = vec!["declaration".into(), "unsafe".into()];

        assert_eq!(normalized, vec![function, expected_keyword]);
    }

    #[test]
    fn semantic_tokens_do_not_carry_paint_colors() {
        let token = SemanticToken::new(0..3, "type");

        assert_eq!(token.class.as_ref(), "type");
        assert!(token.modifiers.is_empty());
    }
}
