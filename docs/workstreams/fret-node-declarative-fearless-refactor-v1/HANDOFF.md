# `fret-node` Fearless Refactor (v1) - Handoff

Status: Active
Last updated: 2026-05-27

## Current State

This workstream remains the active lane for making `fret-node` the declarative-first,
controller/binding-first, editor-grade node graph surface for Fret. Recent work closed the
runtime/store contract hazards and retained canvas mirror cleanup. The current risk is no longer the
existence of the primitives; it is consumer-facing drift where docs or examples might keep teaching
older graph/view/controller triplets or direct retained authoring.

## Active Task

- Task ID: none currently in progress.
- Last closed task: FNDX-030.
- Next proposed task: FNDX-040, a concrete declarative overlay parity/conformance slice.
- Status: FNDX-010, FNDX-020, and FNDX-030 are done and closeout-verified for their bounded
  claims. The full workstream remains active because FNDX-030 intentionally closed ownership
  placement, not full declarative overlay parity.
- Review: use `review-workstream` before accepting broader lane closure.
- Evidence:
  - `ecosystem/fret-node/src/surface_policy_tests.rs` locks the FNDX-030 ownership boundary:
    toolbar public policy types stay in `toolbar_policy.rs`, menu/searcher policy enums stay in
    `state_overlay_policy.rs`, and retained menu/searcher lifecycle writes route through named
    overlay seams.
  - Adjacent overlay policy gates still pass with `compat-retained-canvas` enabled.
  - Review/verify follow-up passed the package/boundary gates:
    `cargo check -p fret-node --no-default-features`,
    `cargo check -p fret-node --features compat-retained-canvas`, and
    `python3 tools/check_layering.py`.
  - Closeout gates passed on 2026-05-27:
    `cargo fmt --check`,
    `cargo nextest run -p fret-node`, and
    `cargo check -p fret-node --features compat-retained-canvas --tests`.

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
- Keep this workstream active after closeout verification and split the next step as FNDX-040
  instead of marking the whole lane complete.

## Blockers

- None for FNDX-030.

## Next Recommended Action

- Start FNDX-040: choose one observable overlay behavior that matters to the XYFlow-style consumer
  story (anchoring, input transparency, dismissal, focus return, or add-on placement), then add a
  focused declarative behavior gate or paired declarative-vs-retained conformance gate.
