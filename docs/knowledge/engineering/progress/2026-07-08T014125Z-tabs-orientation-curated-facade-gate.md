---
type: Work Progress
title: TabsOrientation promoted to the curated shadcn facade
timestamp: 2026-07-08T01:41:25Z
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

After the `SelectPosition` facade slice, the next real raw public-surface gap was
`shadcn::raw::tabs::TabsOrientation::Vertical` in the Tabs gallery examples. `TabsOrientation` is the
typed parameter for `Tabs::orientation(...)`, so it belongs on the curated `shadcn` facade instead
of forcing authors into the raw tabs module.

This slice promoted `TabsOrientation` through `fret_ui_shadcn::facade`, migrated the Tabs vertical
and vertical-line snippets to `shadcn::TabsOrientation`, and removed the raw tabs orientation
allowlist entry from the gallery raw escape-hatch gate.

# Changed Files

- `ecosystem/fret-ui-shadcn/src/lib.rs`
- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs`
- `apps/fret-ui-gallery/src/ui/snippets/tabs/vertical.rs`
- `apps/fret-ui-gallery/src/ui/snippets/tabs/vertical_line.rs`
- `apps/fret-ui-gallery/tests/tabs_docs_surface.rs`
- `apps/fret-ui-gallery/tests/ui_authoring_surface_import_policies.rs`

# Verification

- `cargo nextest run -p fret-ui-shadcn --lib surface_policy_tests`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_import_policies`
- `cargo nextest run -p fret-ui-gallery --test tabs_docs_surface`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `cargo fmt --all --check`
- `git diff --check`

# Notes

The raw `TabsOrientation` path is now only present as a negative fixture in
`apps/fret-ui-gallery/tests/ui_authoring_surface_import_policies.rs`.

Remaining broad raw families in the gallery are still dominated by the documented typography,
icon, extras, variant-enum, and explicit primitive follow-up seams.
