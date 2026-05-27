# `fret-node` Fearless Refactor (v1) - Handoff

Status: Active
Last updated: 2026-05-27

## Current State

This workstream remains the active lane for making `fret-node` the declarative-first,
controller/binding-first, editor-grade node graph surface for Fret. Recent work closed the
runtime/store contract hazards, retained canvas mirror cleanup, and four concrete declarative
overlay/add-on parity gates. The current risk is no longer the existence of the primitives; it is
consumer-facing drift where docs or examples might keep teaching older graph/view/controller
triplets or direct retained authoring.

## Active Task

- Task ID: FNDX-043.
- Owner: current Codex session.
- Status: DONE.
- Claim: mounted declarative rename overlays close on Escape without committing a graph transaction
  and restore focus to the graph surface target.
- Review: use `review-workstream` before accepting broader lane closure.
- Evidence:
  - `ecosystem/fret-node/src/ui/overlays/rename_declarative.rs` already carries
    `rename_managed_host_escape_closes_without_transaction_and_restores_focus`, which exercises a
    mounted declarative text-input overlay subtree.
  - `ecosystem/fret-node/src/ui/overlays/rename_command.rs` remains the Escape/cancel command
    protocol for the rename host.
  - `ecosystem/fret-node/src/ui/overlays/rename_lifecycle.rs` remains the lifecycle/focus-restore
    policy seam.
  - `docs/node-graph-xyflow-parity.md` now records mounted declarative overlay dismissal/focus
    return under viewport portals/window-space overlays.
  - Fresh gates passed:
    `cargo nextest run -p fret-node rename_managed_host_escape_closes_without_transaction_and_restores_focus`.
  - Earlier closeout/package gates for FNDX-010 through FNDX-042 remain recorded in
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
- FNDX-043 promoted the existing mounted declarative rename overlay Escape/focus-return gate into
  the parity/evidence map instead of duplicating an equivalent test.

## Blockers

- None for FNDX-043.

## Next Recommended Action

- Pick the next overlay parity behavior only if it has a concrete observable outcome and a narrow
  gate. Good candidates are package follow-up after FNDX-043, or anchoring/focus parity for another
  specific overlay under motion.
