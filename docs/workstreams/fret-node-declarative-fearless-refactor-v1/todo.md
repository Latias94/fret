# `fret-node` Fearless Refactor (v1) - TODO

This tracker is intentionally biased toward small, reviewable slices. Keep items concrete enough to
land in code review; move design discussion back to `README.md` if a TODO turns into prose.

## Cross-cutting guardrails

- [x] Keep `compat-retained-canvas` out of default features.
- [x] Keep the compatibility retained path feature-gated and explicitly named.
- [x] Keep the default lightweight declarative demo path (`node_graph_demo`).
- [ ] Keep the workstream docs aligned with the actual public recommendation after each milestone.
- [ ] Update ADR alignment or ADR text when a hard contract changes rather than hiding the change in
      implementation-only diffs.

## Cross-cutting red lines

- [ ] Do not add new long-term public APIs that require retained widget types.
- [ ] Do not bless direct `Graph` mutation as the editor-grade commit path for declarative surfaces.
- [ ] Do not expand `NodeGraphViewState` with more policy or tuning fields.
- [ ] Do not solve recipe/policy gaps by smuggling new defaults into mechanism code.

## M0 - Decision gates and inventory

- [x] Reframe the workstream docs around architecture closure rather than a paint-only lab log.
- [x] Preserve the historical folder path to avoid breaking references.
- [ ] Add one short "current hazards" section with evidence anchors for:
  - direct graph mutation in the declarative path,
  - overgrown `NodeGraphViewState`,
  - missing controller facade.
- [ ] Add one short reviewer checklist to the README so a reviewer can verify the intended posture
      in under five minutes.

## M1 - Public posture and API closure

- [ ] State explicitly in docs which path is recommended **today** for shipping editor-grade usage.
- [ ] State explicitly in docs which path is the **target** posture after this refactor.
- [ ] State explicitly in docs when `compat-retained-canvas` is acceptable.
- [ ] Document that direct retained `NodeGraphCanvas` authoring is internal-only except for tests,
      diagnostics, or temporary compatibility harnesses.
- [ ] Audit examples and internal guides for wording that still suggests retained authoring is the
      normal downstream path.
- [ ] Decide whether to introduce a future canonical wrapper name such as `node_graph_surface(...)`
      over the existing paint-only / compat-retained milestones.

## M2 - State boundary split

- [ ] Shrink `NodeGraphViewState` to true view state only:
  - pan
  - zoom
  - selected nodes/edges/groups
  - draw order
- [ ] Introduce a separate interaction config type for:
  - selection/connect/drag toggles
  - connection mode
  - key semantics and activation policy
  - interaction defaults that are not pure view state
- [ ] Introduce a separate runtime tuning type for:
  - spatial index tuning
  - cache prune tuning
  - expensive runtime knobs
- [ ] Decide where these new types live and who owns persistence for them.
- [ ] Design the migration/compat strategy for existing serialized `NodeGraphViewState` payloads.
- [ ] Update store code and tests so the new boundary is explicit in subscriptions and controlled
      synchronization.
- [ ] Add focused tests for serialization migration and store behavior after the split.

## M3 - Controller / instance facade

- [ ] Introduce a thin controller or instance facade over store/lookups/view helpers.
- [ ] Make the facade responsible for the ergonomic app-facing surface for:
  - viewport manipulation
  - fit-view / set-center
  - controlled graph replacement / synchronization
  - common graph queries
  - canonical graph update entry points
- [ ] Decide whether `view_queue` stays as the transport for imperative viewport requests or becomes
      an internal detail of the controller.
- [ ] Decide whether `edit_queue` stays public, becomes controller-owned, or is limited to internal
      composition seams.
- [ ] Add a clear mapping from XyFlow-style expectations to the controller API:
  - viewport helpers
  - get node/handle connections
  - update node/edge style helpers where appropriate

## M3 - Transaction-safe declarative commits

- [ ] Replace direct committed graph mutation in the declarative interaction path with
      transaction-safe controller/store entry points.
- [ ] Keep ephemeral drag/hover session state local where that improves ergonomics, but route final
      commits through transactions.
- [ ] Add undo/redo coverage for the declarative path once commits stop mutating `Graph` directly.
- [ ] Add at least one gate proving that a declarative drag or marquee commit produces a
      transaction-safe update path.
- [ ] Define the policy for full replace vs diff-based replace in controlled mode.
- [ ] Consider adding `replace_graph_with_diff` or equivalent if full reset semantics are not enough
      for editor-grade controlled integrations.

## M3 - Callback surface split

- [ ] Split or layer callback surfaces so reviewers can clearly distinguish:
  - headless/store commit callbacks,
  - view-state callbacks,
  - UI gesture lifecycle callbacks.
- [ ] Keep compatibility adapters where useful instead of forcing a flag day.
- [ ] Add one small note explaining which callback layer should be used by apps vs internal UI glue.

## M4 - Declarative interaction closure

- [ ] Migrate selection/marquee state machines toward declarative reducers with explicit commit and
      cancel semantics.
- [ ] Keep pointer-capture and cancel behavior as a first-class regression target while doing this.
- [ ] Decide which interaction pieces remain local surface state vs store-backed editor state.
- [ ] Ensure new declarative interaction work does not regress cache discipline.
- [ ] Add at least one parity gate meaningful to real editor usage, not just synthetic paint-only
      counters.

## M4 - Portal and overlay closure

- [ ] Move from portal/bounds experimentation toward a declared editor-grade portal hosting path for
      the visible subset.
- [ ] Clarify how node content subtrees publish measured geometry into derived stores.
- [ ] Clarify how portal-hosted controls emit edits without bypassing the transaction architecture.
- [ ] Move overlay/menu/toolbar policy to the right ecosystem surfaces where that boundary is
      currently blurry.
- [ ] Add at least one gate that exercises portal + overlay anchoring under motion.

## M5 - Compatibility retained convergence

- [ ] Write explicit exit criteria for `compat-retained-canvas`.
- [ ] Decide which retained-only behaviors still block deprecation.
- [ ] Keep the legacy demo as a compatibility harness, not the default teaching surface.
- [ ] Prevent new retained-only surface area from growing without a documented justification.
- [ ] Add a comparison checklist for declarative vs compat-retained behavior on the flows that matter
      most to editor-grade usage.

## Existing evidence and gates to keep alive

- [x] Paint-only cache and invalidation diagnostics under `tools/diag-scripts/node-graph/`.
- [x] Paint-only portal bounds and hover-anchor diagnostics.
- [x] Retained editor conformance tests in `ecosystem/fret-node/src/ui/canvas/widget/tests/`.
- [x] Store/runtime tests in `ecosystem/fret-node/src/runtime/tests.rs`.
- [ ] Add a compact gate matrix to the README once the first transaction-safe declarative milestone
      lands.

## Open questions that must not get lost

- [ ] Exact naming for the split state types.
- [ ] Exact naming for the controller/instance facade.
- [ ] Whether `edit_queue` and `view_queue` remain public long-term or collapse behind the
      controller surface.
- [ ] Whether controlled sync should expose diff-first helpers by default.
- [ ] Which retained-only behaviors still need a deliberate temporary home while declarative parity
      is being built.
