---
type: "Work Progress"
title: "Breadcrumb demo curated facade gate"
description: "Work Progress for Breadcrumb demo curated facade gate."
timestamp: 2026-07-08T00:59:32Z
tags: ["ui-gallery", "shadcn", "raw-surface", "breadcrumb"]
status: "done"
verified_by: "cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_import_policies"
---

# Summary

The UI Gallery Breadcrumb `Demo` snippet now stays on the curated `shadcn::Breadcrumb*` facade
aliases instead of reopening `use shadcn::raw::breadcrumb::primitives as bc;` on the first
docs-path example.

# Details

- Replaced the raw breadcrumb primitive alias in `src/ui/snippets/breadcrumb/demo.rs` with
  `shadcn::BreadcrumbRoot`, `BreadcrumbList`, `BreadcrumbItemPart`, `BreadcrumbSeparatorPart`,
  `BreadcrumbEllipsis`, `BreadcrumbLink`, and `BreadcrumbPage`.
- Moved the demo snippet from the raw primitive batch gate to the docs-path curated-parts gate in
  `ui_authoring_surface_import_policies.rs`.
- Updated the Breadcrumb page notes so the documented curated docs-path set includes `Demo`.

# Next Action

Continue scanning UI Gallery shadcn pages for default docs-path examples that still reopen raw
breadcrumb/collapsible/accordion-style primitive lanes when a curated facade alias already exists.

# Citations

- `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/demo.rs`
- `apps/fret-ui-gallery/src/ui/pages/breadcrumb.rs`
- `apps/fret-ui-gallery/tests/ui_authoring_surface_import_policies.rs`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_import_policies`
- `cargo nextest run -p fret-ui-gallery --test breadcrumb_docs_surface`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `cargo fmt --all --check`
- `git diff --check`
