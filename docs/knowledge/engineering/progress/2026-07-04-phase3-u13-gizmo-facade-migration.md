---
type: Work Progress
title: Phase 3 U13 gizmo facade migration
tags: fret,phase3,u13,cookbook,facade,gizmo,canvas,pointer
timestamp: 2026-07-04
related_plan: docs/plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

Phase 3 U13 migrated `apps/fret-cookbook/examples/gizmo_basics.rs` from retained/raw UI seams to
the default app-facing authoring surface. The migration first filled the public facade gap that the
example exposed, then removed `gizmo_basics.rs` from the advanced/manual quarantine.

# Design Decision

`gizmo_basics.rs` should not stay advanced just because editor-style drawing needs wheel input,
stable canvas keys, vector paths, and theme access. Those are reusable app-facing canvas needs, not
retained-tree ownership needs. The facade added for this slice is intentionally narrow:

- `fret::pointer::PointerRegion` exposes `on_wheel(...)` alongside pointer down/move/up.
- `fret::canvas` exposes `Canvas`, `CanvasSurface`, `AppCanvasPainter`, path/key helpers, and core
  paint/geometry types needed by custom editor canvases.
- `AppCanvasPainter` exposes theme snapshots, stable key helpers, and vector path paint methods
  without requiring authors to import raw `fret_ui::canvas::CanvasPainter`.

# Changed Files

- `ecosystem/fret/src/view/pointer.rs`
- `ecosystem/fret/src/view/canvas.rs`
- `ecosystem/fret/src/view.rs`
- `ecosystem/fret/src/lib.rs`
- `apps/fret-cookbook/Cargo.toml`
- `apps/fret-cookbook/examples/gizmo_basics.rs`
- `apps/fret-cookbook/src/lib.rs`
- `docs/crate-usage-guide.md`
- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

# Verification

- `cargo nextest run -p fret --features canvas canvas_builder_defaults_to_full_size_and_accepts_cache_policy root_surface_exposes_explicit_style_and_icon_modules --no-fail-fast`
- `cargo nextest run -p fret --features canvas usage_docs_prefer_grouped_app_ui_actions --no-fail-fast`
- `cargo nextest run -p fret-cookbook migrated_basics_examples_use_the_new_app_surface selected_cookbook_examples_prefer_handle_first_tracked_reads --no-fail-fast`
- `cargo check -p fret-cookbook --features cookbook-gizmo,cookbook-diag --example gizmo_basics`

# Next Action

Continue U13 by reassessing the remaining advanced cookbook/example surfaces. Good next candidates
are the surfaces that still have actual platform/window/driver ownership, not just missing facade
helpers: docking, embedded viewport, external texture import, utility window materials, and the
remaining example harness classifications.

# Citations

- [Phase 3 retained bridge deletion plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [Crate usage guide](../../../crate-usage-guide.md)
