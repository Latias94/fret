# Public Surface Classification - 2026-05-12

Status: Initial M1 classification

This note classifies the current editor public surface into stable, experimental, and
internal-by-accident buckets. It is intentionally conservative: "stable" means the item already
matches the ADR 0185 buffer/view/surface split and has enough evidence to remain part of the public
contract.

## Immediate Finding

The first M1 code fix is a root re-export cleanup in `fret-code-editor`:

- `CodeFontFeaturePolicy`
- `CodeFontFeaturePreset`
- `CodeEditorCacheSizeSnapshotV1`
- `CodeEditorMemorySnapshotV1`

These types were already part of public method signatures on `CodeEditor` / `CodeEditorHandle`, but
were not re-exported from `ecosystem/fret-code-editor/src/lib.rs`. The lane treats that as a public
surface correctness issue, not a new feature.

Regression anchor:

- `ecosystem/fret-code-editor/tests/public_surface.rs`

## `fret-code-editor-buffer`

Stable:

- `DocId`
- `DocUri`
- `Revision`
- `Selection`
- `Edit`
- `AppliedEdit`
- `TextBufferTx`
- `TextBufferTransaction`
- `LineDelta`
- `BufferDelta`
- `EditError`
- `TextBuffer`

Rationale: these are the document identity, byte-indexed selection, edit, revision, and transaction
contracts that ADR 0185 places in the buffer layer. They are also free of UI/runtime dependencies.

Experimental:

- `TextBuffer::text_string`
- `TextBuffer::line_text`
- `TextBuffer::line_char_count`

Rationale: useful for tests/debugging and small surfaces, but editor-grade call sites should avoid
whole-line or whole-buffer materialization on hot paths unless the call is explicitly diagnostic.

Internal-by-accident candidates:

- none identified in the root buffer model yet.

## `fret-code-editor-view`

Stable or near-stable:

- `DisplayPoint`
- `DisplayMap`
- `MaterializedDisplayRow`
- `DisplayRowSpan`
- `DisplayRowFragment`
- `DisplayRowFragmentsError`
- `FoldSpan`
- `FoldSpanError`
- `InlaySpan`
- `InlaySpanError`
- `InlinePreedit`

Experimental extension contracts:

- `DiagnosticSeverity`
- `DiagnosticSourceKind`
- `DiagnosticSpan`
- `DiagnosticSpanError`
- `DiagnosticLineSummary`
- `validate_diagnostic_spans`
- `normalized_diagnostic_spans`
- `diagnostic_line_summaries`
- `GutterMarkerAnchor`
- `GutterMarkerKind`
- `GutterMarkerVisual`
- `GutterMarkerHitTarget`
- `GutterMarker`
- `GutterMarkerError`
- `validate_gutter_markers`
- `normalized_gutter_markers`
- `RangeDecorationLayer`
- `RangeDecorationHitTest`
- `RangeDecoration`
- `RangeDecorationError`
- `validate_range_decorations`
- `normalized_range_decorations`
- `SemanticToken`
- `SemanticTokenError`
- `validate_semantic_tokens`
- `normalized_semantic_tokens`

Rationale: diagnostics are the first M2 extension model slice. They use the correct view-layer
coordinate ownership, and gutter markers/range decorations now have explicit payload contracts.
Semantic tokens now have a paint-color-free input contract. These remain experimental until a UI
proof is wired on top of the data contract.

Experimental:

- `code_wrap_policy::CodeWrapPreset`
- `code_wrap_policy::CodeWrapKnobs`
- `code_wrap_policy::CodeWrapPolicy`
- `code_wrap_policy::CodeWrapRowStart`
- `row_starts_for_code_wrap`
- `row_spans::buffer_local_to_display_local`
- `row_spans::map_buffer_range_to_display_ranges`

Rationale: code wrap and row-span mapping are the right layer, but the exact API shape may change
when diagnostics/decorations/gutter payloads are added.

Internal-by-accident candidates:

