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

## Goals

- Make "cache-hit" a performance optimization only: it MUST NOT change lifetime semantics.
- Define a deterministic liveness root set for GC under view-cache reuse (no frame-age guessing).
- Make structural detaches explainable and attributable (callsite + owning root + membership).
- Unblock removing the global "skip sweep under reuse" stopgap (workstream MVP2-cache-005).
- Keep future refactors incremental: this contract must continue to hold as more work moves to
  prepaint (ADR 0182) and prepaint-windowed surfaces (ADR 0190).

## Terminology (normative)

- **Element**: a stable identity (`GlobalElementId`) with optional local state keyed by `(id, TypeId)`.
- **Node**: a retained UI tree node (`NodeId`) which may carry an element identity.
- **Ownership root**: the element-runtime root id (`NodeEntry.root`) that "owns" an element/node entry.
- **Liveness roots**: the root set used for reachability under GC.
  - **Layer roots**: installed layer roots for the window (including invisible layers).
  - **View-cache reuse roots**: cache-root elements that are marked as reused for this frame.
  - **Retained keep-alive roots**: runtime-owned keep-alive buckets that must remain live even when
    they are not currently in the visible/active window (e.g. retained virtual-list keep-alive,
    ADR 0192).
- **ViewCache root**: an element/node that defines a reuse boundary and a cache key boundary (ADR 1152).
- **Cache hit**: reusing a ViewCache root without re-running its declarative child render closure.
- **Structural detach**: an explicit parent/child relationship removal (e.g. `set_children` dropping a
  child, removing a layer root). Detach is the only semantic event that should make a subtree eligible
  for collection (modulo lag).
- **Island root**: a subtree root that is unreachable from *all* liveness roots at sweep time.
- **Subtree membership list**: the per-reuse-root element list used to deterministically "touch" a
  retained subtree on cache-hit frames.

## Implementation Mapping (Fret; non-normative)

This section maps the normative terms to the current codebase to keep future refactors grounded.
The contract is the source of truth; the mapping may evolve.

- **Layer roots**: `UiTree::all_layer_roots` (`crates/fret-ui/src/tree/layers.rs`).
- **View-cache reuse roots**: `WindowElementState::{mark_view_cache_reuse_root, view_cache_reuse_roots}` (`crates/fret-ui/src/elements/runtime.rs`).
- **Retained keep-alive roots**: `WindowElementState::retained_virtual_list_keep_alive_roots` (`crates/fret-ui/src/elements/runtime.rs`).
- **Subtree membership list**: `WindowElementState::view_cache_elements_for_root` (`crates/fret-ui/src/elements/runtime.rs`).
- **GC reachability under reuse**: `crates/fret-ui/src/declarative/mount.rs` (see the `reachable_from_view_cache_roots` set and the sweep predicate).
- **Removed-subtree attribution** (diagnostics): `UiDebugRemoveSubtreeRecord` exported from `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`.
  - Note: `removed_subtrees.reachable_from_layer_roots` MUST reflect the same conservative reachability used by GC, not just `UiTree.children`.
    Cache-hit frames can temporarily have incomplete `UiTree` child edges, while the last mounted `WindowFrame.children` still retains authoritative
    element-tree edges. For GC and diagnostics, reachability should be computed from the liveness roots using the union of both sources.
  - Note: the field name is historical. In current implementations it represents reachability from the
    window's liveness roots (layer roots + retained keep-alive roots), not strictly from layer roots.
- **Ownership root**: `NodeEntry.root` (`crates/fret-ui/src/elements/runtime.rs`); cross-root overwrites must not be “repaired” implicitly (ADR 0191).

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
  - Concretely, the per-frame element-state map is driven by an explicit "accessed set": `with_element_state` records keys, and `reuse_prepaint`/`reuse_paint` replay extends `next_frame.accessed_element_states` by copying the recorded key slice from the previous frame's range. A cache hit therefore cannot accidentally drop state simply because a subtree did not rebuild.

