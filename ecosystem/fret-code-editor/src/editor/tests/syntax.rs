use super::*;
#[cfg(feature = "syntax-rust")]
use crate::editor::syntax as syntax_cache;

#[cfg(feature = "syntax-rust")]
#[test]
fn rust_syntax_spans_are_materialized_for_rows() {
    let handle = CodeEditorHandle::new("fn main() {\n    let x = 1;\n}\n");
    handle.set_language(Some(Arc::<str>::from("rust")));

    let mut st = handle.state.borrow_mut();
    let line_count = st.buffer.line_count();
    assert!(line_count > 0);

    let mut any_highlight = false;
    for row in 0..line_count {
        let spans = syntax_cache::cached_row_syntax_spans(&mut st, row, 256);
        if !spans.is_empty() {
            any_highlight = true;
            break;
        }
    }
    assert!(
        any_highlight,
        "expected at least one highlighted span for rust"
    );
}

#[cfg(feature = "syntax-rust")]
#[test]
fn set_language_is_idempotent_for_same_value() {
    let handle = CodeEditorHandle::new("fn main() {\n    let x = 1;\n}\n");

    assert_eq!(
        handle.cache_stats().syntax_resets,
        0,
        "new handles should not reset syntax caches"
    );

    handle.set_language(Some(Arc::<str>::from("rust")));

    {
        let mut st = handle.state.borrow_mut();
        let _ = syntax_cache::cached_row_syntax_spans(&mut st, 0, 256);
        let _ = syntax_cache::cached_row_syntax_spans(&mut st, 1, 256);
        assert!(
            st.syntax_row_cache.contains_key(&0),
            "expected syntax cache entry for row 0"
        );
        assert!(
            st.syntax_row_cache.contains_key(&1),
            "expected syntax cache entry for row 1"
        );
    }

    let resets_before = handle.cache_stats().syntax_resets;

    // The UI layer may call set_language during render; that must be a no-op when the language is
    // unchanged to avoid per-frame cache resets and re-highlighting work.
    handle.set_language(Some(Arc::<str>::from("rust")));
    assert_eq!(
        handle.cache_stats().syntax_resets,
        resets_before,
        "idempotent set_language must not reset syntax caches"
    );

    {
        let st = handle.state.borrow();
        assert!(
            st.syntax_row_cache.contains_key(&0),
            "expected syntax cache entry for row 0 to survive idempotent set_language"
        );
        assert!(
            st.syntax_row_cache.contains_key(&1),
            "expected syntax cache entry for row 1 to survive idempotent set_language"
        );
    }
}

#[test]
fn set_line_folds_is_idempotent_for_same_value() {
    let handle = CodeEditorHandle::new("abcdef\n");

    let placeholder = Arc::<str>::from("…");
    let spans = vec![FoldSpan {
        range: 1..3,
        placeholder,
    }];

    handle.set_line_folds(0, spans.clone());

    let (folds_epoch_before, row_text_resets_before) = {
        let st = handle.state.borrow();
        (st.folds_epoch, st.cache_stats.row_text_resets)
    };

    handle.set_line_folds(0, spans);

    let st = handle.state.borrow();
    assert_eq!(
        st.folds_epoch, folds_epoch_before,
        "idempotent set_line_folds must not bump folds_epoch"
    );
    assert_eq!(
        st.cache_stats.row_text_resets, row_text_resets_before,
        "idempotent set_line_folds must not reset row text caches"
    );
}

#[test]
fn set_line_inlays_is_idempotent_for_same_value() {
    let handle = CodeEditorHandle::new("abcdef\n");

    let spans = vec![InlaySpan {
        byte: 2,
        text: Arc::<str>::from("<inlay>"),
    }];

    handle.set_line_inlays(0, spans.clone());

    let (inlays_epoch_before, row_text_resets_before) = {
        let st = handle.state.borrow();
        (st.inlays_epoch, st.cache_stats.row_text_resets)
    };

    handle.set_line_inlays(0, spans);

    let st = handle.state.borrow();
    assert_eq!(
        st.inlays_epoch, inlays_epoch_before,
        "idempotent set_line_inlays must not bump inlays_epoch"
    );
    assert_eq!(
        st.cache_stats.row_text_resets, row_text_resets_before,
        "idempotent set_line_inlays must not reset row text caches"
    );
}

