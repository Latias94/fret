---
type: Work Progress
title: SwitchStyle curated facade gate
timestamp: 2026-07-08T02:17:16Z
tags:
  - shadcn
  - public-surface
  - ui-gallery
  - verification
status: verified
---

# Summary

Promoted `SwitchStyle` through the curated `fret-ui-shadcn` facade so gallery authoring examples
can refine switch styling without reaching into `shadcn::raw::switch`.

# Changed Files

- `ecosystem/fret-ui-shadcn/src/lib.rs`: re-exported `SwitchStyle` from the curated facade.
- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs`: added `SwitchStyle` to facade-only
  export expectations and refreshed the state-query feature-gate marker.
- `apps/fret-ui-gallery/src/ui/snippets/switch/bluetooth.rs`: moved the style example to
  `shadcn::SwitchStyle`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_import_policies.rs`: removed the raw
  `SwitchStyle` allowlist seam and added a forbidden fixture for raw usage.

# Verification

- `cargo nextest run -p fret-ui-gallery --test switch_docs_surface`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_import_policies`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app switch`
- `cargo nextest run -p fret-ui-shadcn --lib surface_policy_tests`
- `cargo fmt --all --check`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `git diff --check`

The first `surface_policy_tests` run in this slice caught a stale feature-gate marker that still
encoded the older switch facade export list. The marker was updated, and the rerun passed.

# Next Candidates

Remaining gallery raw escape hatches still include menu item variant types, explicit breadcrumb and
collapsible primitive lanes, experimental data grid models, typography/icon/extras families, and
other intentionally broad advanced seams. Menu item variant types look like the nearest facade
promotion candidates because they are app-facing builder parameters.
