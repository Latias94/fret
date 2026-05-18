# Pressable Clean Geometry Propagation v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

`retained-layout-orchestration-v1` closed after landing the smallest proven `Semantics`
clean-geometry propagation fast path. Its after evidence still shows `Pressable` as a top retained
layout owner in the same resize-jitter scenario, but `Pressable` is not only a visual wrapper. It
owns hit testing, focus participation, pointer capture, pressed state, hover state, and activation
hooks. Those side effects need a separate proof before the layout runtime treats `Pressable` as a
safe propagation step.

## Relevant Authority

- ADRs:
  - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
  - `docs/adr/0069-outside-press-and-dismissable-non-modal-overlays.md`
  - `docs/adr/0076-declarative-layout-performance-hardening.md`
- Existing docs:
  - `docs/workstreams/fret-ui-layout-architecture-audit-v1/CLOSEOUT_AUDIT_2026-05-18.md`
  - `docs/workstreams/retained-layout-orchestration-v1/CLOSEOUT_AUDIT_2026-05-18.md`
  - `docs/workstreams/retained-layout-orchestration-v1/EVIDENCE_AND_GATES.md`
  - `docs/runtime-contract-matrix.md`
- Related workstreams:
  - `docs/workstreams/fret-ui-layout-architecture-audit-v1/`
  - `docs/workstreams/retained-layout-orchestration-v1/`
  - `docs/workstreams/ui-focus-overlay-fearless-refactor-v1/`

## Problem

`Pressable` already appears in `clean_geometry_node_contract(...)` as a pure
`PreserveLocalOrigins` wrapper, and its host-widget layout path uses the same positioned-container
shape as other wrapper elements. The execution-side support matrix does not yet allow `Pressable`
to use the clean-geometry propagation fast path, so a small width-only resize can still run avoidable
wrapper/subtree layout after the root Taffy solve is skipped.

The open risk is semantic, not geometric: if a future change skips the wrong work, pointer hit
targets, focus traversal, hover edges, pressed state, capture release, or activation coordinates may
drift from the authoritative current bounds.

## Target State

This lane should close with one of two outcomes:

- `Pressable` is proven safe for clean-geometry propagation, implemented as the smallest supported
  element addition, and locked by resize plus interaction side-effect tests.
- Or `Pressable` is left out of the fast path with a recorded no-change verdict explaining which
  side effect or attribution gap makes the optimization unsafe.

Either result must keep `fret-ui` as a mechanism layer. No component policy, shadcn recipe, or
ecosystem default behavior should move into the runtime.

Closeout result:

- `Pressable` is proven safe for the targeted clean-geometry propagation path.
- `ElementInstance::Pressable(_)` is now in the execution allowlist.
- The focused layout proof and `Pressable` interaction gates pass.
- Fresh UI Gallery resize-jitter evidence shows `Pressable` moved off the worst-frame layout
  hotspot list; the remaining owners are `ViewCache`, `Scroll`, and a small `Flex` owner.

## In Scope

- `crates/fret-ui/src/tree/layout/clean_geometry.rs`
- `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`
- Focused `Pressable` interaction tests, likely under:
  - `crates/fret-ui/src/declarative/tests/interactions/pressable.rs`
  - `crates/fret-ui/src/declarative/tests/layout/interactivity.rs`
  - `crates/fret-ui/src/tree/tests/pointer_move_hover.rs`
  - `crates/fret-ui/src/tree/tests/focus_scope.rs`
- Fresh or reused diag evidence from the UI Gallery code-editor resize-jitter script.

## Out Of Scope

- Broad clean-geometry expansion for `Scroll`, `ViewCache`, text wrapping, `Canvas`, or unknown
  component wrappers.
- Rewriting the layout model or replacing the current clean-geometry axes without fresh evidence.
- Moving interaction policy into `fret-ui`.
- shadcn recipe changes, component padding/spacing policy, or UI kit authoring API changes.
- GPU renderer, text shaping, and paint batching work.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The correct next lane is a narrow follow-on, not reopening `retained-layout-orchestration-v1`. | Confident | `docs/workstreams/retained-layout-orchestration-v1/CLOSEOUT_AUDIT_2026-05-18.md` says future `Pressable` work needs its own proof. | Reopening the closed lane would blur `Semantics` closeout evidence with a different side-effect owner. |
| `Pressable` is geometrically wrapper-like for the current target scenario. | Likely | `crates/fret-ui/src/declarative/host_widget/layout.rs` lays out `Pressable` through `layout_positioned_container_impl(...)`; `crates/fret-ui/src/tree/layout/clean_geometry.rs` classifies it as `PreserveLocalOrigins`. | If `Pressable` has unmodeled geometry behavior, the lane should record a no-change verdict. |
| `Pressable` has real runtime side effects that make it riskier than `Semantics`. | Confident | `layout.rs` sets hit-test, focus traversal, focusability, and clip flags; `event/pressable.rs` manages capture, pressed state, focus, hooks, and activation; `dispatch/hover.rs` derives hovered pressable targets. | A pure layout-only test would be insufficient and could bless stale interaction geometry. |
| The first code proof should be RED: show current small-resize behavior still relayouts a `Pressable` wrapper while preserving side-effect expectations. | Likely | RLO-030's RED note recorded leftover `layout_nodes_performed` when the fixture still had `Pressable` leaves. | If current code no longer reproduces that after unrelated changes, PGP-030 should pivot to an attribution/no-change note. |
| The after perf evidence is local orientation evidence, not a cross-machine baseline. | Confident | `retained-layout-orchestration-v1/EVIDENCE_AND_GATES.md` labels the resize-jitter bundle local orientation evidence. | The lane should avoid promising universal performance percentages before a fresh campaign exists. |

## Architecture Direction

Keep the proof inside the retained layout mechanism:

1. Treat `clean_geometry_node_contract(...)` as the declarative safety model.
2. Treat `clean_engine_geometry_propagation_supported_element(...)` as the execution allowlist.
3. Do not add `Pressable` to the execution allowlist until tests show that propagated bounds remain
   authoritative for hit testing, focus traversal, hover derivation, pressed state, capture release,
   and activation hooks.
4. If the proof lands, the implementation should be as small as the `Semantics` slice: one support
   matrix change plus targeted regression tests. Any larger redesign must start from a separate
   architecture decision.

## Closeout Condition

This lane can close when:

- fresh source audit and RED proof are recorded,
- either the minimal runtime change lands or a no-change verdict is documented,
- targeted layout and `Pressable` interaction gates pass,
- the workstream docs and `WORKSTREAM.json` reflect the shipped decision,
- and any remaining owners (`Scroll`, `ViewCache`, broader clean geometry) are split or explicitly
  deferred.

Status: Met on 2026-05-18. See
`docs/workstreams/pressable-clean-geometry-propagation-v1/CLOSEOUT_AUDIT_2026-05-18.md`.
