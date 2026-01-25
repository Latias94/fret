Status: Accepted

# ADR 0191: Declarative Liveness Roots and GC Under ViewCache Reuse

## Context

Fret’s declarative element tree is rebuilt per-frame, while the `UiTree` is retained (ADR 0005, ADR 0028).
Element-local state is keyed by stable identity (`(GlobalElementId, TypeId)`) and is retained across frames with a small lag window (ADR 0028, ADR 1151).

The ViewCache experiment allows clean cache roots to skip re-running the declarative child render closure while reusing the mounted subtree (ADR 1152) and reusing prepaint/paint ranges (ADR 0055, ADR 0182).

However, the declarative GC for nodes/elements historically used a time-based heuristic (“visited in the last `gc_lag_frames`”), which is incompatible with cache-hit frames that intentionally skip mounting/execution. This produced a stopgap: “skip sweeping while any reuse roots exist” (workstream MVP2-cache-005), which prevents correctness bugs but also prevents collecting genuinely detached nodes.

This ADR locks the contract needed to:

- remove the global stopgap safely,
- keep cache-hit frames correct under multi-root overlays (ADR 0011, ADR 0067),
- and keep the system extensible toward prepaint-driven ephemeral items (ADR 0190).

## Non-normative reference patterns (best practice survey)

These systems converge on the same core idea: **GC must be reachability/ownership driven, not frame-age driven**.

- **Flutter**: retained `Element`/`RenderObject` trees; disposal happens when a subtree is structurally removed during reconciliation, not because it was “not visited this frame”. `BuildOwner.finalizeTree` completes lifecycle transitions for removed elements; “inactive” elements remain owned until finalized.
- **React**: Fiber nodes are created/retained based on reconciliation; nodes are deleted when they are not present in the next tree (structural removal), not because they were skipped by an optimization.
- **Jetpack Compose / SwiftUI**: compositions are retained via slot tables/graph ownership; nodes are disposed when removed from composition, not due to timeouts.
- **GPUI-style view caching**: cache hits reuse recorded frame ranges and keep view dependencies/state live because the view is still present in the window’s view graph; cache hits do not imply liveness loss.

In all cases, “not rebuilt this frame” is not a signal for disposal. Liveness comes from explicit roots (composition/window roots) and ownership bookkeeping.

## Decision

### 1) Definitions

- **Element runtime root**: a per-window root scope derived from `(window, root_name)`, e.g. `global_root(window, root_name)` in `crates/fret-ui/src/elements/*`.
- **Layer root**: a `NodeId` that is installed as a `UiLayer` root in the `UiTree` (ADR 0011).
- **ViewCache reuse root**: a `GlobalElementId` for which the runtime decides “reuse mounted subtree” for this frame (ADR 1152).
- **Liveness root set**: the set of roots from which reachability is computed for GC.

### 2) Liveness roots are explicit, stable, and independent of paint visibility

When view-cache is enabled, the liveness root set MUST include:

1. **All installed layer roots** for the window (base root + overlay/popup/modal roots).
2. **All ViewCache reuse roots**, mapped to their current `NodeId` via the element runtime’s identity map.
3. Optional: additional explicitly pinned roots (future: long-lived background roots, debugging tools).

Critically:

- A layer’s `visible=false` MUST NOT remove it from the liveness root set. Visibility is a paint concern, not a lifetime concern.
- Roots are removed from liveness only when the layer is uninstalled (e.g. `remove_layer`) or the cache root is no longer marked as reused.

### 3) Ownership is stable; “touch” must not reassign root ownership

Each element’s node entry has an owning element runtime root (conceptually `owner_root: ElementRootId`).

Rules:

- The owner root is established when the element is first mounted under a given element runtime root.
- “Touch existing subtree” paths (used to refresh liveness/diagnostics on cache-hit frames) MUST update `last_seen_frame` and diagnostics, but MUST NOT overwrite the owning root if it is already set to a different root.
- If an element identity appears under a different element runtime root than its owner, that is a correctness bug (cross-root identity collision or bookkeeping corruption). Debug builds/diagnostics MUST surface it (see Diagnostics section).

