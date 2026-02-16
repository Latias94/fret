# ADR 0039: Component Authoring Model (Render/RenderOnce + IntoElement, GPUI-Inspired)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Accepted

## Context

Fret’s UI execution model is trending toward a GPUI-style “build element tree every frame” approach (ADR 0028),
but that alone does not define the *authoring* experience.

To build a Unity/Unreal-class editor UI, we need:

- high-level composability (build complex widgets/panels quickly),
- an escape hatch for low-level, performance-sensitive widgets (custom layout/paint),
- a clean split between **stateful views** (bound to app-owned models) and **stateless components** (pure props),
- a stable foundation for a `fret-components` ecosystem (ADR 0037).

GPUI’s approach is a useful reference because it separates:

- **Elements**: low-level layout/paint workhorses,
- **Views**: stateful entities that render into elements,
- **Components**: stateless “recipes” that render into elements once.

References:

- GPUI `Render` / `RenderOnce` / `IntoElement`:
  - `repo-ref/zed/crates/gpui/src/element.rs` (search `trait Render`, `trait RenderOnce`, `derive(IntoElement)`)

## Decision

### 1) Provide three authoring layers

Fret defines three conceptual layers (exact names may differ, but the split is stable):

1. **Element layer (low-level)**:
   - custom layout/paint/hit-test primitives,
   - used to implement containers, virtualization, and rendering-heavy widgets.
2. **View layer (stateful)**:
   - “screen pieces” bound to app-owned models (ADR 0031),
   - renders each frame into an element tree (ADR 0028).
3. **Component layer (stateless)**:
   - pure “props → element tree” recipes,
   - convenient for composing reusable UI patterns without owning state.

This preserves “ImGui-like freedom” while still enabling a retained-capability runtime.

### 2) `Render` for stateful views

Stateful UI units implement a `Render`-like trait:

- signature takes `&mut self` and a `Context<Self>`-like handle, and returns something convertible to an element tree.
- view state is primarily app-owned models (ADR 0031) plus limited per-element cross-frame state (ADR 0028).

### 3) `RenderOnce` for stateless components

Stateless, reusable “components” implement a `RenderOnce`-like trait:

- signature consumes `self` (props), returns something convertible to an element tree,
- components do not own long-lived state (state belongs in models or element state stores).

This keeps components easy to reason about and easy to reuse across `fret-components`.

### 4) `IntoElement` as the unifying conversion boundary

To keep APIs ergonomic and composable:

- any element, view output, or component output must be convertible into a common element representation
  (conceptually `IntoElement`).
- containers accept children as `impl IntoElement` (builder-style authoring).

This enables an immediate-feel authoring style without a diff engine.

### 5) Derive macro support is part of the plan (ergonomics is a product feature)

To make the component layer practical at editor scale, Fret should support a derive macro similar to:

- `#[derive(IntoElement)]` for `RenderOnce` components.

This is not required for correctness, but it is critical for:

- reducing boilerplate in `fret-components`,
- keeping APIs fluent (builder patterns with `.child(...)`, `.children(...)`, etc.),
- enabling debug tooling that can capture source locations for inspector visualization (ADR 0036).

### 6) ID and keying rules apply consistently across layers

Regardless of authoring layer:

- dynamic lists/trees must use explicit keys to preserve identity (ADR 0028),
- focus/IME/capture stability depends on identity stability (ADR 0012 / ADR 0020).

## Consequences

- Fret can support both “simple composition” and “low-level custom widgets” without forcing a single style.
- `fret-components` can be built as a cohesive ecosystem with predictable ergonomics (ADR 0037).
- Debug tooling can show meaningful structure (components/views) even though elements are rebuilt each frame.

### Interaction-first primitives live in components by default

Some editor UX affordances are better modeled as **component-level primitives** that expand into existing scene ops,
instead of immediately becoming new core primitives. This keeps `fret-core` contracts small and stable.

Examples:

- Dashed selection rectangles / docking drop-zone highlights: implement as a component that emits multiple short
  `SceneOp::Quad` segments (ADR 0030), rather than requiring “dashed borders” on `SceneOp::Quad` in P0.
  This yields stable, interaction-oriented semantics (per-edge restart, stable phase under resize) without committing
  the renderer to a specific dashed-border shader model early.

## Alternatives Considered

### A) Retained widgets only (no element/view split)

Pros:

- fewer concepts initially.

Cons:

- becomes hard to scale authoring ergonomics without introducing a diff/reconcile layer,
- tends to fight Rust borrowing once UI complexity grows.

### B) React-style diff/reconcile as the primary model

Pros:

- familiar to web developers.

Cons:

- significantly more machinery early (keyed diff, reconciliation, retained host tree),
- not clearly aligned with the “GPUI responsibilities + ImGui freedom” goal.

## Open Questions (To Decide Before Implementation)

### Locked P0 Choices

1) **Placement**: authoring traits live in `fret-ui`.
   - The macro support lives in a separate proc-macro crate (e.g. `fret-macros`) in the core repo.

2) **Allocation model**: frame-local element arena.
   - Elements are rebuilt each frame; the runtime uses a frame-local arena (or bump allocator) to reduce allocations.
   - Persistent state is held only in model store (ADR 0031) and the element state store (ADR 0028).

3) **Source locations and inspector identities**:
   - In debug builds, callsite/source location is captured via `#[track_caller]` and threaded into inspector hooks (ADR 0036).
   - In release builds, source locations are not stored.

4) **Theme/settings access**:
   - Components access theme tokens via a `Theme`/`StyleSystem` service (ADR 0032) and read-only settings views (ADR 0014).
   - Components do not read files directly; they only query the resolved services from `App`/context.
