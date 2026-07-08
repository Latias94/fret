---
type: "Work Progress"
title: "Collapsible raw wildcard path gate"
description: "Work Progress for Collapsible raw wildcard path gate."
timestamp: 2026-07-08T00:38:04Z
tags: ["ui-gallery", "shadcn", "raw-surface", "collapsible"]
status: "done"
verified_by: "cargo nextest run -p fret-ui-gallery --test collapsible_docs_surface collapsible_page_documents_source_axes_and_children_api_decision"
---

# Summary

The UI Gallery Collapsible docs no longer teach the module-wide
`shadcn::raw::collapsible::primitives::*` wildcard path. The remaining raw Collapsible escape hatch
is the explicit `use shadcn::raw::collapsible::primitives as shadcn_col;` alias used by the
source-alignment settings-panel snippet.

# Details

- Reworded Collapsible page copy to describe the raw primitive alias without spelling a wildcard
  `shadcn::raw::*` import path.
- Added a docs-surface assertion that the Collapsible page does not teach
  `shadcn::raw::collapsible::primitives::*`.
- Removed the wildcard path from the Gallery raw escape-hatch allowlist and added it to the
  forbidden examples in `ui_authoring_surface_import_policies.rs`.

# Next Action

Continue scanning user-facing Gallery pages for raw module-wide imports or advanced terms that
should be replaced by curated facade wording or explicit narrow aliases.

# Citations

- `apps/fret-ui-gallery/src/ui/pages/collapsible.rs`
- `apps/fret-ui-gallery/tests/collapsible_docs_surface.rs`
- `apps/fret-ui-gallery/tests/ui_authoring_surface_import_policies.rs`
- `apps/fret-ui-gallery/src/ui/snippets/collapsible/settings_panel.rs`
- `cargo nextest run -p fret-ui-gallery --test collapsible_docs_surface collapsible_page_documents_source_axes_and_children_api_decision`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_import_policies -E 'test(raw_shadcn_escape_hatch_gate_is_symbol_level_not_module_level) or test(gallery_source_tree_limits_raw_shadcn_escape_hatches)'`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `cargo fmt --all --check`
