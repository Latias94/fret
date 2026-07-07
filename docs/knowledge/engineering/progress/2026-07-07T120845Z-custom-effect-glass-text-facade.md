---
type: "Work Progress"
title: "Custom effect glass chrome web uses app text facade"
description: "Work Progress for Custom effect glass chrome web uses app text facade."
timestamp: 2026-07-07T12:08:45Z
tags: ["ui-surface", "examples", "text", "facade", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/custom-effect-glass-text-facade"
---

# Summary

Moved the custom effect v2 glass chrome web demo's foreground-aware control label/readout helpers
off raw `fret_ui_kit::declarative::text` and onto the app-facing `fret::app::text` facade.

# Details

Changed files:

- `apps/fret-examples/src/custom_effect_v2_glass_chrome_web_demo.rs`
- `apps/fret-examples/tests/custom_effect_overlay_text_surface.rs`

Decision:

- Match the existing custom-effect v2 web pattern: helpers are generic over `AppRenderContext<'a>`
  and apply inherited foreground after constructing app text roles.
- Keep the file's other raw effect/rendering seams unchanged.
- Update the text-role source test to forbid `decl_text` on this app-lane demo.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples custom_effect_v2_web_templates_use_shared_text_roles --no-fail-fast`
- `cargo nextest run -p fret-examples custom_effect_v2_glass_chrome_web_common_controls_use_binding --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `git diff --check`

# Next Action

Merge this slice back to `main` and push remote `main`, then continue separating remaining
advanced/kernel text seams from default app-lane text facade candidates.

# Citations

- `apps/fret-examples/src/custom_effect_v2_glass_chrome_web_demo.rs`
- `apps/fret-examples/tests/custom_effect_overlay_text_surface.rs`
