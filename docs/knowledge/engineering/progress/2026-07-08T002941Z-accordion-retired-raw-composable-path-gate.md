---
type: "Work Progress"
title: "Accordion retired raw composable path gate"
description: "Work Progress for Accordion retired raw composable path gate."
timestamp: 2026-07-08T00:29:41Z
tags: ["ui-gallery", "shadcn", "raw-surface", "accordion"]
status: "done"
verified_by: "cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_import_policies -E 'test(raw_shadcn_escape_hatch_gate_is_symbol_level_not_module_level) or test(gallery_source_tree_limits_raw_shadcn_escape_hatches)'"
---

# Summary

The UI Gallery Accordion docs no longer spell the retired `shadcn::raw::accordion::composable`
path in user-visible page copy. The global Gallery raw escape-hatch classifier now treats that path
as forbidden instead of classifying it as a documented raw seam.

# Details

- Updated the Accordion API-reference note to describe the old path as the "legacy raw composable
  escape hatch" without teaching the concrete `shadcn::raw::*` path.
- Removed `shadcn::raw::accordion::composable` from the allowed raw escape-hatch classifier in
  `ui_authoring_surface_import_policies.rs`.
- Added it to the classifier's forbidden examples so future Gallery snippets/pages cannot pass the
  raw gate by relying on retired Accordion raw vocabulary.
- Renamed the default-app authoring-surface test from the old "advanced seam" language to the
  current curated typed-children surface.

# Next Action

Continue scanning user-facing Gallery pages and snippet gates for raw/advanced terminology that is
only historical and should not be taught on the current default lane.

# Citations

- `apps/fret-ui-gallery/src/ui/pages/accordion.rs`
- `apps/fret-ui-gallery/tests/accordion_docs_surface.rs`
- `apps/fret-ui-gallery/tests/ui_authoring_surface_import_policies.rs`
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`
- `cargo nextest run -p fret-ui-gallery --test accordion_docs_surface accordion_page_documents_docs_path_and_children_api_decision`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_import_policies -E 'test(raw_shadcn_escape_hatch_gate_is_symbol_level_not_module_level) or test(gallery_source_tree_limits_raw_shadcn_escape_hatches)'`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app accordion_usage_snippet_keeps_curated_typed_children_seam`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `cargo fmt --all --check`
