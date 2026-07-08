---
type: Work Progress
title: SelectPosition promoted to the curated shadcn facade
timestamp: 2026-07-08T01:30:21Z
status: ready-to-commit
tags:
  - fret
  - fret-ui-shadcn
  - ui-gallery
  - public-surface
  - shadcn-parity
source_workspace: /Users/frankorz/Documents/projects/rust/fret
git_branch: main
---

# Summary

The public-surface follow-up lane found that `SelectContent::position(...)` was forcing first-party
gallery snippets through `shadcn::raw::select::SelectPosition`. This was a real default-authoring
surface gap because `SelectPosition` is the typed enum accepted by the curated `SelectContent`
builder.

This slice promoted `SelectPosition` through `fret_ui_shadcn::facade` and `prelude`, migrated the
Select, Button Group, and Resizable gallery snippets to `shadcn::SelectPosition`, and removed the
raw select-position allowlist entry from the gallery raw escape-hatch gate.

# Changed Files

- `ecosystem/fret-ui-shadcn/src/lib.rs`
- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs`
- `apps/fret-ui-gallery/src/ui/snippets/select/align_item_with_trigger.rs`
- `apps/fret-ui-gallery/src/ui/snippets/select/rich_items.rs`
- `apps/fret-ui-gallery/src/ui/snippets/button_group/button_group_select.rs`
- `apps/fret-ui-gallery/src/ui/snippets/resizable/multi_viewport_select.rs`
- `apps/fret-ui-gallery/tests/select_docs_surface.rs`
- `apps/fret-ui-gallery/tests/ui_authoring_surface_import_policies.rs`

# Verification

- `cargo nextest run -p fret-ui-shadcn --lib surface_policy_tests`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_import_policies`
- `cargo nextest run -p fret-ui-gallery --test select_docs_surface`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `cargo fmt --all --check`
- `git diff --check`

# Notes

The first `fret-ui-shadcn` surface run exposed a stale marker for `DataTableUiBuilderExt`; the
actual `DataTable::into_element` seam already requires `H: UiHost + 'static`, so the marker was
updated to continue gating the explicit `into_element(...) -> AnyElement` landing seam.

Remaining module-level raw shadcn aliases in the gallery are still the documented Fret-specific
follow-ups:

- `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/responsive.rs`
- `apps/fret-ui-gallery/src/ui/snippets/collapsible/settings_panel.rs`
