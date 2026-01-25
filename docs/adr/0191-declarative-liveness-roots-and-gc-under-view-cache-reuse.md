Status: Accepted

# ADR 0191: Declarative Liveness Roots and GC Under ViewCache Reuse

## Context

Fret's declarative element tree is rebuilt per-frame, while the `UiTree` is retained (ADR 0005, ADR 0028).
Element-local state is keyed by stable identity (`(GlobalElementId, TypeId)`) and is retained across frames with a small lag window (ADR 0028, ADR 1151).

The ViewCache experiment allows clean cache roots to skip re-running the declarative child render closure while reusing the mounted subtree (ADR 1152) and reusing prepaint/paint ranges (ADR 0055, ADR 0182).

However, the declarative GC for nodes/elements historically used a time-based heuristic ("visited in the last `gc_lag_frames`"), which is incompatible with cache-hit frames that intentionally skip mounting/execution. This produced a stopgap: "skip sweeping while any reuse roots exist" (workstream MVP2-cache-005), which prevents correctness bugs but also prevents collecting genuinely detached nodes.

This ADR locks the contract needed to:

- remove the global stopgap safely,
- keep cache-hit frames correct under multi-root overlays (ADR 0011, ADR 0067),
- and keep the system extensible toward prepaint-driven ephemeral items (ADR 0190).

## Non-normative reference patterns (best practice survey)

These systems converge on the same core idea: **GC must be reachability/ownership driven, not frame-age driven**.

- **Flutter**: retained `Element`/`RenderObject` trees; disposal happens when a subtree is structurally removed during reconciliation, not because it was "not visited this frame".
  - `Element.deactivateChild` detaches a child from its parent and moves it to `_InactiveElements` (still owned, not yet disposed).
  - `BuildOwner.finalizeTree` unmounts `_InactiveElements` at the end of a frame/build pass (lifecycle becomes `defunct`), and performs sanity checks (e.g. duplicate `GlobalKey` detection).
  - Visibility is not lifetime: an `OverlayEntry` or an `Offstage` subtree can be non-visible while remaining part of the owned tree.
  - "KeepAlive" in sliver/list contexts is a separate mechanism: it prevents render-object disposal even when off-screen; it does not redefine the meaning of "structurally removed".
- **React**: Fiber nodes are created/retained based on reconciliation; nodes are deleted when they are not present in the next tree (structural removal), not because they were skipped by an optimization.
- **Jetpack Compose / SwiftUI**: compositions are retained via slot tables/graph ownership; nodes are disposed when removed from composition, not due to timeouts.
- **GPUI-style view caching**: cache hits reuse recorded frame ranges and keep view dependencies/state live because the view is still present in the window's view graph.
  - View caching is gated by "dirty views" (e.g. `WindowInvalidator.dirty_views`) plus a cache key (bounds/content mask/text style).
  - Cache hits still restore dependency tracking (e.g. extend `accessed_entities`) and preserve element-local state by "accessing" it as part of `prepaint`/`paint` replay.
  - Concretely, the per-frame element-state map is driven by an explicit "accessed set": reuse paths extend the next frame's accessed element-state keys from the recorded range, so a cache hit cannot accidentally drop state simply because a subtree did not rebuild.

In all cases, "not rebuilt this frame" is not a signal for disposal. Liveness comes from explicit roots (composition/window roots) and ownership bookkeeping.

### Failure mode we are preventing (workstream MVP2-cache-005)

The overlay torture regression (when the global stopgap is disabled) demonstrates a common pitfall in cached-subtree systems:

- A subtree that the app still expects to be interactive (e.g. a semantics target) becomes an *island root*: it is no longer reachable from any installed layer root, and also no longer reachable from any ViewCache reuse root at sweep time.
- Once it is an island, a reachability-based GC is allowed to collect it, and scripted clicks (or real pointer input) start failing as if the UI "randomly disappeared".

In other words, the GC is correctly acting on the reachable graph it sees; the bug is that the ownership/attachment bookkeeping allowed a still-needed subtree to become structurally detached.

The best-practice implication (seen in Flutter/React/Compose/GPUI) is that optimizations must not create implicit structural detaches, and root membership must be explicit and diagnosable.

### Derived best practices (applicable to Fret)

