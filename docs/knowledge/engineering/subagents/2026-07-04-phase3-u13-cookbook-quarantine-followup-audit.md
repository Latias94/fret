---
type: Subagent Finding
title: Phase 3 U13 cookbook quarantine follow-up audit
tags: fret,phase3,u13,cookbook,subagent
timestamp: 2026-07-04
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
git_branch: feat/ui-framework-phase2-refactor
subagent_id: 019f2a73-1db6-7120-9e17-ff6fa325931b
---

# Finding

Readonly explorer `019f2a73-1db6-7120-9e17-ff6fa325931b` audited the remaining Phase 3 U13
`ADVANCED_MANUAL_SURFACES` records after the IMUI plot binding migration. It made no edits.

The audit confirmed that every remaining quarantine record still has a real raw seam; none can be
removed by deleting allowlist entries alone.

# Evidence

Highest-priority public-looking remaining surfaces:

- `drag_basics.rs`: raw `advanced::prelude`, `fret_core`, `fret_runtime`, `fret_ui`, raw pointer
  action host, and `PointerRegionProps`.
- `async_inbox_basics.rs`: raw `AppUiRawActionNotifyExt`, `Model<T>`, dispatcher/inbox registry,
  and `AnyElement`.
- `canvas_pan_zoom_basics.rs`: raw pointer action host plus canvas painter/scene-operation seams.

Surfaces to keep advanced/manual for now:

- `chart_interactions_basics.rs`: still crosses manual command registry, `ElementContext`,
  `UiTree`, and shared `Model<ChartEngine>`.
- `docking_basics.rs`, `embedded_viewport_basics.rs`, `external_texture_import_basics.rs`,
  `gizmo_basics.rs`, and `utility_window_materials_windows.rs`: interop/manual driver or
  viewport/render target proof surfaces.
- `customv1_basics.rs`, `compositing_alpha_basics.rs`, and `image_asset_cache_basics.rs`:
  renderer/custom effect/cache proof surfaces.

# Recommendation

Migrate in this order:

1. Add a narrow app-facing pointer/drag wrapper and move `drag_basics.rs` to default clean.
2. Add a narrow app-facing async inbox / background action helper before migrating
   `async_inbox_basics.rs`.
3. Reuse the pointer wrapper for `canvas_pan_zoom_basics.rs`, then add canvas-specific authoring
   wrappers without widening `fret::app::prelude::*`.

# Disposition

Implemented item 1 in the
[Phase 3 U13 pointer drag facade](../progress/2026-07-04-phase3-u13-pointer-drag-facade.md)
slice. Items 2 and 3 remain active U13 follow-ups.
