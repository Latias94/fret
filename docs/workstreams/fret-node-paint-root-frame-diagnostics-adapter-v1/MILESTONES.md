# Fret Node Paint Root Frame Diagnostics Adapter v1 - Milestones

Status: Active
Last updated: 2026-05-25

## M0 - Scope Freeze

Exit criteria:

- Follow-on relationship to the closed frame clip lane is explicit.
- Non-goals exclude cache begin, viewport, clip, background, grid, tail, and cached/immediate
  passes.
- Gate set is recorded.

Status: Complete.

Evidence:

- `docs/workstreams/fret-node-paint-root-frame-diagnostics-adapter-v1/DESIGN.md`
- `docs/workstreams/fret-node-paint-root-frame-diagnostics-adapter-v1/EVIDENCE_AND_GATES.md`

## M1 - Path-Cache Diagnostics Seam

Exit criteria:

- Path-cache diagnostics recording no longer directly reads retained `PaintCx` fields in
  `frame/cache.rs`.
- The retained diagnostics binding lives in a dedicated retained adapter module.
- Source-policy coverage locks the diagnostics adapter boundary.
- Cache begin, viewport, clip, background paint, grid paint, and tail cleanup remain out of scope.

Status: Complete.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame_diagnostics_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame_diagnostics_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame/cache.rs`
- `ecosystem/fret-node/src/lib.rs`
