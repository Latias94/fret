---
type: "Work Progress"
title: "Custom effect v3 uses app text facade"
description: "Work Progress for Custom effect v3 uses app text facade."
timestamp: 2026-07-07T10:08:22Z
tags: ["ui-surface", "examples", "custom-effects", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/custom-effect-v3-text-facade"
---

# Summary

Moved `custom_effect_v3_demo.rs` overlay label helper off
`fret_ui_kit::declarative::text` and onto the app-facing `fret::app::text` facade.

# Details

Changed files:

- `apps/fret-examples/src/custom_effect_v3_demo.rs`
- `apps/fret-examples/tests/custom_effect_overlay_text_surface.rs`

Decision:

- Keep explicit CustomV3 setup, diagnostics probe programs, renderer hooks, and app view structure
  unchanged.
- Preserve the existing `AppRenderContext<'a>` helper shape, but remove the raw
  `cx.elements()`/kit text detour inside the helper.
- Keep inherited white foreground styling after the app text facade call.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- direct text helper scan for `custom_effect_v3_demo.rs` found no `decl_text`,
  `fret_ui_kit::declarative::text`, or `text_section_chrome_label` hits.

Note: the Rust test build still emits the pre-existing `fret-chart::visual_map_track_at` dead code
warning.

# Next Action

Merge this slice back to `main` and push remote `main`, then continue with remaining default app
text seams while leaving KernelApp/raw diagnostics surfaces for a separate facade design.

# Citations

- `apps/fret-examples/src/custom_effect_v3_demo.rs`
- `apps/fret-examples/tests/custom_effect_overlay_text_surface.rs`