- Free helper functions that duplicate `DisplayMap` methods should be reviewed before they become
  a long-term public teaching surface:
  - `byte_to_display_point`
  - `display_point_to_byte`
  - `clamp_to_char_boundary`
  - `clamp_to_grapheme_boundary_down`
  - `clamp_to_grapheme_boundary_up`
  - `prev_char_boundary`
  - `next_char_boundary`
  - `select_word_range`
  - `select_line_range`
  - `move_word_left`
  - `move_word_right`
  - `select_word_range_in_buffer`
  - `move_word_left_in_buffer`
  - `move_word_right_in_buffer`

These may remain public, but the docs should decide whether app authors should learn the free
helpers or the `DisplayMap`/buffer-owned APIs first.

## `fret-code-editor`

Stable:

- `CodeEditor`
- `CodeEditorHandle`
- `CodeEditorInteractionOptions`

Near-stable diagnostics/perf:

- `CodeEditorCacheStats`
- `CodeEditorCacheSizeSnapshotV1`
- `CodeEditorMemorySnapshotV1`
- `CodeEditorPaintPerfFrame`

Rationale: these are already consumed by diagnostics and perf workstreams. They are versioned or
diagnostics-oriented, but still need root export and compatibility discipline because they are
returned by public methods.

Compatibility re-export:

- `Selection` remains available from `fret-code-editor` root, but its canonical owner is now
  `fret-code-editor-buffer` because it is a byte-indexed buffer model type.

Experimental:

- `PreeditState`
- `CodeFontFeaturePreset`
- `CodeFontFeaturePolicy`
- `CodeEditorTorture`

Rationale: preedit and font-feature policy are real editor features, but their long-term shape is
still tied to ADR 0188 and text shaping work. `CodeEditorTorture` is a harness feature, not an app
model.

Internal-by-accident candidates:

- debug and diagnostics helpers on `CodeEditorHandle`:
  - `set_preedit_debug`
  - `debug_platform_set_marked_text_for_selection`
  - `debug_platform_cancel_marked_text`
  - `debug_decorated_line_text`
  - `diag_buffer_contains_str_cached`

These should either stay explicitly named as debug/diag APIs or move behind a diagnostics/test
adapter before a stable release.

## `CodeEditorHandle` Method Groups

Model mutation:

- `replace_buffer`
- `set_text`
- `set_selection`
- `set_caret`

View configuration:

- `set_soft_wrap_cols`
- `set_code_wrap_policy`
- `set_line_folds`
- `clear_all_folds`
- `set_line_inlays`
- `clear_all_inlays`
- `set_language`
- `set_code_font_feature_policy`

Interaction policy:

- `interaction`
- `set_interaction`
- `set_text_boundary_mode`
- `set_text_boundary_mode_override`

State readout:

- `buffer_revision`
- `selection`
- `preedit_active`
- `allow_decorations_under_inline_preedit`
- `compose_inline_preedit`
- `region_id`
- `text_boundary_mode`
- `text_boundary_mode_override`
- `can_undo`
- `can_redo`
- `with_buffer`

Diagnostics/perf:

- `cache_stats`
- `cache_size_snapshot`
- `memory_snapshot`
- `paint_perf_frame`
- `reset_cache_stats`
- `diag_buffer_contains_str_cached`

Debug/staging:

- `set_preedit_debug`
- `debug_platform_set_marked_text_for_selection`
- `debug_platform_cancel_marked_text`
- `debug_decorated_line_text`
- `set_allow_decorations_under_inline_preedit`
- `set_compose_inline_preedit`

## Next Cleanup Candidates

1. Decide whether `CodeEditorHandle` should expose grouped option/update structs for view
   decorations instead of one setter per feature family.
2. Decide whether fold/inlay APIs should move to a general `EditorDecorationSet` or remain direct
   v1 methods until diagnostics/gutter are implemented.
3. Decide whether debug/diag helpers stay on `CodeEditorHandle` or move to a diagnostics adapter.
4. Decide whether `fret-code-editor` should re-export common buffer/view input types such as
   `TextBuffer`, `FoldSpan`, `InlaySpan`, and `CodeWrapPolicy`, or require app authors to depend on
   the lower-layer crates explicitly.
