---
type: "Work Progress"
title: "Gallery raw shadcn escape hatch gate tightening"
description: "Work Progress for Gallery raw shadcn escape hatch gate tightening."
timestamp: 2026-07-07T23:26:18Z
tags: ["fret", "shadcn", "ui-gallery", "raw-surface", "public-surface", "source-policy"]
---

# Summary

Tightened the UI Gallery raw shadcn escape-hatch source policy from a broad module-name allowlist
to symbol-level classified seams. This keeps intentional raw lanes visible while preventing new
unreviewed `shadcn::raw::*` modules from passing by module name alone.

# Details

- Updated `apps/fret-ui-gallery/tests/ui_authoring_surface_import_policies.rs` so
  `gallery_source_tree_limits_raw_shadcn_escape_hatches` now checks each raw line against
  classified symbols/families.
- Added `raw_shadcn_escape_hatch_gate_is_symbol_level_not_module_level` with positive examples for
  current classified seams and negative examples for unclassified `kbd`, `toggle_group`,
  `experimental`, `button`, and breadcrumb-private raw paths.
- Updated `docs/shadcn-declarative-progress.md` to describe the raw budget as symbol-level rather
  than module-level.
- Verification passed:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_import_policies
  raw_shadcn_escape_hatch_gate_is_symbol_level_not_module_level`;
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_import_policies`;
  `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`;
  `python3 tools/check_layering.py`;
  `python3 tools/report_largest_files.py --top 30 --min-lines 800`;
  `cargo fmt --all --check`;
  `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`;
  `git diff --check`.
- The wiki memory validation stayed structurally OK and reported only pre-existing rollup/history
  warnings.

# Next Action

Commit and push `main`.

# Citations

- `apps/fret-ui-gallery/tests/ui_authoring_surface_import_policies.rs`
- `docs/shadcn-declarative-progress.md`
