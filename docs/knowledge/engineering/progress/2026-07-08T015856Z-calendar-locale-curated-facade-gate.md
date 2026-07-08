---
type: Work Progress
title: CalendarLocale promoted to the curated shadcn facade
timestamp: 2026-07-08T01:58:56Z
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

The fourth public-surface cleanup slice found `shadcn::raw::calendar::CalendarLocale::Es` in the
Calendar locale gallery snippet. `CalendarLocale` is the typed parameter for
`Calendar::locale(...)`, so it belongs on the curated facade rather than the raw calendar module.

This slice re-exported `CalendarLocale` through `fret_ui_shadcn::facade`, migrated the gallery
locale snippet to `shadcn::CalendarLocale::Es`, and moved raw CalendarLocale into the negative
fixtures for `ui_authoring_surface_import_policies`.

# Changed Files

- `ecosystem/fret-ui-shadcn/src/lib.rs`
- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs`
- `apps/fret-ui-gallery/src/ui/snippets/calendar/locale.rs`
- `apps/fret-ui-gallery/tests/ui_authoring_surface_import_policies.rs`

# Verification

- `cargo nextest run -p fret-ui-shadcn --lib surface_policy_tests`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_import_policies`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app calendar`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `cargo fmt --all --check`
- `git diff --check`

# Notes

The raw `CalendarLocale` path is now only present as a negative fixture in the gallery import-policy
test. This follows the same pattern as the earlier `SelectPosition` and `TabsOrientation` facade
promotions.
