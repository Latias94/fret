# Editor Text Pipeline v1 — Milestones

This is a milestone checklist for:

- `docs/workstreams/editor-text-pipeline-v1.md`

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

Evidence anchors:

- Implementation:
  - `crates/fret-ui/src/tree/mod.rs` (`text_input_region_platform_text_input_query`)
  - `crates/fret-ui/src/tree/paint.rs` (snapshot publishing)
- Tests:
  - `crates/fret-ui/src/declarative/tests/semantics.rs` (`declarative_text_input_region_answers_platform_text_input_queries_in_utf16`)

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
- Paint-only stability gate:
  - `ecosystem/fret-code-editor/src/editor/tests/mod.rs` (`row_geom_key_ignores_paint_only_changes`)
