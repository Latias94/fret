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

Status (code exists; tests pending):

- Row text cache + LRU exists in the editor paint path:
  - `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (`cached_row_text_with_range`)
- Remaining: add a regression gate that fails if paint materializes the whole buffer per frame.

Evidence checklist:

- Add a unit test or micro benchmark-like test that:
  - edits a large buffer,
  - repaints a small viewport,
  - and asserts bounded allocations / bounded row rebuild.

## M2 — Per-row attributed spans for syntax highlighting

Exit criteria:

- Syntax highlighting spans are generated per visible row and passed as `AttributedText`.
- Paint-only changes do not trigger reshaping/wrapping.
- A regression test exists for “theme color change” not affecting shaping keys.

## M3 — Code wrap policy separation

Exit criteria:

- Editor view model drives wrap segmentation for code.
- Renderer wrapper is not relied on for editor row segmentation.
- Cursor movement / selection semantics match the display-row segmentation (no drift).
