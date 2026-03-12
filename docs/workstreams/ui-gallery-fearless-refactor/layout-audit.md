# UI Gallery Doc Layout Audit

This document tracks **layout drift risks** in UI Gallery pages (primarily `DocSection` scaffolding).

Scope:

- Scan source files under `apps/fret-ui-gallery/src/ui/pages/**` for `.max_w(Px(..))` usage.
- This is a **coarse text scan** (not AST-aware) intended to highlight hotspots for follow-up
  normalization work.

## How to regenerate

Run:

```bash
python3 tools/ui_gallery_layout_audit.py > docs/workstreams/ui-gallery-fearless-refactor/layout-audit.generated.md
```

Then review the diff and decide which overrides should be removed or standardized.

## Notes

- With the new page-level centering (`apps/fret-ui-gallery/src/ui/doc_layout.rs`), max-width
  differences should no longer create inconsistent **left gutters**, but they can still create
  inconsistent **right margins** and perceived “density” across pages.
- The audit defaults to `--scope doc_sections`, which reports `.max_w(Px(..))` usage only in
  `DocSection::new(...)` builder chains (the doc scaffold surface). Use `--scope all` if you want a
  full text scan of every `.max_w(Px(..))` in the file (including layout refinements inside the
  page's preview composition).

## Reviewed allowlist (2026-03-11)

The remaining `DocSection`-level width overrides were reviewed and are **intentionally retained**.
No further doc-scaffold width normalization is currently required for this workstream.

Retained buckets:

- `1000.0` for canvas / torture / workflow surfaces that need broad interaction area:
  - `apps/fret-ui-gallery/src/ui/pages/ai_canvas_world_layer_spike.rs`
  - `apps/fret-ui-gallery/src/ui/pages/ai_transcript_torture.rs`
  - `apps/fret-ui-gallery/src/ui/pages/ai_workflow_canvas_demo.rs`
  - `apps/fret-ui-gallery/src/ui/pages/ai_workflow_node_graph_demo.rs`
- `1100.0` for comparison / gallery-heavy compositions:
  - `apps/fret-ui-gallery/src/ui/pages/chart.rs`
  - `apps/fret-ui-gallery/src/ui/pages/item.rs`
- `980.0` for wide-but-structured recipe pages where narrower defaults materially reduce readability:
  - `apps/fret-ui-gallery/src/ui/pages/calendar.rs`
  - `apps/fret-ui-gallery/src/ui/pages/card.rs`
  - `apps/fret-ui-gallery/src/ui/pages/data_table.rs`
  - `apps/fret-ui-gallery/src/ui/pages/date_picker.rs`
  - `apps/fret-ui-gallery/src/ui/pages/icons.rs`
  - `apps/fret-ui-gallery/src/ui/pages/image_object_fit.rs`
  - `apps/fret-ui-gallery/src/ui/pages/sheet.rs`
  - `apps/fret-ui-gallery/src/ui/pages/sidebar.rs`

Follow-up rule:

- New `DocSection::max_w(Px(..))` overrides should either use the default width or add a short
  rationale to this audit if they intentionally join the retained wide-page allowlist.
