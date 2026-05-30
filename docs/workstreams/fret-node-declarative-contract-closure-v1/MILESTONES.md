# `fret-node` Declarative Contract Closure v1 - Milestones

Status: Closed
Last updated: 2026-05-28

## M0 - Scope And Evidence Freeze

Status: Complete (2026-05-28).

Exit criteria:

- The new workstream has DESIGN, TODO, MILESTONES, EVIDENCE_AND_GATES, HANDOFF, and WORKSTREAM.json.
- The workstream catalog can discover the lane.
- The first executable task is bounded and does not reopen closed retained lanes.

Primary evidence:

- `docs/workstreams/fret-node-declarative-contract-closure-v1/DESIGN.md`
- `docs/workstreams/fret-node-declarative-contract-closure-v1/TODO.md`
- `docs/workstreams/fret-node-retained-exit-and-parity-seams-v1/CLOSEOUT_AUDIT_2026-05-28.md`

## M1 - Retained Current-Fact Drift Closure

Status: Complete (2026-05-28).

Exit criteria:

- Standalone XyFlow gap analysis no longer recommends deleted `NodeGraphCanvas` as current usage.
- ADR 0135 is marked superseded rather than proposed current middleware guidance.
- ADR 0128 no longer lists node graph retained widget paths as current implementation examples.
- `fret-node` source-policy tests cover these high-risk docs.

Primary gates:

- `cargo nextest run -p fret-node --no-default-features retained_node_graph_current_guidance_stays_declarative`
- `python3 tools/check_workstream_catalog.py`

## M2 - Store Dispatch Commit Path

Status: Complete (2026-05-28).

Exit criteria:

- Profile and non-profile graph dispatch share one internal commit path.
- Middleware ordering, normalization, validation, history recording, and event publication stay
  covered by existing store tests.
- Public `NodeGraphStore` APIs remain stable unless a task explicitly updates docs and tests.

Primary gates:

- `cargo nextest run -p fret-node store_dispatch store_middleware store_rejects`
- `cargo nextest run -p fret-node --no-default-features`

## M3 - Binding Mirror Ownership

Status: Complete (2026-05-28).

Exit criteria:

- Binding mirrors are either shrunk or documented as advanced projections.
- Public guides remain binding/controller/declarative-first.
- No app-facing examples teach raw graph/view/config triplets as the ordinary path.

Primary gates:

- `cargo nextest run -p fret-node binding_surface controller_surface public_node_graph_guides`
- `cargo nextest run -p fret-node`

## M4 - Declarative Interaction Hook Contract

Status: Complete (2026-05-28).

Exit criteria:

- The obsolete retained middleware direction is replaced by a declarative hook contract or a focused
  design note.
- The hook boundary cannot mutate graph state outside `NodeGraphStore`.
- Broad ReactFlow hook facade work is split if it is not part of the proof.

Primary gates:

- focused hook contract test or compile gate
- `cargo nextest run -p fret-node`

## M5 - Paint-only Orchestration Split

Status: Complete (2026-05-28).

Exit criteria:

- At least one pure frame/scene planning extraction lands, or a negative audit records why the
  current module shape should remain until a better target exists.
- Any domain-neutral helper moved to `fret-canvas` has `fret-canvas` tests.

Primary gates:

- `cargo nextest run -p fret-node node_graph_surface cache paint_only`
- `cargo nextest run -p fret-canvas` when touched

## M6 - Closeout

Status: Complete (2026-05-28).

Exit criteria:

- `TODO.md`, `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and `HANDOFF.md` agree.
- Final gates have fresh evidence.
- Remaining semantic focus or ReactFlow facade work is closed, deferred, or split into follow-ons.
