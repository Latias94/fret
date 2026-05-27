# `fret-node` Fearless Refactor (v1) - Handoff

Status: Active
Last updated: 2026-05-27

## Current State

This workstream remains the active lane for making `fret-node` the declarative-first,
controller/binding-first, editor-grade node graph surface for Fret. Recent work closed the
runtime/store contract hazards, retained canvas mirror cleanup, and three concrete declarative
overlay/add-on parity gates. The current risk is no longer the existence of the primitives; it is
consumer-facing drift where docs or examples might keep teaching older graph/view/controller
triplets or direct retained authoring.

## Active Task

- Task ID: FNDX-042.
- Owner: current Codex session.
- Status: DONE.
- Claim: declarative portal text cancel commands are available only for live portal nodes, return
  focus to the graph surface, and do not commit graph/store changes.
- Review: use `review-workstream` before accepting broader lane closure.
- Evidence:
  - `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs` adds
    `declarative_portal_text_cancel_returns_focus_to_surface_without_graph_commit`, which exercises
    real `UiTree` command availability/dispatch and asserts the focused node resolves back to the
    graph surface after cancel.
  - `ecosystem/fret-node/src/ui/editors/portal_text.rs` remains the declarative portal text editor
    command handler.
  - `ecosystem/fret-node/src/ui/portal_commands.rs` remains the portal text submit/cancel/step
    command protocol.
  - `docs/node-graph-xyflow-parity.md` now records the declarative portal text cancel focus-return
    outcome under viewport portals/window-space overlays.
  - Fresh gates passed:
    `cargo nextest run -p fret-node declarative_portal_text_cancel_returns_focus_to_surface_without_graph_commit`,
    `cargo check -p fret-node --features compat-retained-canvas --tests`, and
    `cargo fmt --check`.
  - Earlier closeout/package gates for FNDX-010 through FNDX-041 remain recorded in
    `EVIDENCE_AND_GATES.md`.

## Decisions Since Last Update

- Reuse this existing workstream instead of opening a duplicate XYFlow parity lane.
- Treat `docs/workstreams/standalone/fret-node-xyflow-parity.md` as the historical parity execution
  plan and `docs/node-graph-xyflow-parity.md` as the detailed map.
- Treat the current narrow task as a consumer-surface proof: binding-first docs plus a source-policy
  gate.
- Keep diff-first controlled sync out of the public helper surface for now; require workload
  evidence before adding a `replace_*_with_diff` API.
- Treat FNDX-030 as policy-placement closure, not a full declarative parity claim: remaining
  overlay behavior parity should be split into future focused conformance tasks.
- Keep this workstream active after closeout verification and split concrete overlay behavior gates
  instead of marking the whole lane complete.
- FNDX-040 chose input transparency as the first concrete declarative overlay parity gate and did
  not widen the overlay policy surface.
- FNDX-041 chose motion anchoring as the second concrete declarative overlay parity gate and kept
  the behavior on existing hover-anchor and overlay-spec seams.
- FNDX-042 chose declarative portal text cancel focus return as the next concrete add-on behavior
  gate and kept the implementation on the existing portal command/editor seams.

## Blockers

- None for FNDX-042.

## Next Recommended Action

- Pick the next overlay parity behavior only if it has a concrete observable outcome and a narrow
  gate. Good candidates are a broader declarative add-on dismissal/focus-return case that includes
  a mounted overlay subtree, or anchoring parity for another specific overlay under motion.