1. **Separate lifetime from visibility.** A layer can be `visible=false` while still being live; lifetime changes only when a root is uninstalled.
2. **Make liveness a property of the ownership graph + root set**, not of incidental "visited this frame" behavior. Optimizations (like cached-subtree reuse) must not change lifetime semantics.
3. **Make cache hits explicitly "touch" dependencies and state**, the same way a full rebuild would. In GPUI this happens by replaying prepaint/paint and restoring accessed dependencies; in Flutter it happens by staying in the active tree.
4. **Keep multi-root ownership stable and diagnosable.** Cross-root identity reuse is either forbidden (bug) or must be modeled explicitly; "touch" paths must not silently reassign ownership.
5. **Treat structural detaches as explicit lifecycle events.** Detach should be attributable (callsite/root), and there should be a clear policy for "inactive limbo" vs immediate disposal (Flutter's `_InactiveElements` is a proven pattern).

## Decision

### 1) Definitions

- **Element runtime root**: a per-window root scope derived from `(window, root_name)`, e.g. `global_root(window, root_name)` in `crates/fret-ui/src/elements/*`.
- **Layer root**: a `NodeId` that is installed as a `UiLayer` root in the `UiTree` (ADR 0011).
- **ViewCache reuse root**: a `GlobalElementId` for which the runtime decides "reuse mounted subtree" for this frame (ADR 1152).
- **Liveness root set**: the set of roots from which reachability is computed for GC.

### 2) Liveness roots are explicit, stable, and independent of paint visibility

When view-cache is enabled, the liveness root set MUST include:

1. **All installed layer roots** for the window (base root + overlay/popup/modal roots).
2. **All ViewCache reuse roots**, mapped to their current `NodeId` via the element runtime's identity map.
3. Optional: additional explicitly pinned roots (future: long-lived background roots, debugging tools).

Critically:

- A layer's `visible=false` MUST NOT remove it from the liveness root set. Visibility is a paint concern, not a lifetime concern.
- Roots are removed from liveness only when the layer is uninstalled (e.g. `remove_layer`) or the cache root is no longer marked as reused.

### 3) Ownership is stable; "touch" must not reassign root ownership

Each element's node entry has an owning element runtime root (conceptually `owner_root: ElementRootId`).

Rules:

- The owner root is established when the element is first mounted under a given element runtime root.
- "Touch existing subtree" paths (used to refresh liveness/diagnostics on cache-hit frames) MUST update `last_seen_frame` and diagnostics, but MUST NOT overwrite the owning root if it is already set to a different root.
- If an element identity appears under a different element runtime root than its owner, that is a correctness bug (cross-root identity collision or bookkeeping corruption). Debug builds/diagnostics MUST surface it (see Diagnostics section).

Rationale:

- Multi-root systems (overlays/portals) require strong separation between "where it is painted" (layer root) and "who owns its identity/state" (element runtime root).
- Overwriting ownership during cross-root walks can reclassify nodes for GC and cause unrelated roots to sweep live subtrees (the "island" failure mode).

### 4) GC must be reachability based (with a lag window), using a union of authoritative edge sources

When sweeping nodes/elements for a given element runtime root:

1. A node/element is eligible for collection only if:
   - it is older than the lag cutoff (e.g. `last_seen_frame < frame_id - gc_lag_frames`), AND
   - it is unreachable from the liveness root set using the reachability algorithm below.
2. Reachability MUST walk children edges using the union of authoritative sources:
   - `UiTree` retained child edges, AND
   - `WindowFrame` child edges (the retained declarative snapshot used for mount-time reuse).

This makes "is the subtree still attached?" a well-defined question even when one source is temporarily stale.

### 5) Cache-hit frames must preserve liveness bookkeeping deterministically

On frames where a ViewCache root is reused:

- The runtime MUST be able to map the reuse root to a `NodeId` (identity mapping is stable).
- The runtime MUST keep the subtree's liveness bookkeeping consistent without re-running render:
  - state-key touches (ADR 1152),
  - model/global observation inheritance (ADR 0180),
  - per-frame registries required for correctness (e.g. scroll-handle bindings).

This ADR does not require that all registries be fully generalized today, but it establishes the invariant: cache hits must not silently drop liveness/ownership.

### 6) Structural detachment is explicit; cache-hit must not mutate structure

When a subtree is still live (reachable from the liveness root set), it MUST NOT become detached solely because the runtime skipped executing a render/mount closure.

This implies the following invariants:

- **"Empty children" is never authoritative on cache-hit.** If a ViewCache root is reused (`reuse_view_cache=true`), the runtime MUST NOT interpret an empty declarative child list as a structural removal signal.
- **Only "rebuild" may call `set_children` for a node whose full child list is not known.** If we do not execute a subtree's render closure, we do not know its full child list for the current frame, so calling `set_children(..)` with an incomplete list is forbidden.
- **Cache-hit must preserve attachment.** On reuse frames, the authoritative child-edge graph for the reused subtree MUST remain intact. It can be represented by retained `UiTree` edges, retained `WindowFrame` edges, or both; but the union must remain reachable from the liveness roots.
- **Structural removal is a separate semantic event.** Detaching a child from a parent (e.g. `set_children` dropping a child, or explicitly removing a layer root) is equivalent to Flutter's `deactivateChild`: it is an explicit structural change and is the only time GC eligibility should meaningfully change (modulo lag).
- **Structural detaches under reuse must be explainable.** If a subtree becomes an island while any reuse roots exist, the runtime MUST provide enough diagnostics to attribute the detach (parent + callsite + frame id), and this should be treated as a correctness bug until proven otherwise.

Rationale:

- GPUI cache hits reuse recorded ranges without modifying the view graph; the view remains reachable from window roots and cannot "randomly disappear".
- Flutter separates "temporarily inactive" from "disposed" via `_InactiveElements`; a subtree is not unmounted because it was not rebuilt.

## Diagnostics and explainability (hard requirement)

When diagnostics are enabled and a subtree is removed during GC, a single `bundle.json` MUST contain enough information to answer:

1. **Why was it collected?**
   - `reachable_from_layer_roots` and `reachable_from_view_cache_roots` (or a single combined `reachable_from_liveness_roots`),
   - the lag cutoff decision (`last_seen_frame`, `cutoff`).
2. **Which pass removed it?**
   - the triggering element runtime root (`trigger_element_root` + best-effort debug path),
   - and the removed subtree's owning root (if known).
3. **Did ownership bookkeeping drift?**
   - a record of `NodeEntry` root overwrites (element + old_root + new_root + debug paths).
4. **Where did it become detached?**
   - best-effort attribution of the detach callsite (e.g. a parent `set_children` write) and the severed parent node's element/path, when available.
5. **Was this a structural child swap?**
   - for cache roots, export a small sample of `set_children` "before/after" child element ids (and best-effort debug paths) so we can detect accidental subtree replacement that explains the disappearance.

## Alternatives considered

1. **Keep the global stopgap forever** ("skip sweeping while reuse exists").
   - Pro: simplest and safest short-term behavior.
   - Con: makes GC ineffective under reuse (detached nodes pile up), hides real structural-detach bugs, and blocks future features that rely on explicit lifetime semantics.
2. **Continue using frame-age only** (`last_seen_frame` heuristic), without explicit liveness roots.
   - Pro: cheap and simple.
   - Con: fundamentally incompatible with cache-hit frames and produces "randomly disappears" failures.
3. **Force mount/render closures to run even on cache hits** (never skip declarative rebuild).
   - Pro: trivializes liveness bookkeeping.
   - Con: defeats the entire purpose of ViewCache and prevents reaching GPUI-like steady-state performance.
4. **Adopt a fully retained Element tree** (Flutter-style) for all declarative structure.
   - Pro: clean lifetime semantics; no "not visited" ambiguity.
   - Con: a large architectural shift (higher short-term risk), and unnecessary given the workstream's perf-first incremental approach.
5. **Root-set reachability with stable ownership (chosen)**.
   - Pro: aligns with best practice across Flutter/GPUI/React/Compose; makes caching an optimization that does not alter lifetime semantics; avoids a full rewrite.
   - Con: requires stricter bookkeeping invariants and slightly more debug/diagnostic cost.

## Future extensions

This ADR intentionally keeps the contract strict today, but it is designed to enable future work without another large refactor:

- **Explicit cross-root reparenting ("teleport")**: if we ever want Flutter `GlobalKey`-like moves across roots/layers, it must be an explicit operation with well-defined ownership transfer; implicit "touch" must never cause it.
- **Pinned liveness handles**: a small API to pin a subtree for N frames (or until a token is dropped) can model ephemeral work (tooltips, drag indicators) without forcing rerender.
- **Prepaint-driven ephemeral items** (ADR 0190/ADR 0182): the liveness root set must remain authoritative even as more output shifts from declarative render into prepaint/paint phases.

## Consequences / Tradeoffs

- GC becomes slightly more expensive in debug/diagnostics modes because reachability and explainability are required.
- The ownership invariants constrain "reusing the same `GlobalElementId` across roots" (which is a bug in this model).
- This ADR is intentionally strict to avoid "randomly disappears / stale semantics" classes of bugs, which are extremely costly in cached-subtree systems.

## Rollout plan (workstream gate)

1. Implement diagnostics for liveness roots and ownership overwrites.
2. Make "touch existing subtree" update `last_seen_frame` without reassigning ownership.
3. Re-run the overlay torture and sidebar refresh harnesses with the stopgap disabled.
4. Remove the global "skip sweep while reuse exists" stopgap.

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
- Flutter lifecycle reference (optional pinned checkout): `repo-ref/flutter/packages/flutter/lib/src/widgets/framework.dart`
  - Anchors (as of the pinned checkout used during this workstream): `_InactiveElements` (~2099), `BuildOwner.finalizeTree` (~3339), `Element.deactivateChild` (~4632).
- GPUI view caching reference (optional pinned checkout): `repo-ref/zed/crates/gpui/src/view.rs`, `repo-ref/zed/crates/gpui/src/window.rs`
  - Anchors: `AnyView::cached` (~103), `reuse_prepaint` (~216), `reuse_paint` (~280), `WindowInvalidator.dirty_views` (~105), `Window::mark_view_dirty` (~1476).
