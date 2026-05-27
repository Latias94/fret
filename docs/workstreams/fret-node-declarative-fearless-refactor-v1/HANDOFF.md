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

- Task ID: FNDX-041.
- Owner: current Codex session.
- Status: DONE.
- Claim: diagnostics hover-tooltip overlay placement follows drag-adjusted hover anchors when portal
  bounds are disabled or unavailable, so tooltip anchoring does not drift back to stale pre-drag
  node bounds.
- Review: use `review-workstream` before accepting broader lane closure.
- Evidence:
  - `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs` adds
    `declarative_hover_tooltip_overlay_tracks_dragged_anchor_when_portals_disabled`, which composes
    hover-anchor sync, portal-disabled fallback, and final tooltip spec placement.
  - `ecosystem/fret-node/src/ui/declarative/paint_only/hover_anchor.rs` remains the drag-adjusted
    hover-anchor authority.
  - `ecosystem/fret-node/src/ui/declarative/paint_only/overlay_elements.rs` remains the final
    tooltip overlay spec authority.
  - `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs` adds
    `declarative_overlay_layer_is_input_transparent_over_canvas_region`, which fails if an overlay
    layer can intercept pointer input before the canvas region.
  - `ecosystem/fret-node/src/ui/declarative/paint_only/overlays.rs` keeps the declarative overlay
    layer behind `hit_test_gate(false)`.
  - `docs/node-graph-xyflow-parity.md` now names declarative overlay-layer input transparency next
    to the retained portal-root input transparency contract.
  - Fresh gates passed:
    `cargo nextest run -p fret-node declarative_hover_tooltip_overlay_tracks_dragged_anchor_when_portals_disabled`,
    `cargo nextest run -p fret-node declarative_overlay_layer_is_input_transparent_over_canvas_region`,
    `cargo check -p fret-node --features compat-retained-canvas --tests`, and
    `cargo fmt --check`.
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
- Keep this workstream active after closeout verification and split concrete overlay behavior gates
  instead of marking the whole lane complete.
- FNDX-040 chose input transparency as the first concrete declarative overlay parity gate and did
  not widen the overlay policy surface.
- FNDX-041 chose motion anchoring as the second concrete declarative overlay parity gate and kept
  the behavior on existing hover-anchor and overlay-spec seams.

## Blockers

- None for FNDX-041.

## Next Recommended Action

- Pick the next overlay parity behavior only if it has a concrete observable outcome and a narrow
  gate. Good candidates are dismissal/focus-return behavior for an actual declarative add-on or
  anchoring parity for a specific overlay under motion.