### Upstream anchors (non-normative; line numbers may drift)

- **Zed / GPUI**
  - `crates/gpui/src/window.rs`: `WindowInvalidator{ dirty_views }`, `WindowInvalidator::invalidate_view`, `Window::mark_view_dirty` (marks ancestor views dirty).
    - Source: https://github.com/zed-industries/zed/blob/main/crates/gpui/src/window.rs
  - `crates/gpui/src/view.rs`: view caching and reuse (`reuse_prepaint`, `reuse_paint`) + dependency tracking (`accessed_entities`).
    - Source: https://github.com/zed-industries/zed/blob/main/crates/gpui/src/view.rs
- **Flutter**
  - `packages/flutter/lib/src/widgets/framework.dart`: `_InactiveElements`, `BuildOwner.finalizeTree`, `BuildOwner.deactivateChild`.
  - `packages/flutter/lib/src/widgets/overlay.dart`: `OverlayEntry`/`OverlayState` (lifetime is explicit; invisibility/offstage is not disposal).
    - Sources:
      - https://github.com/flutter/flutter/blob/master/packages/flutter/lib/src/widgets/framework.dart
      - https://github.com/flutter/flutter/blob/master/packages/flutter/lib/src/widgets/overlay.dart

- **Pinned references in this repo** (preferred for audits; URLs/line numbers drift):
  - `repo-ref/zed/crates/gpui/src/{window.rs,view.rs}`
  - `repo-ref/flutter/packages/flutter/lib/src/{widgets/framework.dart,rendering/sliver_multi_box_adaptor.dart,widgets/overlay.dart}`

In all cases, "not rebuilt this frame" is not a signal for disposal. Liveness comes from explicit roots (composition/window roots) and ownership bookkeeping.

### Failure mode we are preventing (workstream MVP2-cache-005)

The overlay torture regression (when the global stopgap is disabled) demonstrates a common pitfall in cached-subtree systems:

- A subtree that the app still expects to be interactive (e.g. a semantics target) becomes an *island root*: it is no longer reachable from any installed layer root, and also no longer reachable from any ViewCache reuse root at sweep time.
- Once it is an island, a reachability-based GC is allowed to collect it, and scripted clicks (or real pointer input) start failing as if the UI "randomly disappeared".

In other words, the GC is correctly acting on the reachable graph it sees; the bug is that the ownership/attachment bookkeeping allowed a still-needed subtree to become structurally detached.

In practice, the most common way for this to happen in a cache-hit system is **incomplete liveness bookkeeping under reuse**:

- a cache root swaps or re-parents its child subtree (e.g. a content slot changes pages),
- but the runtime fails to record a complete "subtree element list" for the new retained subtree,
- so cache-hit frames do not "touch" some still-live elements (their `last_seen_frame` is not refreshed),
- and once the lag window expires, the GC is allowed to sweep an otherwise interactive subtree.

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
3. **All retained keep-alive roots** (e.g. retained VirtualList keep-alive buckets; ADR 0192).
4. Optional: additional explicitly pinned roots (future: long-lived background roots, debugging tools).

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

### 7) ViewCache subtree element lists are authoritative for liveness bookkeeping

When a ViewCache root is reused, the runtime still needs an explicit, deterministic way to refresh liveness for the retained subtree without re-running declarative render.

Contract:

- For every ViewCache root (a `GlobalElementId` that may become a reuse root), the runtime MUST maintain a complete list of `GlobalElementId`s that belong to the retained subtree under that root.
- The list MUST be refreshed on frames where the subtree is structurally rebuilt (cache-miss frames), and MUST be "touched" (last-seen updated) on cache-hit frames.
- The list MUST NOT depend on "this frame's instances" being present: cache hits can legitimately skip instance creation for large portions of the subtree.
- The list MUST be derivable from authoritative retained data:
  - preferred: walk the retained node graph and recover element identity via the element runtime's node entries,
  - allowed fallbacks: `WindowFrame.instances` and `UiTree::node_element(node)` when present.

