---
type: "Work Progress"
title: "Components gallery uses app text facade"
description: "Work Progress for Components gallery uses app text facade."
timestamp: 2026-07-07T12:58:23Z
tags: ["ui-surface", "examples", "text", "facade", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/components-gallery-text-facade"
---

# Summary

Moved the components gallery fixed table/chrome/control/overlay text roles from raw
`fret_ui_kit::declarative::text` helpers to the app-facing `fret::app::text` facade.

# Details

Changed files:

- `apps/fret-examples/src/components_gallery.rs`
- `apps/fret-examples/tests/components_gallery_surface.rs`

Decision:

- Keep `components_gallery.rs` classified as an advanced surface because it still owns the manual
  window lifecycle, component-state matrix, file-dialog hooks, diagnostics integration, and model
  owner helpers.
- Move only fixed text roles onto `fret::app::text`, including retained table cell renderers,
  gallery chrome/readouts, and overlay prose.
- Preserve the table callback shape expected by `table_virtualized_retained_v0`; the callback still
  receives `dyn ElementContextAccess` and uses its concrete `ElementContext<'_, App>` for app text
  roles.
- Update the component gallery source tests to require app text roles and reject `decl_text`.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples components_gallery --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `git diff --check`
- `rg -n "use fret_ui_kit::declarative::text as decl_text|decl_text::" apps/fret-examples/src/components_gallery.rs apps/fret-examples/tests/components_gallery_surface.rs`
  only found the negative test assertion.

# Next Action

Merge this slice back to `main` and push remote `main`, then continue classifying the remaining
KernelApp/manual text seams.

# Citations

- `apps/fret-examples/src/components_gallery.rs`
- `apps/fret-examples/tests/components_gallery_surface.rs`
