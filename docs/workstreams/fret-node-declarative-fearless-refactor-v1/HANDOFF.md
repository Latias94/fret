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

- Task ID: FNDX-030
- Owner: current Codex session
- Files:
  - `ecosystem/fret-node/src/surface_policy_tests.rs`
  - `ecosystem/fret-node/src/ui/overlays/toolbar_policy.rs`
  - `ecosystem/fret-node/src/ui/overlays/toolbars_declarative.rs`
  - `ecosystem/fret-node/src/ui/canvas/state/state_overlay_policy.rs`
  - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/ui/overlay.rs`
  - `ecosystem/fret-node/src/ui/canvas/widget/searcher_ui/overlay.rs`
  - `docs/workstreams/fret-node-declarative-fearless-refactor-v1/*`
- Validation:
  - `cargo nextest run -p fret-node --features compat-retained-canvas overlay_menu_toolbar_policy_ownership_stays_on_named_seams`
  - `cargo nextest run -p fret-node --features compat-retained-canvas overlay_policy_modules_compile_without_retained_canvas_compat default_overlay_policy_surfaces_stay_off_retained_bridge`
  - `cargo fmt --check`
- Status: DONE
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
  - Formatting passed on 2026-05-27.

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

## Blockers

- None for FNDX-030.

## Next Recommended Action

- Either run the heavier closeout gates (`cargo nextest run -p fret-node` and
  `cargo check -p fret-node --features compat-retained-canvas --tests`) before lane closeout, or
  split the next follow-up into a concrete declarative overlay parity/conformance task instead of
  reopening policy placement.
