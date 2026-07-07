---
type: "Work Progress"
title: "Image-heavy memory demo uses app text facade"
description: "Work Progress for Image-heavy memory demo uses app text facade."
timestamp: 2026-07-07T09:43:37Z
tags: ["ui-surface", "examples", "memory", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/image-heavy-memory-text-facade"
---

# Summary

Moved `image_heavy_memory_demo.rs` status readout text off `fret_ui_kit::declarative::text` and
onto the app-facing `fret::app::text` facade.

# Details

Changed files:

- `apps/fret-examples/src/image_heavy_memory_demo.rs`
- `apps/fret-examples/tests/image_heavy_memory_demo_surface.rs`

Decision:

- Keep explicit GPU hooks and renderer ownership unchanged; this slice only cleans the ordinary app
  text readout surface.
- Tighten the extracted `render_view` helper from raw `ElementContextAccess<'a, App>` to
  `AppRenderContext<'a>`, then use `cx.app_mut()` for global state access before rendering the app
  facade text.
- Keep low-level `cx.elements()` only for the image grid and scroll primitives that still need raw
  element construction.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test image_heavy_memory_demo_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- direct text helper scan for `image_heavy_memory_demo.rs` found no `decl_text`,
  `fret_ui_kit::declarative::text`, or `text_control_readout` hits.

Note: the Rust test build still emits the pre-existing `fret-chart::visual_map_track_at` dead code
warning.

# Next Action

Merge this slice back to `main` and push remote `main`, then continue prioritizing default App/AppUi
surfaces where existing app facades cover the text role.

# Citations

- `apps/fret-examples/src/image_heavy_memory_demo.rs`
- `apps/fret-examples/tests/image_heavy_memory_demo_surface.rs`
