---
type: "Work Progress"
title: "Embedded viewport demo uses app text facade"
description: "Work Progress for Embedded viewport demo uses app text facade."
timestamp: 2026-07-07T10:28:33Z
tags: ["ui-surface", "examples", "embedded-viewport", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/embedded-viewport-text-facade"
---

# Summary

Moved `embedded_viewport_demo.rs` fixed toggle labels and readouts off raw
`fret_ui_kit::declarative::text` imports and onto the app-facing `fret::app::text` facade.

# Details

Changed files:

- `apps/fret-examples/src/embedded_viewport_demo.rs`
- `apps/fret-examples/tests/embedded_viewport_demo_surface.rs`
- `tools/check_surface_policy.py`

Decision:

- Keep embedded viewport interop hooks, forwarded input handling, render target setup, diagnostics,
  and model owner logic unchanged.
- Convert the local label/readout helpers from `ElementContext<'_, H>`/`UiHost` to
  `AppRenderContext<'a>`.
- Pass `cx` directly for toggle label text instead of dropping to `cx.elements()` only to call the
  raw kit text constructors.
- Shrink the `embedded_viewport_demo.rs` surface-policy allowlist by removing the stale
  `ElementContext` raw seam after the helper conversion.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test embedded_viewport_demo_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `git diff --check`

Note: the Rust test build still emits the pre-existing `fret-chart::visual_map_track_at` dead code
warning.

# Next Action

Merge this slice back to `main` and push remote `main`, then continue with remaining default app text
facade seams such as docking and virtual-list examples where the current app facade already has
matching roles.

# Citations

- `apps/fret-examples/src/embedded_viewport_demo.rs`
- `apps/fret-examples/tests/embedded_viewport_demo_surface.rs`
- `tools/check_surface_policy.py`