#[cfg(feature = "syntax-rust")]
#[test]
fn syntax_cache_invalidation_preserves_far_rows_on_inline_edit() {
    let mut text = String::new();
    for _ in 0..200 {
        text.push_str("fn main() {}\n");
    }

    let handle = CodeEditorHandle::new(text.as_str());
    handle.set_language(Some(Arc::<str>::from("rust")));

    let mut st = handle.state.borrow_mut();
    let max_entries = 4096;
    let _ = syntax_cache::cached_row_syntax_spans(&mut st, 0, max_entries);
    let _ = syntax_cache::cached_row_syntax_spans(&mut st, 150, max_entries);
    assert!(
        st.syntax_row_cache.contains_key(&150),
        "expected far-row cache entries to be populated"
    );

    input::apply_and_record_edit(
        &mut st,
        UndoGroupKind::Typing,
        Edit::Insert {
            at: 0,
            text: "x".to_string(),
        },
        Selection {
            anchor: 1,
            focus: 1,
        },
    )
    .expect("apply edit");

    assert!(
        st.syntax_row_cache.contains_key(&150),
        "expected far-row cache entries to survive inline edit invalidation"
    );
    assert!(
        !st.syntax_row_cache.contains_key(&0),
        "expected near-row cache entries to be invalidated"
    );
}

#[cfg(feature = "syntax-rust")]
#[test]
fn syntax_cache_invalidation_shifts_far_rows_on_newline_insertion() {
    let mut text = String::new();
    for _ in 0..200 {
        text.push_str("fn main() {}\n");
    }

    let handle = CodeEditorHandle::new(text.as_str());
    handle.set_language(Some(Arc::<str>::from("rust")));

    let mut st = handle.state.borrow_mut();
    let max_entries = 4096;

    let _ = syntax_cache::cached_row_syntax_spans(&mut st, 0, max_entries);
    let spans_150 = syntax_cache::cached_row_syntax_spans(&mut st, 150, max_entries);
    assert!(
        st.syntax_row_cache.contains_key(&150),
        "expected far-row cache entries to be populated"
    );

    input::apply_and_record_edit(
        &mut st,
        UndoGroupKind::Typing,
        Edit::Insert {
            at: 0,
            text: "\n".to_string(),
        },
        Selection {
            anchor: 1,
            focus: 1,
        },
    )
    .expect("apply edit");

    let shifted_row = 151;
    let (shifted_entry, _) = st
        .syntax_row_cache
        .get(&shifted_row)
        .expect("expected shifted far-row cache entry");
    assert!(
        Arc::ptr_eq(shifted_entry, &spans_150),
        "expected the old far-row cache entry to move to the shifted row key"
    );
    assert!(
        !st.syntax_row_cache.contains_key(&0),
        "expected near-row cache entries to be invalidated"
    );

    let hits_before = st.cache_stats.syntax_hits;
    let spans_after = syntax_cache::cached_row_syntax_spans(&mut st, shifted_row, max_entries);
    assert!(
        st.cache_stats.syntax_hits > hits_before,
        "expected shifted far-row cache to hit"
    );
    assert!(Arc::ptr_eq(&spans_after, &spans_150));
}

