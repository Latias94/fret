---
type: "Work Progress"
title: "Custom effect v2 web demos use app text facade"
description: "Work Progress for Custom effect v2 web demos use app text facade."
timestamp: 2026-07-07T10:19:37Z
tags: ["ui-surface", "examples", "custom-effects", "web", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/custom-effect-v2-web-text-facade"
---

# Summary

Moved the CustomEffectV2 Web demo overlay label/readout helpers off raw
`fret_ui_kit::declarative::text` imports and onto the app-facing `fret::app::text` facade.

# Details

Changed files:

- `apps/fret-examples/src/custom_effect_v2_web_demo.rs`
- `apps/fret-examples/src/custom_effect_v2_identity_web_demo.rs`
- `apps/fret-examples/src/custom_effect_v2_lut_web_demo.rs`
- `apps/fret-examples/tests/custom_effect_overlay_text_surface.rs`

Decision:

- Keep owner, renderer, diagnostics, effect hook, and WebGPU adapter logic unchanged.
- Preserve the shared overlay helper shape, but make both helper variants generic over
  `AppRenderContext<'a>` instead of `ElementContext<'_, H>`/`UiHost`.
- Keep explicit foreground inheritance after the app text facade call so the overlay visual contract
  does not move into the facade.
- Extend the source surface test to require `fret::app::{AppRenderContext, text}` and forbid the old
  raw `decl_text`/bare text constructor path in the three Web CustomEffectV2 demos.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `git diff --check`

Note: the Rust test build still emits the pre-existing `fret-chart::visual_map_track_at` dead code
warning.

# Next Action

Merge this slice back to `main` and push remote `main`, then continue with remaining default app text
facade seams such as docking/editor examples where the current app facade already has matching roles.

# Citations

- `apps/fret-examples/src/custom_effect_v2_web_demo.rs`
- `apps/fret-examples/src/custom_effect_v2_identity_web_demo.rs`
- `apps/fret-examples/src/custom_effect_v2_lut_web_demo.rs`
- `apps/fret-examples/tests/custom_effect_overlay_text_surface.rs`
