# ADR 0028: Declarative Element Tree and Cross-Frame Element State (GPUI-Style)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Accepted

## Context

Fret targets editor-grade UI (Unity/Unreal-like docking, multi-window, layered overlays, embedded engine viewports).
For authoring ergonomics, we want **GPUI-like responsibilities** plus **ImGui-like freedom**:

- UI code should feel immediate and composable (easy to build complex layouts quickly).
- The system must still support retained capabilities (focus/IME correctness, multi-window overlays, caching, virtualization).
- We must avoid a future rewrite when scaling from demos to a full editor UI.

Two broad implementation families exist:

1) **Retained tree + diff/reconcile** (React-style keyed diff)
2) **Declarative per-frame element tree + externalized state** (GPUI-style “build, layout, paint, drop”)

GPUI demonstrates that the second approach can deliver immediate-like ergonomics while scaling to large apps by:

- rebuilding an element tree each frame (`Render::render()`),
- using stable IDs to locate cross-frame state (`GlobalElementId`),
- storing only the necessary per-element state outside of element objects (`with_element_state`),
- optionally caching view subtrees when dependencies have not changed.

References:

- GPUI element lifecycle and IDs:
  - `repo-ref/zed/crates/gpui/src/element.rs`
  - `repo-ref/zed/crates/gpui/src/window.rs` (`ElementId`, `with_element_state`)
  - `repo-ref/zed/crates/gpui/src/view.rs` (`AnyView::cached`)
- GPUI ownership model inspiration:
  - https://zed.dev/blog/gpui-ownership

Implementation anchors (Fret MVP2 skeleton):

- Element IDs + window-scoped cross-frame state store (`(GlobalElementId, TypeId)`):
  - `crates/fret-ui/src/elements/mod.rs`
- Unkeyed list reorder detection (debug warning; requires explicit keys for dynamic lists):
  - `crates/fret-ui/src/elements/mod.rs` (logs: "unkeyed element list order changed; add explicit keys to preserve state")

## Decision

Adopt a **GPUI-style declarative element model** as the long-term authoring and runtime direction:

### 1) Each frame builds a fresh element tree

- A window’s UI is described by a root `render()` function that constructs an element tree from current app state.
- After layout and paint, the element objects are dropped; the next frame rebuilds the tree.

This provides ImGui-like “write the UI every frame” ergonomics, but the runtime remains retained in behavior
(focus/IME, command routing, docking, overlays).

#### Execution Contract (Important)

For the declarative authoring path, **a window/root must call `render_root(...)` once per frame** *before*
`UiTree::layout_all(...)` / `UiTree::paint_all(...)`.

Rationale:

- Model observation (`observe_model(...)`) and invalidation wiring are derived from the latest render pass.
- The runtime intentionally treats model observation as **per-frame data**; if a frame advances but you do not
  call `render_root(...)`, the next `layout`/`paint` pass may run with missing observation relationships and
  will not automatically “remember” which models should invalidate which elements.

This is consistent with the GPUI-style “build every frame” mental model and keeps the runtime simple and
predictable. If we later want to support “skip render passes” as an optimization, it must come with a
separately specified observation/cache lifecycle.

### 2) Cross-frame element state is externalized and keyed by IDs

Introduce a cross-frame element state store:

- Key: `(GlobalElementId, TypeId)`
- Value: `Box<dyn Any>` state

Elements can access state via a scoped API (conceptually):

- `with_element_state(global_id, |prev: Option<S>| -> (R, S))`

State is preserved across frames if and only if:

- the element has a stable ID, and
- its ancestor ID path (GlobalElementId) is stable within the same window/root scope.

### 3) Element IDs support both convenience and correctness

We support a mixed ID strategy:

- **Default / convenience**: callsite-based IDs (for rapid authoring).
- **Explicit keys**: required for any dynamic list/tree, where insertion/reordering must preserve identity.

The contract must document when explicit keys are required to keep focus/IME/capture stable.

### 4) Multi-root overlays remain first-class

The declarative model must support multiple roots per window (base UI + overlay/popup/modal) as described in ADR 0011.
Each root has its own element ID namespace, but the window-level state store must support addressing elements in all roots.

### 5) Optional caching exists above element state

On top of element state, we may add subtree caching (GPUI-style `AnyView::cached`) where appropriate:

