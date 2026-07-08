---
type: Work Progress
title: Data grid explicit experimental facade
timestamp: 2026-07-08T03:09:06Z
tags:
  - shadcn
  - public-surface
  - ui-gallery
  - verification
status: verified
---

# Summary

Added an explicit `shadcn::experimental` facade submodule for unstable element-grid prototypes and
moved the gallery data grid preview from `shadcn::raw::experimental::*` to
`shadcn::experimental::*`.

The data grid element prototype remains experimental; this slice only removes the unnecessary raw
module reopen from gallery authoring code.

# Changed Files

- `ecosystem/fret-ui-shadcn/src/lib.rs`: added `facade::experimental` re-exporting
  `DataGridElement` and `DataGridRowState`.
- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs`: added a policy test that keeps
  element-grid prototypes under `facade::experimental` and prevents promotion to stable top-level
  facade names.
- `apps/fret-ui-gallery/src/ui/previews/gallery/data/data_grid.rs`: migrated the preview to
  `shadcn::experimental::*`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_import_policies.rs`: removed
  `raw::experimental::DataGrid*` from the gallery raw allowlist, added forbidden fixtures, and
  allowed only the explicit `shadcn::experimental` module path.

# Verification

- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_import_policies`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews data_grid`
- `cargo nextest run -p fret-ui-shadcn --lib data_grid`
- `cargo fmt --all`
- `cargo fmt --all --check`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `git diff --check`

# Next Candidates

The gallery raw allowlist now covers only broad raw typography, icon, and extras helpers. Those
families are intentionally large helper namespaces rather than single missing facade symbols, so the
next useful slice should start with a smaller inventory before promoting any names.