#[cfg(feature = "syntax-rust")]
#[test]
fn syntax_cache_invalidation_shifts_far_rows_on_newline_deletion() {
    let mut text = String::new();
    for _ in 0..200 {
        text.push_str("fn main() {}\n");
    }

    let newline = text.find('\n').expect("expected a newline");

    let handle = CodeEditorHandle::new(text.as_str());
    handle.set_language(Some(Arc::<str>::from("rust")));

    let mut st = handle.state.borrow_mut();
    let max_entries = 4096;

    let _ = syntax_cache::cached_row_syntax_spans(&mut st, 0, max_entries);
    let spans_150 = syntax_cache::cached_row_syntax_spans(&mut st, 150, max_entries);
    assert!(
        st.syntax_row_cache.contains_key(&150),
        "expected far-row cache entries to be populated"
    );

    input::apply_and_record_edit(
        &mut st,
        UndoGroupKind::Typing,
        Edit::Delete {
            range: newline..newline + 1,
        },
        Selection {
            anchor: newline,
            focus: newline,
        },
    )
    .expect("apply edit");

    let shifted_row = 149;
    let (shifted_entry, _) = st
        .syntax_row_cache
        .get(&shifted_row)
        .expect("expected shifted far-row cache entry");
    assert!(
        Arc::ptr_eq(shifted_entry, &spans_150),
        "expected the old far-row cache entry to move to the shifted row key"
    );
    assert!(
        !st.syntax_row_cache.contains_key(&0),
        "expected near-row cache entries to be invalidated"
    );

    let hits_before = st.cache_stats.syntax_hits;
    let spans_after = syntax_cache::cached_row_syntax_spans(&mut st, shifted_row, max_entries);
    assert!(
        st.cache_stats.syntax_hits > hits_before,
        "expected shifted far-row cache to hit"
    );
    assert!(Arc::ptr_eq(&spans_after, &spans_150));
}

#[cfg(feature = "syntax-rust")]
#[test]
fn syntax_cache_invalidation_invalidates_bounded_window_around_edit() {
    let mut text = String::new();
    for _ in 0..300 {
        text.push_str("fn main() {}\n");
    }

    let handle = CodeEditorHandle::new(text.as_str());
    handle.set_language(Some(Arc::<str>::from("rust")));

    let mut st = handle.state.borrow_mut();
    let max_entries = 4096;

    let line_10 = syntax_cache::cached_row_syntax_spans(&mut st, 10, max_entries);
    let line_50 = syntax_cache::cached_row_syntax_spans(&mut st, 50, max_entries);
    let line_150 = syntax_cache::cached_row_syntax_spans(&mut st, 150, max_entries);
    let line_200 = syntax_cache::cached_row_syntax_spans(&mut st, 200, max_entries);

    assert!(st.syntax_row_cache.contains_key(&10));
    assert!(st.syntax_row_cache.contains_key(&50));
    assert!(st.syntax_row_cache.contains_key(&150));
    assert!(st.syntax_row_cache.contains_key(&200));

    let edit_line = 100usize;
    let at = st
        .buffer
        .line_start(edit_line)
        .expect("expected line start");

    input::apply_and_record_edit(
        &mut st,
        UndoGroupKind::Typing,
        Edit::Insert {
            at,
            text: "x".to_string(),
        },
        Selection {
            anchor: at + 1,
            focus: at + 1,
        },
    )
    .expect("apply edit");

    assert!(
        st.syntax_row_cache.contains_key(&10),
        "expected far-row entry outside the invalidation window to survive"
    );
    assert!(
        st.syntax_row_cache.contains_key(&200),
        "expected far-row entry outside the invalidation window to survive"
    );
    assert!(
        !st.syntax_row_cache.contains_key(&50),
        "expected entry inside the lookback/lookahead invalidation window to be evicted"
    );
    assert!(
        !st.syntax_row_cache.contains_key(&150),
        "expected entry inside the lookback/lookahead invalidation window to be evicted"
    );

    let hits_before = st.cache_stats.syntax_hits;
    let line_10_after = syntax_cache::cached_row_syntax_spans(&mut st, 10, max_entries);
    let line_200_after = syntax_cache::cached_row_syntax_spans(&mut st, 200, max_entries);
    assert!(
        st.cache_stats.syntax_hits >= hits_before + 2,
        "expected preserved far-row cache to hit"
    );
    assert!(Arc::ptr_eq(&line_10_after, &line_10));
    assert!(Arc::ptr_eq(&line_200_after, &line_200));

    let _ = line_50;
    let _ = line_150;
}

