# Fret Node Paint Root Frame Setup Adapter v1

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

`fret-node-paint-root-cache-plan-adapter-v1` moved cache-plan route inputs behind a named adapter
seam. The next paint-root dependency is frame setup, but `paint_root/frame.rs` mixes several
operation families. This lane starts with a source audit and split decision before any broad frame
adapter exists.

## Relevant Authority

- `docs/workstreams/fret-node-paint-root-cache-plan-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/workstreams/fret-node-paint-prepaint-adapter-v1/PAINT_ROOT_SCOPE_AUDIT_2026-05-25.md`
- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame.rs`

## Problem

`prepare_paint_root_frame` still takes retained `PaintCx` directly. The function currently combines
cache frame begin, diagnostics recording, bounds/viewport math, render-cull calculation, clip scene
emission, background paint, and grid paint. A single broad adapter would repeat the paint-root scope
mistake by hiding too many operations behind one trait.

## Target State

- Frame setup operation families are audited and split.
- The first implementation candidate is selected only after the audit.
- Frame setup does not gain a broad `PaintCx` adapter unless the audit proves a single coherent
  seam.

## In Scope

- Source audit of `paint_root/frame.rs`, `paint_root/frame/cache.rs`, and
  `paint_root/frame/background.rs`.
- Split recommendation for bounds/viewport, cache stats diagnostics, clip emission, background
  paint, and grid paint.
- Optional first seam only if it is narrow and does not include scene emission.

## Out Of Scope

- Static layer replay/store.
- Cached/immediate scene pass emission.
- Tail prune/cleanup.
- Public style, skinning, renderer, or diagnostics registry contract changes.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Frame setup is not one operation family. | Confident | `frame.rs` combines cache begin, diagnostics, viewport math, clip, background, and grid. | If one seam is truly coherent, implement it after the audit. |
| Bounds/viewport math is the likely smallest first proof. | Likely | It uses `cx.bounds` and returns `PaintRootFrameViewport` without needing `scene` or `services`. | If diagnostics is smaller, split diagnostics first. |
| Scene emission should be deferred. | Confident | Clip/background/grid write to `cx.scene` and call paint helpers. | Scene adapters should be their own lane or later task. |

## Architecture Direction

Audit first. If a first implementation is selected, prefer route-input seams that remove direct
`PaintCx` reads from frame preparation without moving scene mutation.

## Closeout Condition

This lane can close when frame setup has either one narrow adapter proof or a documented split into
the next paint-root follow-on.

## Closeout State

Closed on 2026-05-25 with `CLOSEOUT_AUDIT_2026-05-25.md`. Bounds/viewport/render-cull route inputs
now use the frame viewport adapter seam. Diagnostics, clip scene emission, background paint, and
grid paint remain split follow-on candidates.