Rationale:

- Multi-root systems (overlays/portals) require strong separation between “where it is painted” (layer root) and “who owns its identity/state” (element runtime root).
- Overwriting ownership during cross-root walks can reclassify nodes for GC and cause unrelated roots to sweep live subtrees (the “island” failure mode).

### 4) GC must be reachability based (with a lag window), using a union of authoritative edge sources

When sweeping nodes/elements for a given element runtime root:

1. A node/element is eligible for collection only if:
   - it is older than the lag cutoff (e.g. `last_seen_frame < frame_id - gc_lag_frames`), AND
   - it is unreachable from the liveness root set using the reachability algorithm below.
2. Reachability MUST walk children edges using the union of authoritative sources:
   - `UiTree` retained child edges, AND
   - `WindowFrame` child edges (the retained declarative snapshot used for mount-time reuse).

This makes “is the subtree still attached?” a well-defined question even when one source is temporarily stale.

### 5) Cache-hit frames must preserve liveness bookkeeping deterministically

On frames where a ViewCache root is reused:

- The runtime MUST be able to map the reuse root to a `NodeId` (identity mapping is stable).
- The runtime MUST keep the subtree’s liveness bookkeeping consistent without re-running render:
  - state-key touches (ADR 1152),
  - model/global observation inheritance (ADR 0180),
  - per-frame registries required for correctness (e.g. scroll-handle bindings).

This ADR does not require that all registries be fully generalized today, but it establishes the invariant: cache hits must not silently drop liveness/ownership.

## Diagnostics and explainability (hard requirement)

When diagnostics are enabled and a subtree is removed during GC, a single `bundle.json` MUST contain enough information to answer:

1. **Why was it collected?**
   - `reachable_from_layer_roots` and `reachable_from_view_cache_roots` (or a single combined `reachable_from_liveness_roots`),
   - the lag cutoff decision (`last_seen_frame`, `cutoff`).
2. **Which pass removed it?**
   - the triggering element runtime root (`trigger_element_root` + best-effort debug path),
   - and the removed subtree’s owning root (if known).
3. **Did ownership bookkeeping drift?**
   - a record of `NodeEntry` root overwrites (element + old_root + new_root + debug paths).

## Consequences / Tradeoffs

- GC becomes slightly more expensive in debug/diagnostics modes because reachability and explainability are required.
- The ownership invariants constrain “reusing the same `GlobalElementId` across roots” (which is a bug in this model).
- This ADR is intentionally strict to avoid “randomly disappears / stale semantics” classes of bugs, which are extremely costly in cached-subtree systems.

## Rollout plan (workstream gate)

1. Implement diagnostics for liveness roots and ownership overwrites.
2. Make “touch existing subtree” update `last_seen_frame` without reassigning ownership.
3. Re-run the overlay torture and sidebar refresh harnesses with the stopgap disabled.
4. Remove the global “skip sweep while reuse exists” stopgap.

Success criteria:

- `tools/diag-scripts/ui-gallery-overlay-torture.json` stays green under `cache+shell` with stopgap disabled.
- `tools/diag-scripts/ui-gallery-sidebar-scroll-refresh.json` stays green under `cache+shell` with stopgap disabled.
- Failing bundles remain explainable (diagnostics fields are present and actionable).

## References

- ADR 0028: declarative element GC + identity mapping: `docs/adr/0028-declarative-elements-and-element-state.md`
- ADR 0011: overlays and multi-root: `docs/adr/0011-overlays-and-multi-root.md`
- ADR 0067: overlay policy boundary: `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
- ADR 1151: identity debug paths + staged state: `docs/adr/1151-element-identity-debug-paths-and-frame-staged-element-state.md`
- ADR 1152: ViewCache subtree reuse + state retention: `docs/adr/1152-view-cache-subtree-reuse-and-state-retention.md`
- Workstream tracker: `docs/workstreams/gpui-parity-refactor-todo.md` (MVP2-cache-005)

