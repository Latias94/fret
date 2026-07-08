---
type: Work Progress
title: ButtonStyle raw gallery references migrated to the curated shadcn facade
timestamp: 2026-07-08T01:49:01Z
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

The next public-surface sweep found `shadcn::raw::button::ButtonStyle::default()` in DatePicker and
Collapsible gallery snippets. `ButtonStyle` was already exported through the curated
`fret_ui_shadcn::facade`, so the raw allowlist entry was historical drift rather than a real escape
hatch.

This slice migrated the gallery snippets to `shadcn::ButtonStyle` and moved raw ButtonStyle into the
negative fixtures for `ui_authoring_surface_import_policies`.

# Changed Files

- `apps/fret-ui-gallery/src/ui/snippets/date_picker/basic.rs`
- `apps/fret-ui-gallery/src/ui/snippets/date_picker/demo.rs`
- `apps/fret-ui-gallery/src/ui/snippets/date_picker/usage.rs`
- `apps/fret-ui-gallery/src/ui/snippets/collapsible/file_tree.rs`
- `apps/fret-ui-gallery/tests/ui_authoring_surface_import_policies.rs`

# Verification

- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_import_policies`
- `cargo nextest run -p fret-ui-gallery --test collapsible_docs_surface`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app date_picker`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `cargo fmt --all --check`
- `git diff --check`

# Notes

An attempted `cargo nextest run -p fret-ui-gallery --test date_picker_docs_surface` failed because
that test target does not exist. The DatePicker coverage was instead provided by the existing
`ui_authoring_surface_default_app` `date_picker` filter set.
