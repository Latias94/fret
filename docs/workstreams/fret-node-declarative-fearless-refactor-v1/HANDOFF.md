# `fret-node` Fearless Refactor (v1) - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

This workstream remains the active lane for making `fret-node` the declarative-first,
controller/binding-first, editor-grade node graph surface for Fret. Recent work closed the retained
canvas mirror cleanup, concrete declarative overlay/add-on parity gates, and now the first
store/view-policy hazard found in the 2026-05-28 `fret-node` architecture audit and the first
default declarative public-extension decision. The current risk is consumer-facing drift where
public extension or store surfaces look authoritative but bypass the store's contracts or imply
unimplemented view-policy parity.

## Active Task

- Task ID: FNDX-045.
- Owner: current Codex session.
- Status: DONE.
- Claim: the default declarative surface now exposes narrow `NodeGraphSurfaceProps.edge_types` and
  `NodeGraphSurfaceProps.skin` hooks for edge render hints/custom paint paths and paint-only skin
  policy, while custom `NodeGraphPresenter` remains deferred from the default app-facing surface.
- Review: use `review-workstream` before accepting broader lane closure.
- Evidence:
  - `ecosystem/fret-node/src/ui/declarative/paint_only.rs` carries the new `edge_types` / `skin`
    props and passes them into frame preparation.
  - `ecosystem/fret-node/src/ui/declarative/paint_only/cache.rs` applies `edgeTypes` then skin to
    edge draw hints, includes their revisions in edge paint cache keys, and uses custom paths for
    default declarative paint/culling.
  - `ecosystem/fret-node/src/surface_policy_tests.rs` now carries
    `default_declarative_surface_exposes_edge_types_and_skin_without_custom_presenter`.
  - Fresh gates passed:
    `cargo fmt --check`,
    `cargo check -p fret-node --tests` and
    `cargo nextest run -p fret-node edges_cache_key_changes_when_edge_types_or_skin_revision_changes declarative_edge_types_feed_default_surface_edge_draws declarative_skin_refines_edge_draw_hints_after_edge_types default_declarative_surface_exposes_edge_types_and_skin_without_custom_presenter`,
    `cargo check -p fret-node --all-features --tests`,
    `cargo check -p fret-node --no-default-features`, `python3 tools/check_layering.py`,
    `git diff --check`, and `cargo nextest run -p fret-node`.
  - Earlier closeout/package gates for FNDX-010 through FNDX-044 remain recorded in
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
- FNDX-044 chose the smallest store/view-policy refactor from the 2026-05-28 audit: delete the
  public raw `view_state_mut` bypass first, then leave broader presenter/skin/edge-type wiring for
  a follow-up slice.
- FNDX-045 wires `NodeGraphEdgeTypes` and `NodeGraphSkin` into the default declarative edge paint
  path, but does not expose custom `NodeGraphPresenter` as a default prop because that trait still
  mixes geometry, labels, context menus, search/insert policy, and rendering hints.

## Blockers

- None for FNDX-045.

## Next Recommended Action

- Pick the next view-policy/public-extension slice with a concrete gate. The strongest candidate is
  to split the broad `NodeGraphPresenter` contract into narrower default-path contracts for labels,
  geometry, menus, and insertion/search policy, or to close the custom edge path hit-testing gap
  with an explicit spatial-index input.
