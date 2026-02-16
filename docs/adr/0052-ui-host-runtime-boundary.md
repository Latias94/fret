# ADR 0052: UI Host Runtime Boundary (Embeddable Core vs Integrated App)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Accepted (Prototype implemented; still iterating)
Scope: Workspace-level crate boundaries (`fret-ui`, `fret-app`, runner/platform)

## Context

Fret currently ships as a cohesive workspace where:

- `fret-ui` implements the retained UI runtime and declarative elements (ADR 0005 / ADR 0028 / ADR 0039),
- `fret-app` provides the app runtime (models, globals, effects, scheduling, commands) (ADR 0001 / ADR 0031 / ADR 0034),
- `fret-launch` drains effects, drives window lifecycle, and presents frames.

This is productive for the demo and for “first-party editor apps”.
However, the long-term project goal includes **third-party editor reuse** (GPUI-like adoption) and **wasm/WebGPU**
as a mid-term target.

Today, parts of `fret-ui` depend directly on `fret-app::App` (for example in the elements runtime and `ElementContext`).
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
- Place `UiHost` (and portable host-facing value types) in `fret-runtime` so `fret-ui` does not depend on `fret-app`.
- Implement `UiHost` for `fret_app::App` so the demo behavior stays unchanged.
- Preserve integrated-app ergonomics via a bridge crate:
  - `fret-ui-app` provides `fret-app`-bound type aliases (`UiTree`, `EventCx`, etc.) and common re-exports.

Deferred work (P1):

- Reduce host trait coupling to `fret_app` types over time (phase approach):
  - Phase 0: boundary exists (host-generic `fret-ui`).
  - Phase 1: host-facing types are portable (`fret-runtime` or `fret-core` where appropriate).
  - Phase 2: third-party hosts implement the traits without adopting `fret-app`.

## Phase Checklist (Living)

This section is a **living checklist** to keep the Option B plan concrete and to prevent accidental re-coupling.

### Phase 0 — Boundary exists (prototype; still uses `fret-app` host types)

- [x] Introduce `UiHost` used by the `fret-ui` runtime (`crates/fret-runtime/src/ui_host.rs`).
- [x] Implement `UiHost` for `fret_app::App` to keep the demo behavior unchanged (`crates/fret-app/src/ui_host.rs`).
- [x] Keep widget authoring host-generic (`Widget<H: UiHost>`): `crates/fret-ui/src/widget.rs` and `crates/fret-ui/src/declarative/host_widget/*`.
- [x] Keep retained-compat helpers explicitly feature-gated: `crates/fret-ui/src/retained_bridge.rs` (feature: `unstable-retained-bridge`).
- [x] Remove the `fret-ui -> fret-app` dependency.
- [x] Provide an integrated-app bridge crate (`crates/fret-ui-app`) for demo/editor ergonomics.
- [ ] Inventory remaining coupling inside `UiHost` and consider splitting into smaller service traits.

### Phase 1 — Portable host-facing types (reduce `fret-app` in the public boundary)

Goal: keep `fret-ui` depending on `fret-core` + a small portable “runtime boundary” crate.

- [x] Extract host-facing *types* that leaked `fret_app` (e.g. `Effect`, `CommandId`, `InputContext`,
      drag session types, menu types) into `fret-runtime` (or `fret-core` where appropriate).
- [ ] Keep the boundary minimal: define only “services” (scheduling, redraw, effects, commands) and avoid editor semantics.
- [ ] Ensure portability gates exist for platform-specific features (see ADR 0054) and that payload types avoid desktop-only
      values (see ADR 0053).

### Phase 2 — Third-party host integration (no `fret-app` adoption required)

- [ ] Provide a minimal example host implementation (engine/editor runtime) that drives `UiTree` via `UiHost` without
      depending on `fret-app`.
- [ ] Confirm wasm/WebGPU runner can implement the boundary with a single-window capability profile (see ADR 0054).

## Current Coupling Hotspots (Non-exhaustive)

These are the areas to watch when tightening the boundary:

- `UiHost` still represents multiple host services (effects, commands, drag sessions, scheduling). The types are now portable
  (`fret-runtime`), but the trait surface may still be split further over time.
- UI contexts (`EventCx`, `CommandCx`, `LayoutCx`, `PaintCx`) embed portable boundary types (`fret-runtime`), and dispatch
  host outputs as `Effect`.
- `elements` runtime and element-local state storage may still assume host globals (see ADR 0028 / ADR 0039).

## Post-Phase1 Inventory (Current Boundary Shape)

This inventory is meant to stay actionable: it lists where the UI runtime still assumes “host services”, even though the
types are now portable and no longer tie `fret-ui` to `fret-app`.

### `crates/fret-runtime/src/ui_host.rs` — host service surface

Notes:

- The trait currently bundles multiple services (globals, models, commands, scheduling, drag sessions, effect output).
- Next tightening step is likely *splitting* into smaller service traits (e.g. `EffectSink`, `Globals`, `Models`, `Commands`, `Timers`, `DragHost`)
  so third-party hosts can implement only what they need.

### `crates/fret-ui/src/tree/mod.rs` — retained UI runtime

Notes:

- The retained tree currently expects to resolve shortcuts and dispatch commands via host globals (`KeymapService`) and
  command metadata; these live in the host runtime but are portable value types.
- If we want a smaller host surface for embedding, this is the hotspot to refactor: either introduce a thin host-facing
  trait for keymap/commands, or keep the concrete services in `fret-runtime` and treat them as part of the public contract.

### `crates/fret-ui/src/elements/mod.rs` — element-local state runtime

Notes:

- This is close to portable already; it mostly needs a global `ElementRuntime` store and frame/tick IDs.

### `crates/fret-ui-app` — integrated ergonomics

Notes:

- `fret-ui-app` is intentionally optional and exists only to keep first-party apps ergonomic (`impl Widget for ...`,
  `UiTree` aliases, etc.) while the core UI runtime stays embeddable.

## Proposed API Shape (Sketch, not final)

This is intentionally a sketch to drive discussion; the exact surface should be kept minimal.

- `trait UiHost { fn frame_id(&self) -> FrameId; fn request_redraw(&mut self, window: AppWindowId); fn push_effect(&mut self, Effect); ... }`
- `trait ModelHost { fn take_changed_models(&mut self) -> Vec<ModelId>; ... }`
- `trait GlobalHost { fn get_global<T: Any>(&self) -> Option<&T>; fn with_global_mut<T: Any, R>(&mut self, init: fn() -> T, f: impl FnOnce(&mut T, &mut Self) -> R) -> R; }`
  - P1: `fn take_changed_globals(&mut self) -> Vec<TypeId>` for global observation propagation (see ADR 0051).

Non-goal: redesign the entire authoring model; ADR 0028/0039 remain the authoring contract.

## Consequences

- We keep shipping MVP features with the current integrated workspace.
- We document and constrain future work so that an embeddable core remains feasible.
- We reduce the risk of an expensive “late decoupling” refactor.

## Next Steps

1. Keep tightening `UiHost` to the smallest “host services” surface we can get away with (consider splitting into smaller traits).
2. Provide a minimal example host implementation (not using `fret-app`) that can drive `UiTree` for embedding validation.
3. Move element-local state storage to be UI-runtime-owned where possible (avoid requiring host globals).
4. Keep runner/backends integrated for now; revisit runner genericity only when a real third-party host needs it.

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
- ADR 0053: `docs/adr/0053-external-drag-payload-portability.md`
- ADR 0054: `docs/adr/0054-platform-capabilities-and-portability-matrix.md`
- Zed/GPUI crate split as an empirical reference (non-normative):
  - runtime substrate: `repo-ref/zed/crates/gpui`
  - policy-heavy UI surfaces: `repo-ref/zed/crates/ui`
  - app integration glue and runners: `repo-ref/zed/crates/gpui_tokio`, `repo-ref/zed/crates/zed`
