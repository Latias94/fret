---
type: "Work Progress"
title: "Custom effect labels use app text facade"
description: "Work Progress for Custom effect labels use app text facade."
timestamp: 2026-07-07T10:02:42Z
tags: ["ui-surface", "examples", "custom-effects", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/custom-effect-label-text-facade"
---

# Summary

Moved the fixed overlay label helpers in `custom_effect_v1_demo.rs` and `custom_effect_v2_demo.rs`
off `fret_ui_kit::declarative::text` and onto the app-facing `fret::app::text` facade.

# Details

Changed files:

- `apps/fret-examples/src/custom_effect_v1_demo.rs`
- `apps/fret-examples/src/custom_effect_v2_demo.rs`
- `apps/fret-examples/tests/custom_effect_overlay_text_surface.rs`

Decision:

- Keep explicit custom-effect setup, shader/effect ownership, and advanced driver hooks unchanged.
- Convert the shared label helper shape from raw `ElementContext<'_, H>`/`UiHost` to the default app
  `AppRenderContext<'a>` lane.
- Keep the inherited white foreground styling at the call boundary after delegating to
  `fret::app::text::section_chrome_label`.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- direct text helper scan for the v1/v2 source files found no `decl_text`,
  `fret_ui_kit::declarative::text`, `text_section_chrome_label`, or `UiHost` hits.

Note: the Rust test build still emits the pre-existing `fret-chart::visual_map_track_at` dead code
warning.

# Next Action

Merge this slice back to `main` and push remote `main`, then continue with remaining App/AppUi
text seams where the existing app facade already covers the role.

# Citations

- `apps/fret-examples/src/custom_effect_v1_demo.rs`
- `apps/fret-examples/src/custom_effect_v2_demo.rs`
- `apps/fret-examples/tests/custom_effect_overlay_text_surface.rs`