- caching is keyed by a “cache key” of layout-affecting inputs (bounds, content mask, text style, etc.).
- caching is invalidated by explicit notifications from models (app-owned state changes).

Caching is an optimization; correctness relies on IDs + state store, not caching.

## Consequences

- Authoring feels immediate and composable, enabling “complex editor UI” without fighting Rust borrowing.
- Retained behaviors (focus/IME/commands) remain correct because state is keyed by stable identity, not object lifetimes.
- We avoid implementing a heavyweight diff/reconcile engine early.
- We can still support virtualization and caching because the element tree is rebuilt from data each frame.

## Open Questions (To Decide Before Implementation)

### Locked P0 Choices

#### 1) `ElementId` variants (minimal set, future-proof)

`ElementId` supports:

- `Callsite` (implicit default for rapid authoring; derived from `Location::caller()`),
- `Key(u64)` (explicit numeric keys),
- `Name(Arc<str>)` (explicit human-readable keys).

More variants (UUID, composite keys) can be added later, but these three cover:

- stable identity for dynamic trees/lists,
- stable identity for persisted panels and debug tooling,
- “just works” ergonomics for static trees.

#### 2) Keying rule: dynamic collections must use explicit keys

Any element container that renders a dynamic collection (where children count/order can change at runtime)
must assign an explicit key to each child.

Diagnostics policy:

- in debug builds, if a container renders a dynamic list without explicit keys and the child sequence changes
  between frames, emit an inspector-visible warning and include the offending `GlobalElementId` path.

#### 3) State store lifetime: mark/sweep with a small lag

Element state is stored by `(GlobalElementId, TypeId)`.

Each rendered frame:

- marks all `GlobalElementId`s that were actually visited during element execution,
- after present, sweeps state entries that have not been seen for `gc_lag_frames = 2` frames.

Rationale:

- avoids thrash for transient overlay rebuilds,
- still bounds memory for large editors.

Interaction state (focus/capture/hover) is not “lagged”: it is cleared immediately when its target element
is not present in the current frame’s element tree.

#### 4) Interop with the existing `UiTree`: evolve, do not fork

The element execution model should be implemented by evolving the existing UI runtime structures
(`UiTree` layout/hit-test/event routing) instead of introducing a parallel runtime.

Rationale:

- minimizes duplicated semantics (focus, capture, overlays),
- avoids long-lived “two UI runtimes” migrations.

##### Bridge sketch (the missing “reconciliation”)

Even though the element tree is rebuilt every frame, we still need a stable runtime substrate to:

- preserve focus/capture/IME targets,
- preserve per-node layout and hit-test metadata,
- avoid a parallel “second UI runtime” during migration.

The bridge is **identity mapping**, not a React-style structural diff engine:

- During element execution, each element has a stable `GlobalElementId`.
- The UI runtime maintains a map: `GlobalElementId -> NodeId` (where `NodeId` is the retained `UiTree` key).
- When an element is built:
  - if a `NodeId` exists for its `GlobalElementId`, reuse that node,
  - otherwise create a new `UiTree` node and record the mapping.
- Each frame:
  - mark all visited `GlobalElementId`s,
  - after present, sweep nodes that were not visited for `gc_lag_frames = 2` frames (align with state-store GC lag).

This yields:

- “write the UI every frame” ergonomics,
- retained semantics (stable focus/capture targets),
- predictable behavior without a heavyweight reconciliation algorithm.

##### Where input state lives (TextInput example)

For a `<TextInput>`-like element, split state deliberately:

- **Model state (app-owned)**: the text buffer content and domain policies (validation, undo, etc.).
- **Element-local state (cross-frame element state store)**: selection range, caret position, scroll offsets,
  IME composition range, “last click” timing, etc.

The element-local state is stored under `(GlobalElementId, TypeId)` and is what keeps cursor/selection stable
across frames even though element objects are dropped after paint.

##### “Delete second item, third becomes second”: explicit keys

The runtime cannot infer “move vs destroy” from structural position alone.

Therefore:

- dynamic lists/trees must provide explicit keys,
- keyed identity keeps `GlobalElementId` stable under insert/remove/reorder, so both element-local state and `NodeId`
  reuse stay attached to the correct item.

#### 5) Debuggability: inspector exposes element identity and state types

The UI inspector hooks (ADR 0036) must be able to show:

- `GlobalElementId` paths,
- element source locations (when available),
- state types stored under an element (TypeId names in debug builds).
