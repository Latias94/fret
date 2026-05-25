# Fret Node Paint Root Pass Clip Adapter v1

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

`fret-node-paint-root-tail-cleanup-adapter-v1` closed root frame tail cleanup `PopClip` emission and
left pass-level scene access as the next small paint-root operation-family candidate.

## Relevant Authority

- `docs/workstreams/fret-node-paint-root-tail-cleanup-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/workstreams/fret-node-paint-root-frame-grid-diagnostics-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/immediate_pass.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_pass.rs`

## Problem

`paint_root/immediate_pass.rs` still routes static group, selected-group overlay, and static node
painting by reading retained `PaintCx` scene sink fields directly:

- `cx.scene`
- `cx.services`
- `cx.scale_factor`

That makes the pass router depend on retained paint context shape instead of a named pass-scene
contract.

The initial candidate name mentions cached and immediate pass clip/scene emission. The source audit
narrows the landable slice:

- `cached_pass.rs` itself has no direct `cx.scene` access.
- Cached scene access lives deeper in cached group/node/edge internals and should be split into
  cache-layer follow-ons.
- `immediate_pass.rs` is the pass-router file with direct scene sink reads and is the right owner for
  this lane.

## Target State

- Immediate pass static scene routing is behind a named pass scene adapter.
- `immediate_pass.rs` no longer reads `cx.scene`, `cx.services`, or `cx.scale_factor` for static
  group/node scene emission.
- The retained `PaintCx` binding owns the retained field reads and calls the existing static paint
  helpers.
- `cached_pass.rs` remains direct-scene-free at this level; deeper cached internals remain explicit
  follow-ons.
- Source-policy coverage locks the adapter boundary.

## In Scope

- `paint_root/immediate_pass.rs`
- `paint_root/cached_pass.rs` audit only
- new pass scene adapter modules under `paint_root/`
- source-policy coverage in `ecosystem/fret-node/src/lib.rs`
- workstream evidence and gates

## Out Of Scope

- cached group/node/edge internal scene replay,
- edge paint routing,
- overlay layer paint routing,
- root frame clip/background/tail emission,
- grid plan or chrome hint routing,
- public scene schema changes,
- visual behavior changes.

## Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| `cached_pass.rs` has no direct retained scene sink access at the pass-router level. | Confident | Source audit of `paint_root/cached_pass.rs`. | If a direct scene read appears, include it in this seam before closeout. |
| The smallest useful seam is immediate static group/node scene routing. | Likely | `immediate_pass.rs` directly reads `cx.scene`, `cx.services`, and `cx.scale_factor` for static groups and nodes. | Split a narrower immediate-only lane if edge/overlay routing proves coupled. |
| The adapter should expose named pass operations rather than a raw `Scene` handle. | Likely | Prior frame adapters hide scene emission instead of making routers field-aware. | If helper visibility blocks this, fall back to a minimal scene/services/scale adapter and document the weaker seam. |

## Architecture Direction

Prefer a narrow action adapter:

- `paint_root_pass_groups_static(cx, canvas, groups, zoom)`
- `paint_root_pass_groups_selected_overlay(cx, canvas, groups, zoom)`
- `paint_root_pass_nodes_static(cx, canvas, render, zoom)`

The retained binding should read `PaintCx.scene`, `PaintCx.services`, and `PaintCx.scale_factor`.
The pass router should keep pass ordering and edge/overlay routing only.

## Closeout Condition

This lane can close when immediate pass static scene sink access is isolated behind the adapter,
`cached_pass.rs` remains direct-scene-free at the pass-router level, source-policy coverage locks
the seam, and validation gates pass.

## Closeout State

Closed on 2026-05-25 with `CLOSEOUT_AUDIT_2026-05-25.md`. Immediate pass static scene routing now
uses the pass scene adapter seam. Cached static layer scene replay, cached edge replay, immediate
edge paint routing, immediate overlay routing, grid plan, and chrome hint routing remain separate
follow-on candidates.
