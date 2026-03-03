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

