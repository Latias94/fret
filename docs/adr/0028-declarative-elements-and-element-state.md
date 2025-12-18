# ADR 0028: Declarative Element Tree and Cross-Frame Element State (GPUI-Style)

Status: Proposed

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

## Decision

Adopt a **GPUI-style declarative element model** as the long-term authoring and runtime direction:

### 1) Each frame builds a fresh element tree

- A window’s UI is described by a root `render()` function that constructs an element tree from current app state.
- After layout and paint, the element objects are dropped; the next frame rebuilds the tree.

This provides ImGui-like “write the UI every frame” ergonomics, but the runtime remains retained in behavior
(focus/IME, command routing, docking, overlays).

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

1) **ElementId shape**:
   - which variants are supported (string, integer, UUID, callsite, named child, focus handle)?
2) **Keying rules**:
   - what is the exact rule for “lists must use explicit keys”, and how do we enforce/diagnose violations?
3) **State store lifetime and GC**:
   - how do we prune element state that is no longer referenced by the current frame?
4) **Interop with existing `UiTree`**:
   - do we evolve `fret-ui::UiTree` into the element execution engine, or introduce a parallel `ElementRuntime`?
5) **Debuggability**:
   - do we add inspector hooks that expose GlobalElementId paths and state types?

