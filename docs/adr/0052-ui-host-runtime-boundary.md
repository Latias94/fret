# ADR 0052: UI Host Runtime Boundary (Embeddable Core vs Integrated App)

Status: Accepted (Prototype implemented; still iterating)
Scope: Workspace-level crate boundaries (`fret-ui`, `fret-app`, runner/platform)

## Context

Fret currently ships as a cohesive workspace where:

- `fret-ui` implements the retained UI runtime and declarative elements (ADR 0005 / ADR 0028 / ADR 0039),
- `fret-app` provides the app runtime (models, globals, effects, scheduling, commands) (ADR 0001 / ADR 0031 / ADR 0034),
- `fret-runner-winit-wgpu` drains effects, drives window lifecycle, and presents frames.

This is productive for the demo and for “first-party editor apps”.
However, the long-term project goal includes **third-party editor reuse** (GPUI-like adoption) and **wasm/WebGPU**
as a mid-term target.

Today, parts of `fret-ui` depend directly on `fret-app::App` (for example in the elements runtime and `ElementCx`).
This creates a decision point:

- Should the “public framework surface” be intentionally **integrated** (third parties adopt `fret-app`), or
- Should `fret-ui` become **embeddable** behind a small host trait layer (third parties provide the host runtime)?

This choice is difficult to change later because it permeates:

- how element-local state is stored and garbage-collected,
- how widgets dispatch commands and schedule redraws/timers,
- how model observation (ADR 0051) is implemented across windows,
- how platform services (clipboard/IME/drag-and-drop) are represented.

## Goals

- Preserve the existing architecture constraints and contracts (multi-window, overlays, ordered scene ops).
- Allow an engine/editor host to integrate without “accepting the entire Fret app runtime” if they cannot.
- Keep `fret-core` platform-agnostic and wasm-capable.
- Avoid premature over-abstraction that slows down MVP iteration.

## Options

### Option A — Integrated “GPUI-style” Workspace (status quo, explicit)

Define the public framework as:

- `fret-core` + `fret-app` + `fret-ui` (+ official runners/backends).

Third-party apps embed by adopting `fret-app::App` as their runtime.

Pros:

- Strong consistency and fewer abstraction layers.
- Fewer trait objects / less API surface to stabilize.
- The demo is already aligned with this direction.

Cons:

- More invasive for hosts that already have an engine runtime, scheduler, or ECS.
- Harder to ship a “thin” embeddable core for wasm or alternate runtimes.

### Option B — Embeddable UI Core (recommended direction)

Introduce a small host boundary crate (name TBD, e.g. `fret-runtime`) that defines traits used by `fret-ui`,
such as:

- `UiHost` / `EffectSink`: enqueue effects, request redraw, RAF, timers,
- `ModelHost`: model read/update + changed-model propagation (align with ADR 0031 / ADR 0051),
- `TextHost`: `TextService` access (already trait-based in `fret-core`),
- optional: `ClipboardHost` / `ImeHost` / `PlatformHost` (align with ADR 0003 / ADR 0041),
- a monotonic `frame_id` / `tick_id` provider (align with ADR 0034 / ADR 0015).

`fret-app` becomes the default implementation of these traits (and continues to power the demo).
Third-party apps may implement the traits directly or wrap their own runtime.

Pros:

- Easier adoption by engines/editors with existing runtimes.
- A clearer path to wasm/WebGPU where platform services and file handles differ.
- Reduces the risk that demo glue hard-binds `fret-ui` to `fret-app` internals.

Cons:

- Requires stabilizing a trait surface early (maintenance cost).
- Risk of “leaky abstractions” if the trait boundary is poorly chosen.

## Decision

We converge on **Option B**, implemented incrementally.

Immediate (P0) contract to lock:

- Do not move more `fret-app::App` concepts into `fret-ui` public APIs.
- Any new framework-level dependencies from `fret-ui` should be expressible via small traits.

Prototype implementation (P0, landed):

- Introduce a minimal `UiHost` trait used by the `fret-ui` runtime.
- Implement `UiHost` for `fret_app::App` so the demo behavior stays unchanged.
- Use default type parameters so most framework-facing types remain ergonomic:
  - `Widget<H: UiHost = fret_app::App>`
  - `EventCx<'_, H>`, `LayoutCx<'_, H>`, etc.

Deferred work (P1):

- Consider extracting `UiHost` into a small dedicated crate (e.g. `fret-runtime`) once the trait surface stabilizes.
- Reduce host trait coupling to `fret_app` types over time (phase approach):
  - Phase 0: boundary exists, but host types may still come from `fret_app` (current).
  - Phase 1: move host-facing types into a portable crate (`fret-runtime` or `fret-core` where appropriate).
  - Phase 2: third-party hosts implement the traits without adopting `fret-app`.

## Proposed API Shape (Sketch, not final)

This is intentionally a sketch to drive discussion; the exact surface should be kept minimal.

- `trait UiHost { fn frame_id(&self) -> FrameId; fn request_redraw(&mut self, window: AppWindowId); fn push_effect(&mut self, Effect); ... }`
- `trait ModelHost { fn take_changed_models(&mut self) -> Vec<ModelId>; ... }`
- `trait GlobalHost { fn get_global<T: Any>(&self) -> Option<&T>; fn with_global_mut<T: Any, R>(&mut self, init: fn() -> T, f: impl FnOnce(&mut T, &mut Self) -> R) -> R; }`

Non-goal: redesign the entire authoring model; ADR 0028/0039 remain the authoring contract.

## Consequences

- We keep shipping MVP features with the current integrated workspace.
- We document and constrain future work so that an embeddable core remains feasible.
- We reduce the risk of an expensive “late decoupling” refactor.

## Next Steps

1. Inventory `fret-ui -> fret-app` dependencies and categorize them:
   - essential host services vs accidental convenience.
2. Keep tightening `UiHost` to the smallest “host services” surface we can get away with.
3. Move element-local state storage to be UI-runtime-owned where possible (avoid requiring host globals).
4. If/when a dedicated trait crate exists, update `docs/architecture.md` crate layout accordingly.

## References

- ADR 0001: `docs/adr/0001-app-effects.md`
- ADR 0003: `docs/adr/0003-platform-boundary.md`
- ADR 0005: `docs/adr/0005-retained-ui-tree.md`
- ADR 0027: `docs/adr/0027-framework-scope-and-responsibilities.md`
- ADR 0028: `docs/adr/0028-declarative-elements-and-element-state.md`
- ADR 0031: `docs/adr/0031-app-owned-models-and-leasing-updates.md`
- ADR 0034: `docs/adr/0034-timers-animation-and-redraw-scheduling.md`
- ADR 0037: `docs/adr/0037-workspace-boundaries-and-components-repository.md`
- ADR 0051: `docs/adr/0051-model-observation-and-ui-invalidation-propagation.md`
