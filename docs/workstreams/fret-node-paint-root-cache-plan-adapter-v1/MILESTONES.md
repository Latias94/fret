# Fret Node Paint Root Cache Plan Adapter v1 - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Scope And Evidence Freeze

Exit criteria:

- Cache-plan-only scope is explicit.
- Non-goals exclude frame setup and scene emission.
- Gate set is recorded.

Status: Complete.

Evidence:

- `docs/workstreams/fret-node-paint-root-cache-plan-adapter-v1/DESIGN.md`
- `docs/workstreams/fret-node-paint-root-cache-plan-adapter-v1/EVIDENCE_AND_GATES.md`

## M1 - Cache-Plan Adapter Proof

Exit criteria:

- A named retained-agnostic adapter seam exists for host access, bounds, and scale factor.
- Retained `PaintCx` binding is isolated.
- Source-policy tests keep cache-plan adapter helpers off retained Cx names.

Primary gates:

- `cargo test -p fret-node --features compat-retained-canvas paint_root_cache_plan_adapter`
- `cargo check -p fret-node`
- `cargo check -p fret-node --features compat-retained-canvas`

Status: Complete.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cache_plan_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cache_plan_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cache_plan.rs`
- `ecosystem/fret-node/src/lib.rs`

## M2 - Closeout Or Follow-On Split

Exit criteria:

- Evidence gates are fresh.
- Remaining paint-root work is deferred or split into a narrower follow-on.
- `WORKSTREAM.json` status is updated.

Status: Complete.

Evidence:

- `docs/workstreams/fret-node-paint-root-cache-plan-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1/`
