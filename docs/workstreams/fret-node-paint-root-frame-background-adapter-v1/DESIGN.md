# Fret Node Paint Root Frame Background Adapter v1

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

`fret-node-paint-root-frame-diagnostics-adapter-v1` closed path-cache diagnostics recording and left
background paint as the next small paint-root frame operation-family candidate.

## Relevant Authority

- `docs/workstreams/fret-node-paint-root-frame-diagnostics-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/workstreams/fret-node-paint-root-frame-clip-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame/background.rs`

## Problem

`paint_canvas_background` still mixes two concerns:

- resolving the canvas chrome/background color policy from the node graph canvas and skin,
- emitting the retained scene quad through `PaintCx.scene`.

That retained scene write is separate from cache begin, viewport preparation, clip emission,
path-cache diagnostics, grid paint, tail cleanup, and cached/immediate passes.

## Target State

- Paint-root frame background scene emission is behind a named adapter seam.
- `frame/background.rs` no longer writes `cx.scene` or constructs `SceneOp::Quad` directly.
- The retained `PaintCx` binding for scene quad emission lives in a retained background binding
  module.
- The background adapter accepts only the minimal route output needed for emission: viewport rect and
  resolved background color.
- Source-policy coverage locks the background adapter boundary.

## In Scope

- `paint_root/frame/background.rs`
- new frame background adapter modules under `paint_root/`
- source-policy coverage in `ecosystem/fret-node/src/lib.rs`
- workstream evidence and gates

## Out Of Scope

- cache frame begin,
- viewport/bounds/render-cull,
- root frame clip emission,
- path-cache diagnostics,
- grid paint and grid tile diagnostics,
- edge label budget diagnostics,
- tail cleanup,
- cached/immediate passes,
- public scene schema changes,
- canvas chrome hint policy changes.

## Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Background paint is its own operation family. | Confident | Diagnostics closeout names background paint as a separate follow-on candidate. | Split or rename before implementation if it proves coupled to grid paint. |
| The seam should move scene emission, not chrome hint resolution. | Likely | Grid paint also uses `resolve_canvas_chrome_hint`; moving policy now would blur this narrow lane. | Open a separate chrome-hint route lane if repeated retained reads become the next blocker. |
| The adapter should stay scene-op agnostic. | Confident | Frame clip adapter already hides `SceneOp::PushClipRect` behind a retained binding. | If the adapter exposes `SceneOp`, it stops reducing retained paint coupling. |

## Architecture Direction

Prefer a narrow action adapter:
`paint_root_frame_background(cx, viewport_rect, background_color)`. `frame/background.rs`
should own chrome hint policy and fallback color selection; the retained binding should own
`SceneOp::Quad` construction and scene mutation.

## Closeout Condition

This lane can close when background scene emission is isolated behind the adapter, source-policy
coverage locks the seam, and validation gates pass.

## Closeout State

Closed on 2026-05-25 with `CLOSEOUT_AUDIT_2026-05-25.md`. Background scene emission now uses the
frame background adapter seam. Grid paint, tail cleanup, cached/immediate pass clip emission, grid
tile diagnostics, edge label budget diagnostics, and chrome hint routing remain separate follow-on
candidates.
