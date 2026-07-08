---
type: Work Progress
title: Primitive aliases use curated facade
timestamp: 2026-07-08T02:43:01Z
tags:
  - shadcn
  - public-surface
  - ui-gallery
  - verification
status: verified
---

# Summary

Removed the last gallery raw breadcrumb/collapsible primitive alias reopen points that already had
curated facade aliases. `breadcrumb/responsive.rs` now uses `BreadcrumbRoot`, `BreadcrumbList`,
`BreadcrumbItemPart`, `BreadcrumbSeparatorPart`, `BreadcrumbEllipsis`, `BreadcrumbLink`, and
`BreadcrumbPage` through `shadcn::`. `collapsible/settings_panel.rs` now uses
`CollapsibleRoot`, `CollapsibleTriggerPart`, and `CollapsibleContentPart` through `shadcn::`.

# Changed Files

- `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/responsive.rs`: removed
  `use shadcn::raw::breadcrumb::primitives as bc;` and migrated all primitive calls to curated
  facade aliases.
- `apps/fret-ui-gallery/src/ui/snippets/collapsible/settings_panel.rs`: removed
  `use shadcn::raw::collapsible::primitives as shadcn_col;` and migrated to curated collapsible
  primitive aliases.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_import_policies.rs`: removed breadcrumb and
  collapsible primitive aliases from the raw allowlist and added/updated facade alias assertions.
- `apps/fret-ui-gallery/tests/collapsible_docs_surface.rs`: updated the settings-panel expectations
  to require curated collapsible aliases.

# Verification

- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_import_policies`
- `cargo nextest run -p fret-ui-gallery --test collapsible_docs_surface`
- `cargo nextest run -p fret-ui-gallery --test breadcrumb_docs_surface`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app breadcrumb`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app collapsible`
- `cargo fmt --all`
- `cargo fmt --all --check`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `git diff --check`

# Next Candidates

After this slice, the gallery raw allowlist is down to broad raw typography/icon/extras helpers plus
the `raw::experimental` data grid preview. Data grid should be handled separately because the current
policy intentionally marks it experimental, not just a missing facade alias.
