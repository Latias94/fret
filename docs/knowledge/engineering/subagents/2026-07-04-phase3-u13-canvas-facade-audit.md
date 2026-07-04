---
type: Subagent Finding
title: Phase 3 U13 canvas facade audit
tags: fret,phase3,u13,canvas,subagent
timestamp: 2026-07-04
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
subagent_id: 019f2aca-f2fb-7d22-8bbd-e072954b6562
git_branch: feat/ui-framework-phase2-refactor
---

# Finding

Read-only explorer `019f2aca-f2fb-7d22-8bbd-e072954b6562` confirmed that
`canvas_pan_zoom_basics.rs` should leave the advanced cookbook quarantine after the new
`fret::canvas` facade lands.

# Evidence

- The remaining raw seams were app-authoring leaks, not necessary cookbook surface:
  `advanced::prelude::*`, `component::prelude::*`, raw `Model<T>`, `fret_runtime::DefaultAction`,
  raw `OnPointer*` callback types, `UiPointerActionHost`, `CanvasPainter`, `CanvasCachePolicy`,
  direct `fret_core::*`, and direct `fret_canvas::*`.
- The minimum needed facade is `fret::canvas` with `PanZoomCanvas`, `AppCanvasPainter`,
  app-facing paint/geometry helpers, plus `PointerActionCx::bounds()` for hit-testing.
- Canvas should stay explicit and not enter `fret::app::prelude::*`.
- `ResetNode` should clear `node_drag` as well as origin/count to avoid a stale drag state after
  reset during an active left-drag.

# Recommendation

Move `canvas_pan_zoom_basics.rs` to `DEFAULT_AUTHORING_SURFACES`, add a default-surface ban on
direct `fret_canvas::`, and keep low-level `fret_canvas::ui::*`, raw `CanvasProps`,
`PointerRegionProps`, `CanvasCachePolicy`, and raw pointer callbacks available only as advanced or
ecosystem component implementation seams.

# Disposition

Accepted in the canvas facade slice.

# Citations

- [Cookbook canvas example](../../../apps/fret-cookbook/examples/canvas_pan_zoom_basics.rs)
- [Fret canvas facade](../../../ecosystem/fret/src/view/canvas.rs)
- [Surface policy gate](../../../tools/check_surface_policy.py)
