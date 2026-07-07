---
type: "Work Progress"
title: "Liquid glass uses app text facade"
description: "Work Progress for Liquid glass uses app text facade."
timestamp: 2026-07-07T12:22:21Z
tags: ["ui-surface", "examples", "text", "facade", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/liquid-glass-text-facade"
---

# Summary

Moved `liquid_glass_demo.rs` overlay/card chrome text helpers from raw
`fret_ui_kit::declarative::text` to the app-facing `fret::app::text` facade.

# Details

Changed files:

- `apps/fret-examples/src/liquid_glass_demo.rs`
- `apps/fret-examples/tests/custom_effect_overlay_text_surface.rs`

Decision:

- Keep the demo classified as an advanced/reference surface for renderer capabilities, backdrop
  warp, and custom-effect graph validation.
- Narrow `lens_panel` from the unused generic `UiHost` shape to the concrete `App` view lane so the
  overlay title can use `AppRenderContext`.
- Preserve the Rust 2024 precise-capture marker on `lens_panel` with `+ use<>` to avoid holding the
  mutable render context borrow across `into_element`.
- Update the source-surface tests to require app text roles and forbid reintroducing `decl_text` for
  this file.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples liquid_glass_demo_uses_app_view_imports_with_explicit_effect_hooks --no-fail-fast`
- `cargo nextest run -p fret-examples custom_effect_v3_and_effect_reference_chrome_use_shared_roles --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `git diff --check`

# Next Action

Merge this slice back to `main` and push remote `main`, then continue separating remaining advanced
kernel text seams from default app-lane facade candidates.

# Citations

- `apps/fret-examples/src/liquid_glass_demo.rs`
- `apps/fret-examples/tests/custom_effect_overlay_text_surface.rs`
