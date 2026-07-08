---
type: Work Progress
title: Menu item variants curated facade gate
timestamp: 2026-07-08T02:29:24Z
tags:
  - shadcn
  - public-surface
  - ui-gallery
  - verification
status: verified
---

# Summary

Promoted the app-facing menu item variant enums through the curated `fret-ui-shadcn` facade:
`DropdownMenuItemVariant`, `ContextMenuItemVariant`, and `MenubarItemVariant`.

# Changed Files

- `ecosystem/fret-ui-shadcn/src/lib.rs`: re-exported the three menu item variant enums from the
  facade next to their menu item builders.
- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs`: added facade-only export coverage for the
  menu variant enums.
- `apps/fret-ui-gallery/src/ui/snippets/**`: moved context menu, dropdown menu, menubar, avatar,
  button group, and table action examples from `shadcn::raw::*ItemVariant` to `shadcn::*ItemVariant`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_import_policies.rs`: removed the menu variant
  raw allowlist seams and added forbidden fixtures for raw variant usage.

# Verification

- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_import_policies`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app context_menu`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app dropdown`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app menubar`
- `cargo nextest run -p fret-ui-shadcn --lib surface_policy_tests`
- `cargo fmt --all`
- `cargo fmt --all --check`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `git diff --check`

# Next Candidates

Remaining gallery raw escape hatches after this slice are narrower: explicit breadcrumb and
collapsible primitive aliases, experimental data grid models, and broad typography/icon/extras raw
families. The primitive aliases still need source-alignment review before promotion because they
represent composition lanes, not just simple public builder parameter enums.
