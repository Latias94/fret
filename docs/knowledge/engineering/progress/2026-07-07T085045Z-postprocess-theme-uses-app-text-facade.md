---
type: "Work Progress"
title: "Postprocess theme demo uses app text facade"
description: "Work Progress for Postprocess theme demo uses app text facade."
timestamp: 2026-07-07T08:50:45Z
tags: ["ui-surface", "examples", "postprocess", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/markdown-text-facade"
---

# Summary

Moved `postprocess_theme_demo.rs` fixed overlay title/readout helpers off `decl_text` and raw
`AnyElement` signatures onto the app-facing `fret::app::text` and `AppElement` surface.

# Details

Changed files:

- `apps/fret-examples/src/postprocess_theme_demo.rs`
- `apps/fret-examples/tests/custom_effect_overlay_text_surface.rs`
- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Decision:

- Reuse existing `fret::app::text::section_chrome_label` and `control_readout` helpers, then keep
  the demo-specific white overlay foreground as local inherited styling.
- Do not add a new public foreground helper for this slice; the existing app text facade was enough.
- Remove `AnyElement` from the postprocess theme advanced/manual allowed raw seam list.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- raw text seam scan for `postprocess_theme_demo.rs` found no `AnyElement`, `decl_text`, or
  `fret_ui_kit::declarative::text` hits.

Note: the Rust test build still emits the pre-existing `fret-chart::visual_map_track_at` dead code
warning.

# Next Action

Continue with another classified advanced/manual surface only when the remaining seam is facade
shrinkage, not deliberate renderer, runner, or retained-tree ownership.

# Citations

- `apps/fret-examples/src/postprocess_theme_demo.rs`
- `apps/fret-examples/tests/custom_effect_overlay_text_surface.rs`
- `tools/check_surface_policy.py`
