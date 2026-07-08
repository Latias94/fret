---
type: "Work Progress"
title: "Breadcrumb docs path curated facade gate"
description: "Work Progress for Breadcrumb docs path curated facade gate."
timestamp: 2026-07-08T01:10:41Z
tags: ["ui-gallery", "shadcn", "raw-surface", "breadcrumb"]
status: "done"
verified_by: "cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_import_policies"
---

# Summary

The UI Gallery Breadcrumb docs-path examples now keep `Dropdown`, `Link Component`, and `RTL` on
the curated `shadcn::Breadcrumb*` facade aliases instead of reopening
`shadcn::raw::breadcrumb::primitives`.

# Details

- Migrated `dropdown.rs`, `link_component.rs`, and `rtl.rs` from the raw breadcrumb primitive alias
  to `BreadcrumbRoot`, `BreadcrumbList`, `BreadcrumbItemPart`, `BreadcrumbSeparatorPart`,
  `BreadcrumbLink`, and `BreadcrumbPage` on the curated facade.
- Narrowed the raw breadcrumb primitive source-policy batch to the Fret-specific `Responsive`
  follow-up.
- Expanded the docs-path curated-parts gate so `Demo`, `Usage`, `Basic`, `Custom Separator`,
  `Dropdown`, `Collapsed`, `Link Component`, and `RTL` all reject the raw breadcrumb primitive
  escape hatch.
- Updated the Breadcrumb page note so it describes raw breadcrumb primitives as only the responsive
  drawer handoff seam.

# Next Action

Continue scanning UI Gallery shadcn pages for default docs-path examples that still expose raw
primitive aliases or advanced wording when a curated facade alias already exists.

# Citations

- `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/dropdown.rs`
- `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/link_component.rs`
- `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/rtl.rs`
- `apps/fret-ui-gallery/src/ui/pages/breadcrumb.rs`
- `apps/fret-ui-gallery/tests/ui_authoring_surface_import_policies.rs`
- `apps/fret-ui-gallery/tests/breadcrumb_docs_surface.rs`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_import_policies`
- `cargo nextest run -p fret-ui-gallery --test breadcrumb_docs_surface`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `cargo fmt --all --check`
- `git diff --check`
