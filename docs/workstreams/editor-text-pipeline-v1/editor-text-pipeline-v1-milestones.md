# Editor Text Pipeline v1 — Milestones

This is a milestone checklist for:

- `docs/workstreams/editor-text-pipeline-v1/editor-text-pipeline-v1.md`

## M0 — Boundary doc + evidence anchors

Exit criteria:

- The editor pipeline boundary is documented:
  - what the renderer owns vs what the editor owns.
- Evidence anchors are listed for the current implementation.
- A minimal “golden” set of invariants is listed (mapping + cache stability).

## M1 — Row text cache (allocation control)

Exit criteria:

- Visible display rows are materialized as `Arc<str>` only.
- Row cache keyed by:
  - buffer revision,
  - display row index,
  - wrap cols / width bucket (best-effort),
  - fold/inlay epochs (decorations participate in display rows).
- Large documents do not require `to_string()` of the whole buffer per frame.

Status (code + initial regression gates exist):

- Row text cache + LRU exists in the editor paint path:
  - `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (`cached_row_text_with_range`)
- Regression gate exists to prevent whole-buffer string materialization in paint:
  - `ecosystem/fret-code-editor/src/editor/tests/mod.rs` (`paint_source_does_not_materialize_whole_buffer_string`)

Evidence checklist:

- Add (optional) a unit test or micro benchmark-like test that:
  - edits a large buffer,
  - repaints a small viewport,
  - and asserts bounded allocations / bounded row rebuild.

## M2 — Per-row attributed spans for syntax highlighting

Exit criteria:

- Syntax highlighting spans are generated per visible row and passed as `AttributedText`.
- Paint-only changes do not trigger reshaping/wrapping.
- A regression test exists for “theme color change” not affecting shaping keys.

Implementation notes (expected):

- Spans are expressed in row-local UTF-8 byte ranges:
  - ranges are char-boundary aligned,
  - spans do not cross row boundaries.
- Syntax highlighting is paint-only:
  - `TextPaintStyle` is set,
  - `TextShapingStyle` remains unchanged for syntax (no accidental reshaping).

Evidence checklist:

- `cargo nextest run -p fret-code-editor-view -p fret-code-editor`
- `cargo nextest run -p fret-render-wgpu`

Evidence anchors (expected):

- Row-local span construction:
  - `ecosystem/fret-code-editor-view/src/lib.rs`
  - `ecosystem/fret-code-editor-view/src/row_spans.rs`
- Renderer integration for rich text rows:
  - `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (`rich_text_with_blob`)
- Defensive span normalization (stale/out-of-date ranges):
  - `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (`normalize_syntax_spans_for_text`)
- Paint-only shaping_eq gate (editor-level):
  - `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (`paint_only_syntax_color_changes_do_not_affect_rich_text_shaping_eq`)