Implication:

- If an interactive subtree becomes an island while reuse roots exist, and the subtree contains elements that are missing from the corresponding ViewCache subtree list, that is a bookkeeping bug and MUST be treated as a correctness regression until proven otherwise.

## Alignment notes (how this maps to GPUI / Flutter)

This section is non-normative, but it explains why the chosen contract is the "least surprising"
and best-aligned choice for long-term extensibility.

### GPUI (Zed) view caching

- **The view graph is the liveness root set.** A cached view remains part of the window's view graph,
  so it is still "owned" even when its output is reused.
- **Cache hit still "touches" what matters.** On a cache hit, GPUI consults per-view element state
  (`Window::with_element_state`) and reuses `prepaint`/`paint` ranges (`reuse_prepaint`/`reuse_paint`)
  while also extending dependency tracking (`accessed_entities`) and carrying forward `accessed_element_states`
  for the reused range. The reuse gate is driven by
  `dirty_views` (and ancestor propagation via `mark_view_dirty`), not by incidental "was this view
  visited this frame?" heuristics.
- **Why Fret needs an explicit GC contract**: GPUI does not maintain a retained declarative node tree
  that can be swept independently of the view graph; the reused output is range-based. In Fret, we
  intentionally skip mounting/execution on cache-hit frames *while still retaining node identity and
  element runtime bookkeeping*, which makes a reachability-based "liveness roots" contract necessary.
- **Implication for Fret**: our `view_cache_subtree_element_lists` + explicit liveness roots are the
  local equivalent of "touch the right dependencies on cache-hit frames". The membership list must
  be complete (including nested cache roots) and deterministically refreshable on cache-hit frames.
- Evidence anchors:
  - `repo-ref/zed/crates/gpui/src/window.rs` (`WindowFrame.element_states`, `WindowFrame.accessed_element_states`, `Window::with_element_state`, `Window::reuse_prepaint`, `Window::reuse_paint`)
  - `repo-ref/zed/crates/gpui/src/view.rs` (`AnyView::cached`, `dirty_views` reuse gate, `reuse_prepaint`/`reuse_paint`)

### Flutter lifecycle / ownership

- **Rebuild does not define lifetime.** Widgets rebuild freely; elements/render-objects live until
  they are structurally removed.
- **Inactive limbo exists.** Flutter explicitly models a subtree that is detached but not yet
  disposed (`_InactiveElements`), then finalizes at the end of the frame (`BuildOwner.finalizeTree`).
- **Visibility does not define lifetime.** A subtree can be offstage/invisible and still owned.
- **Virtualization keeps explicit ownership buckets.** Sliver lists can keep offscreen children alive
  via a dedicated keep-alive bucket (`RenderSliverMultiBoxAdaptor`’s `_keepAliveBucket`), which makes
  “not currently built/laid out” orthogonal to lifetime.
- **Implication for Fret**: our GC-lag window is analogous to "inactive limbo", but it must be
  driven by explicit detach + reachability, not by "not visited due to caching".
- Evidence anchors:
  - `repo-ref/flutter/packages/flutter/lib/src/widgets/framework.dart` (`class _InactiveElements`, `BuildOwner.finalizeTree`)
  - `repo-ref/flutter/packages/flutter/lib/src/rendering/sliver_multi_box_adaptor.dart` (`_keepAliveBucket`)

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

Diagnostics SHOULD prefer capturing debug paths at the time of removal/attribution (when the identity
is still available), rather than trying to resolve paths after the fact from a pruned identity map.

## Additional invariants (nested cache roots + ownership)

This section is normative. These invariants exist to prevent the "randomly disappears" class of bugs
in cached-subtree systems, and to keep future work (ADR 0190 / ADR 0182) incremental.

### A) Subtree membership lists must be complete under nesting

Contract:

- For a ViewCache root `R`, its recorded subtree membership list MUST include *all* elements in the
  retained subtree, including nested ViewCache roots and their descendants.
