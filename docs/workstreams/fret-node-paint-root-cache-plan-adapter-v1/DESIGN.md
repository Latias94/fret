# Fret Node Paint Root Cache Plan Adapter v1

Status: Active
Last updated: 2026-05-25

## Why This Lane Exists

`fret-node-paint-prepaint-adapter-v1` closed after proving the prepaint cull-window seam and auditing
paint root scope. The audit found that `canvas.paint_root(cx)` is too broad for one adapter, but
`paint_root/cache_plan.rs` is a coherent next slice: it uses host access, bounds, and scale factor to
prepare derived output and static-cache planning without directly emitting scene ops.

## Relevant Authority

- `docs/workstreams/fret-node-paint-prepaint-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/workstreams/fret-node-paint-prepaint-adapter-v1/PAINT_ROOT_SCOPE_AUDIT_2026-05-25.md`
- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cache_plan.rs`

## Problem

`prepare_paint_root_cache_plan` still takes retained `PaintCx` directly for host access, bounds, and
scale factor. That keeps cache-plan preparation tied to retained widget lifecycle context even
though this slice is mostly route preparation and cache-key planning.

## Target State

- Cache-plan preparation has a named retained-agnostic adapter seam.
- The retained `PaintCx` binding is explicit and isolated near the retained paint/cache-plan entry.
- Source-policy tests prevent cache-plan adapter helpers from directly naming retained contexts.
- Frame setup, scene mutation, static layer replay/store, cached/immediate paint passes, and tail
  cleanup remain outside this lane.

## In Scope

- Adapter seam for host access, bounds, and scale factor used by paint-root cache-plan preparation.
- Source-policy tests for the cache-plan adapter surface.
- Focused `fret-node` default and `compat-retained-canvas` checks.

## Out Of Scope

- Rewriting the full paint root.
- Moving scene emission behind an adapter.
- Static layer replay/store refactors.
- Grid/background/frame diagnostics adapters.
- Public style, skinning, or renderer contract changes.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Cache-plan prep is the smallest coherent paint proof. | Likely | NPA-030 split table points to `paint_root/cache_plan.rs` as host/bounds/scale-factor only and no direct scene emission. | Split again into host/bounds key calculation only. |
| Scene emission must stay out of the first task. | Confident | Cached/immediate pass files take `scene`, `services`, and cache replay/store paths directly. | A broad adapter would repeat the NPA-030 rejected shape. |
| Existing source-policy tests can host the new gate. | Confident | `ecosystem/fret-node/src/lib.rs` already contains `paint_prepaint_adapter` source-policy coverage. | Add a dedicated test name/filter if source grouping becomes noisy. |

## Architecture Direction

Mirror the event runtime and prepaint cull-window adapter pattern: keep retained-agnostic route
preparation in a named helper module, and keep the concrete `PaintCx` implementation in an explicit
retained binding module.

## Closeout Condition

This lane can close when cache-plan preparation no longer directly depends on retained `PaintCx`,
source-policy coverage locks the boundary, and the next paint operation family is either deferred or
split into a new follow-on.