- Grapheme-safe span gate (ZWJ/VS16):
  - `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (`normalize_syntax_spans_does_not_split_zwj_or_vs16_graphemes`)
- Platform UTF-16 determinism gate (mixed scripts + surrogates):
  - `crates/fret-ui/src/declarative/tests/semantics.rs`
    (`declarative_text_input_region_utf16_queries_are_deterministic_for_mixed_scripts_and_surrogates`)
- Shaping-key stability gate:
  - `crates/fret-render-wgpu/src/text/mod.rs` (`multispan_paint_changes_do_not_affect_shape_key`)

## M3 — Code wrap policy separation

Exit criteria:

- Editor view model drives wrap segmentation for code.
- Renderer wrapper is not relied on for editor row segmentation.
- Cursor movement / selection semantics match the display-row segmentation (no drift).
- A stable, auditable ecosystem policy surface exists:
  - presets exist (at least `Conservative` / `Balanced` / `Aggressive`),
  - a small set of common knobs is supported for app-specific tuning (paths/URLs, identifiers),
  - behavior is deterministic and covered by fixture-driven conformance tests.

Status (baseline + no-drift gates exist):

- Display-row segmentation uses the ecosystem policy:
  - `ecosystem/fret-code-editor-view/src/lib.rs` (`compute_wrapped_row_start_cols`)
- Policy conformance fixtures (JSON-driven):
  - `ecosystem/fret-code-editor-view/tests/code_wrap_policy_fixtures.rs`
  - `ecosystem/fret-code-editor-view/tests/fixtures/code_wrap_policy_v1.json`
- Policy-aware byte ↔ display mapping gate:
  - `ecosystem/fret-code-editor-view/src/lib.rs` (`byte_to_display_point_respects_code_wrap_policy_rows`)
- No-drift navigation gates (selection + vertical movement):
  - `ecosystem/fret-code-editor/src/editor/tests/mod.rs` (`move_caret_vertical_steps_through_code_wrap_policy_rows`)
  - `ecosystem/fret-code-editor/src/editor/tests/mod.rs` (`shift_vertical_extends_selection_in_display_row_space_when_wrapped`)
- No-drift pointer selection gates (inlays + preedit replacement):
  - `ecosystem/fret-code-editor/src/editor/tests/mod.rs` (`pointer_down_double_click_selects_word_on_inlay_only_row_under_soft_wrap`)
  - `ecosystem/fret-code-editor/src/editor/tests/mod.rs` (`pointer_down_double_click_cancels_preedit_replacement_and_selects_word`)
  - `ecosystem/fret-code-editor/src/editor/tests/mod.rs` (`triple_click_selects_logical_line_on_inlay_only_row_under_soft_wrap`)
  - `ecosystem/fret-code-editor/src/editor/tests/mod.rs` (`triple_click_cancels_preedit_replacement_and_selects_logical_line`)

## M4 — Platform text input interop (TextInputRegion UTF-16)

Exit criteria:

- `TextInputRegion` provides a stable platform-facing view for editor-grade text input by answering:
  - `PlatformTextInputQuery::{SelectedTextRange, MarkedTextRange, TextForRange}`
  - in UTF-16 code units over `TextInputRegionProps.a11y_value`.
- `WindowTextInputSnapshot` is published for focused `TextInputRegion` with:
  - `text_len_utf16`, `selection_utf16`, `marked_utf16` derived from the composed view.
  - `ime_cursor_area` forwarded from data-only props when provided (editor-owned geometry).
- Non-goals for the mechanism layer (stage later):
  - `BoundsForRange`, `CharacterIndexForPoint`, and `replace_*` are not implemented by default in
    `fret-ui`.
  - Ecosystem/editor surfaces may answer `BoundsForRange` / `CharacterIndexForPoint` via
    `TextInputRegionActionHooks.on_platform_text_input_query` while keeping `fret-ui` as a
    routing-only mechanism.
 - Ecosystem/editor surfaces may implement platform replace-by-range (`replace_*`) via
   `TextInputRegionActionHooks` replace handlers while keeping `fret-ui` as a routing-only
   mechanism.
  - Font changes invalidate editor geometry:
    - editor surfaces observe `TextFontStackKey` and clear any cached row geometry used to answer
      platform geometry queries (bounds/hit-test), preventing stale caret/selection rectangles.
  - Staging: selection-replacing preedit is represented in the platform-facing composed view and
    in the display-row text composition so shaping/paint do not drift during IME composition.
    - Evidence:
      - `ecosystem/fret-code-editor-view/src/lib.rs`
      - `ecosystem/fret-code-editor/src/editor/tests/mod.rs`

Future work (deferred):

- Multi-line selection replacement composition (cross-newline ranges) is not implemented yet.
  - Current v1 staging clamps cross-newline ranges to the anchor logical line for determinism.
  - Track implementation in `docs/workstreams/editor-text-pipeline-v1/editor-text-pipeline-v1-todo.md` (M4).

Evidence anchors:

- Implementation:
  - `crates/fret-ui/src/tree/ui_tree_text_input.rs` (`text_input_region_platform_text_input_query`)
  - `crates/fret-ui/src/tree/paint.rs` (snapshot publishing)
- Tests:
  - `crates/fret-ui/src/declarative/tests/semantics.rs` (`declarative_text_input_region_answers_platform_text_input_queries_in_utf16`)
  - `crates/fret-ui/src/declarative/tests/semantics.rs`
    (`declarative_text_input_region_utf16_queries_are_deterministic_for_mixed_scripts_and_surrogates`)
  - `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
    (`a11y_source_does_not_materialize_whole_buffer_string`)
  - `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
    (`a11y_composed_window_is_bounded_for_large_documents`)
  - `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
    (`platform_replace_and_mark_empty_text_cancels_and_restores_selection`)
  - `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
    (`platform_replace_and_mark_range_spanning_newline_is_clamped_to_anchor_line`)
  - `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
    (`platform_text_input_bounds_and_index_roundtrip_under_preedit_replacement_and_wrap`)
  - `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
    (`platform_text_input_bounds_and_index_roundtrip_under_inline_preedit_composed_window_and_wrap`)
  - `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
    (`platform_text_input_bounds_and_index_roundtrip_under_inline_preedit_composed_window_with_decorations_and_wrap`)
- Diag gates:
  - `tools/diag-scripts/ui-gallery-code-editor-a11y-composition-baseline.json`
  - `crates/fret-diag/src/stats.rs` (`check_bundle_for_ui_gallery_code_editor_a11y_composition_json`)

## M5 — Row geometry cache boundary (future fearless refactor)

Exit criteria:

- Editor geometry caches have an explicit, stable key that includes:
  - shaping-relevant style,
  - wrap/constraints buckets,
  - scale factor,
  - `TextFontStackKey` revision.
- Paint-only (theme) changes do not invalidate row geometry caches.
- Regression gates exist for:
  - resize jitter stability,
  - font stack invalidation (no stale geometry answers).

Evidence anchors (initial):

- Geometry cache key:
  - `ecosystem/fret-code-editor/src/editor/geom/mod.rs` (`RowGeomKey`)
  - `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (geometry cache hit uses `RowGeomKey`)
- Display-map epoch gate:
  - `ecosystem/fret-code-editor/src/editor/tests/mod.rs` (`code_wrap_policy_change_invalidates_row_text_cache`)
- Paint-only stability gate:
  - `ecosystem/fret-code-editor/src/editor/tests/mod.rs` (`row_geom_key_ignores_paint_only_changes`)

Resize jitter catastrophic guard (code editor):

- Script:
  - `tools/diag-scripts/ui-gallery-code-editor-window-resize-drag-jitter-steady.json`
- Smoke gate:
  - `tools/perf/diag_code_editor_resize_jitter_smoke_gate.py`