#[cfg(feature = "syntax-rust")]
#[test]
fn syntax_cache_invalidation_shifts_far_rows_on_multiple_newline_insertion() {
    let mut text = String::new();
    for _ in 0..200 {
        text.push_str("fn main() {}\n");
    }

    let handle = CodeEditorHandle::new(text.as_str());
    handle.set_language(Some(Arc::<str>::from("rust")));

    let mut st = handle.state.borrow_mut();
    let max_entries = 4096;
    let spans_150 = syntax_cache::cached_row_syntax_spans(&mut st, 150, max_entries);

    input::apply_and_record_edit(
        &mut st,
        UndoGroupKind::Typing,
        Edit::Insert {
            at: 0,
            text: "\n\n\n".to_string(),
        },
        Selection {
            anchor: 3,
            focus: 3,
        },
    )
    .expect("apply edit");

    let shifted_row = 153;
    let (shifted_entry, _) = st
        .syntax_row_cache
        .get(&shifted_row)
        .expect("expected shifted far-row cache entry");
    assert!(
        Arc::ptr_eq(shifted_entry, &spans_150),
        "expected the old far-row cache entry to move to the shifted row key"
    );
}

#[cfg(feature = "syntax-rust")]
#[test]
fn syntax_prefetch_key_distinguishes_documents_with_same_revision() {
    let language: Arc<str> = Arc::<str>::from("rust");
    let key_a = crate::editor::syntax::SyntaxPrefetchKey {
        doc: DocId::new(),
        rev: fret_code_editor_buffer::Revision(0),
        language: Arc::clone(&language),
        chunk_start: 0,
        chunk_end: 127,
    };
    let key_b = crate::editor::syntax::SyntaxPrefetchKey {
        doc: DocId::new(),
        rev: fret_code_editor_buffer::Revision(0),
        language,
        chunk_start: 0,
        chunk_end: 127,
    };

    assert_ne!(key_a, key_b);
}

#[cfg(feature = "syntax-rust")]
#[test]
fn syntax_rows_from_highlight_spans_maps_across_rows() {
    let row_ranges = vec![0..4, 4..8, 8..12];
    let rows = crate::editor::syntax::syntax_rows_from_highlight_spans(
        0,
        10,
        &row_ranges,
        vec![fret_syntax::HighlightSpan {
            range: 1..10,
            highlight: Some("keyword"),
        }],
    );

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].0, 10);
    assert_eq!(rows[1].0, 11);
    assert_eq!(rows[2].0, 12);
    assert_eq!(
        rows[0].1.as_ref(),
        &[SyntaxSpan {
            range: 1..4,
            highlight: "keyword",
        }]
    );
    assert_eq!(
        rows[1].1.as_ref(),
        &[SyntaxSpan {
            range: 0..4,
            highlight: "keyword",
        }]
    );
    assert_eq!(
        rows[2].1.as_ref(),
        &[SyntaxSpan {
            range: 0..2,
            highlight: "keyword",
        }]
    );
}

#[cfg(feature = "syntax-rust")]
#[test]
fn syntax_cache_invalidation_shifts_far_rows_on_multiple_line_deletion() {
    let mut text = String::new();
    for _ in 0..200 {
        text.push_str("fn main() {}\n");
    }

    let handle = CodeEditorHandle::new(text.as_str());
    handle.set_language(Some(Arc::<str>::from("rust")));

    let mut st = handle.state.borrow_mut();
    let max_entries = 4096;

    let spans_150 = syntax_cache::cached_row_syntax_spans(&mut st, 150, max_entries);
    assert!(
        st.syntax_row_cache.contains_key(&150),
        "expected far-row cache entries to be populated"
    );

    let end = st.buffer.line_start(3).expect("expected a line start");
    input::apply_and_record_edit(
        &mut st,
        UndoGroupKind::Typing,
        Edit::Delete { range: 0..end },
        Selection {
            anchor: 0,
            focus: 0,
        },
    )
    .expect("apply edit");

    let shifted_row = 147;
    let (shifted_entry, _) = st
        .syntax_row_cache
        .get(&shifted_row)
        .expect("expected shifted far-row cache entry");
    assert!(
        Arc::ptr_eq(shifted_entry, &spans_150),
        "expected the old far-row cache entry to move to the shifted row key"
    );
}