- On cache-hit frames for `R`, liveness refresh MUST NOT depend on re-running declarative render for
  nested cache roots. The outer root `R` is responsible for keeping the entire retained subtree alive.
- If child-edge reachability drifts (e.g. missing/partial retained edges for a frame), the membership
  list MUST still prevent premature collection. Such drift is a correctness bug, but should not turn
  into user-visible "missing semantics" failures.

Implementation note:

- Fret already records ViewCache state key accesses to *all* cache roots on the active stack, so
  touching the outer root's recorded state keys is sufficient to keep nested-root state alive.
  Subtree membership lists serve the analogous role for element/node liveness bookkeeping.

### B) Ownership is explicit; cross-root reparenting is not implicit

Contract:

- Touching an existing retained subtree MUST NOT implicitly reassign ownership across roots.
  If an element is already owned by a different root, that mismatch MUST be surfaced via diagnostics;
  it MUST NOT be "repaired" silently.
- If the framework ever needs Flutter `GlobalKey`-like reparenting/teleportation across roots/layers,
  it MUST be an explicit operation with a dedicated contract (ownership transfer event + diagnostics
  attribution). This ADR intentionally does not define that operation.

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
3. Re-run the overlay torture and sidebar refresh harnesses with sweep enabled under reuse (stopgap removed).

Success criteria:

- `tools/diag-scripts/ui-gallery-overlay-torture.json` stays green under `cache+shell` with sweep enabled under reuse.
- `tools/diag-scripts/ui-gallery-sidebar-scroll-refresh.json` stays green under `cache+shell` with sweep enabled under reuse.
- Failing bundles remain explainable (diagnostics fields are present and actionable).

## References

- ADR 0028: declarative element GC + identity mapping: `docs/adr/0028-declarative-elements-and-element-state.md`
- ADR 0011: overlays and multi-root: `docs/adr/0011-overlays-and-multi-root.md`
- ADR 0067: overlay policy boundary: `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
- ADR 1151: identity debug paths + staged state: `docs/adr/1151-element-identity-debug-paths-and-frame-staged-element-state.md`
- ADR 1152: ViewCache subtree reuse + state retention: `docs/adr/1152-view-cache-subtree-reuse-and-state-retention.md`
- Workstream tracker: `docs/workstreams/gpui-parity-refactor-todo.md` (MVP2-cache-005)
- Flutter lifecycle reference: `packages/flutter/lib/src/widgets/framework.dart` (Flutter upstream).
  - Anchors (pinned `repo-ref/flutter`, may drift upstream):
    - `_InactiveElements` (~`framework.dart:2099`)
    - `BuildOwner.finalizeTree` (~`framework.dart:3339`)
    - `Element.deactivateChild` (~`framework.dart:4632`)
- Flutter keep-alive bucket reference: `packages/flutter/lib/src/rendering/sliver_multi_box_adaptor.dart`.
  - Anchors (pinned `repo-ref/flutter`, may drift upstream):
    - `RenderSliverMultiBoxAdaptor._keepAliveBucket` (~`sliver_multi_box_adaptor.dart:233`)
- GPUI view caching reference: `crates/gpui/src/view.rs`, `crates/gpui/src/window.rs` (Zed upstream).
  - Anchors (pinned `repo-ref/zed`, may drift upstream):
    - `ViewCacheKey` (~`view.rs:22`)
    - `AnyView::cached` (~`view.rs:103`)
    - `reuse_prepaint` (~`view.rs:216`)
    - `detect_accessed_entities` (~`view.rs:227`)
    - `reuse_paint` (~`view.rs:280`)
    - `WindowInvalidatorInner.dirty_views` (~`window.rs:105`)
    - `Window::mark_view_dirty` (~`window.rs:1466`)
    - `Window::use_keyed_state` / `with_global_id` (~`window.rs:2839`)
    - `Window::with_element_state` (~`window.rs:2840`)
