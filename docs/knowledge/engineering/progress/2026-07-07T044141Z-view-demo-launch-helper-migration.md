---
type: "Work Progress"
title: "View demo launch helper migration"
description: "Work Progress for View demo launch helper migration."
timestamp: 2026-07-07T04:41:41Z
tags: ["ui-surface", "examples", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/public-example-facade-migration"
---

# Summary

Moved repeated default `View` demo launch wiring out of `plot_image_demo.rs` and `tags_demo.rs`
into crate-local helpers in `apps/fret-examples/src/lib.rs`.

# Details

Changed files:

- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/src/plot_image_demo.rs`
- `apps/fret-examples/src/tags_demo.rs`
- `apps/fret-examples/tests/basic_plot_demos_surface.rs`
- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Decision:

- Keep this as an `apps/fret-examples` helper first, not a stable `fret` public API. The helper is
  demo-shell specific because `apps/fret-demo-web` still calls `build_app`, `build_runner_config`,
  and `build_fn_driver` per demo.
- Centralize the raw `fret::advanced::view` and `fret_runtime::PlatformCapabilities` seams in the
  examples crate root internal harness instead of letting default-looking plot demos import them.
- Tighten the policy records for `plot_image_demo.rs` and `tags_demo.rs` so they only allow the
  remaining `fret_launch` signature seam.

Verification passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret-examples --test basic_plot_demos_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo check -p fret-examples --target wasm32-unknown-unknown`
- `cargo check -p fret-demo-web --target wasm32-unknown-unknown`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_layering.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_consumption_profiles.py`
- engineering wiki memory validation for `docs/knowledge/engineering`
- `git diff --check`

# Next Action

Continue the same migration pattern across other simple default-view demos that still hand-roll
`build_app`, `build_runner_config`, and `build_fn_driver`; once enough examples share the shape,
decide whether this belongs as a stable `fret::app`/`FretApp` web launch parts facade.

# Citations

- `docs/knowledge/engineering/progress/2026-07-07T042107Z-public-example-tail-surface-closure.md`
- `apps/fret-examples/tests/basic_plot_demos_surface.rs`
