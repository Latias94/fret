# ADR 0189: Foreign UI Embedding via Isolated Surfaces

Date: 2026-01-21  
Status: Proposed  
Owners: Fret team

## Context

Fret targets editor-grade applications and explicitly supports embedding engine viewports and other GPU-produced content via `RenderTargetId` + `SceneOp::ViewportSurface` (ADR 0007) and viewport input forwarding (ADR 0025 / ADR 0147).

As the ecosystem grows, users will ask for 鈥渦sing Fret together with other UI ecosystems鈥?(e.g. Iced, egui, engine debug UIs, custom wgpu renderers). The high-risk failure mode is to attempt to merge two widget trees into a single semantics/focus/IME/invalidation model, which tends to:

- bloat kernel contracts (`crates/*`) with policy and third-party semantics,
- create unclear ownership boundaries for focus, IME composition, and accessibility,
- force premature stabilization of integration behaviors that are hard to change later.

We need an explicit, stable policy that preserves kernel boundaries while still enabling practical interop.

## Decision

Fret supports **foreign UI integration only via isolated embedding surfaces**:

- Foreign UI is rendered into an app-owned offscreen render target (`ViewportRenderTarget`).
- Fret composites that target into the UI scene via `ViewportSurface`.
- Pointer/keyboard input can be forwarded to the embedded surface using `ViewportInputEvent`, with explicit mapping and capture rules.
- **Focus/IME/a11y are not shared across runtimes**. The embedded foreign UI is treated as an isolated surface from the perspective of Fret鈥檚 semantics tree and input routing.

Any ergonomics helpers to reduce boilerplate must live in ecosystem crates (e.g. `fret-kit`, `fret-ui-kit`), not in kernel crates.

## Goals

- Provide a safe, stable interop story that does not compromise kernel boundaries.
- Make the 鈥淭ier A鈥?integration path (engine viewports / GPU-heavy panels) the default for foreign UI.
- Keep room for future improvements (better tooling, more helpers) without changing the kernel contract.

## Non-goals

- Mixing a foreign widget tree as first-class Fret `Element`s with shared layout, shared hit testing, shared focus, or shared accessibility semantics.
- Providing a universal bridge for IME composition or text selection across runtimes.
- Guaranteeing perfect behavioral parity between foreign UI frameworks.

## Contract (What 鈥淪upported鈥?Means)

### Supported (Isolated embedding)

**Rendering**

- The host allocates and resizes an offscreen `wgpu::Texture` (via `ViewportRenderTarget`).
- The host registers/updates the render target in `fret-render` to obtain a `RenderTargetId`.
- The UI renders a leaf `ViewportSurface` referencing that `RenderTargetId`.

**Input**

- Fret forwards viewport-mapped events as `ViewportInputEvent` (window + mapping + UV + px).
- The host may translate those events into the foreign UI鈥檚 input model.
- Pointer capture may be used to continue forwarding move/up outside the surface bounds (ADR 0049 / ADR 0168).

**Boundaries**

- Fret鈥檚 semantics snapshot may expose the embedded surface as a single semantic node (e.g. `SemanticsRole::Viewport`) with a label/test id, but does not attempt to expose the foreign UI鈥檚 internal accessibility tree.
- Fret does not guarantee foreign UI focus/IME correctness; foreign UI may implement its own focus within the embedded surface if needed.

### Not supported (Widget tree mixing)

These are explicitly out of scope for the kernel:

- sharing the foreign UI鈥檚 internal hit-testing as part of Fret鈥檚 hit test,
- merging two accessibility trees into one snapshot,
- unified keyboard focus traversal across runtimes,
- unified text input/IME composition across runtimes.

## Recommended Implementation Pattern (Ecosystem-level)

Use a golden-path driver hook surface and keep the embedding policy centralized:

- Use `UiAppDriver::viewport_input(...)` to forward viewport input into a single handler.
- Use `UiAppDriver::record_engine_frame(...)` (ADR 0038) to record the foreign UI鈥檚 GPU pass into the offscreen target.
- Use `fret-ui-kit`鈥檚 declarative `viewport_surface_panel` to present the target in UI.
- Prefer `fret-kit` helpers to reduce boilerplate without changing kernel contracts.

### `fret-kit` helper surface

The recommended entry point for apps is:

- `ecosystem/fret-kit/src/interop/embedded_viewport.rs`

It provides two integration styles (both keep semantics/focus/IME isolated):

1. **Host-recorded embedded viewport**
   - App state owns an `EmbeddedViewportSurface` and implements `EmbeddedViewportRecord`.
   - Driver wiring is one call: `UiAppDriverExt::drive_embedded_viewport()` (or MVU variant).

2. **Foreign UI hosted inside an embedded surface**
   - App state owns an `EmbeddedViewportSurface` and implements `EmbeddedViewportSurfaceOwner`.
   - A foreign runtime implements `EmbeddedViewportForeignUi` (object-safe boundary).
   - The app registers it per window using `set_foreign_ui(app, window, ui)`.
   - Driver wiring is one call: `EmbeddedViewportForeignUiAppDriverExt::drive_embedded_viewport_foreign()` (or MVU variant).

This design keeps the 鈥渋nterop contract鈥?explicit (render target + input forwarding) and allows adapters for
other ecosystems (Iced/egui/custom wgpu passes) without pulling their semantics or layout models into the kernel.
## Alternatives Considered

### A1) Full widget tree integration

Pros:

- 鈥淔eels native鈥?if it worked perfectly.

Cons:

- forces kernel-level policy and third-party semantics,
- hard to stabilize and extremely expensive to maintain,
- likely breaks portability (wasm/web/embedded) and editor-grade correctness expectations.

Rejected.

### A2) Plugin runtime that hosts foreign UI internally

Pros:

- potentially isolates dependencies behind a plugin boundary.

Cons:

- still needs shared semantics/focus/IME if it wants deep integration,
- adds significant platform and tooling complexity.

Deferred (not a replacement for the core embedding story).

## Consequences

- The default interop story is simple and robust: **render target + input forwarding**.
- Rich, unified accessibility/focus across mixed runtimes is not a goal; apps that require it should build native Fret components instead.
- Ecosystem crates can iterate on ergonomics quickly without kernel churn.

## Evidence / Related ADRs

- Viewport surfaces contract: `docs/adr/0007-viewport-surfaces.md`
- Viewport input forwarding: `docs/adr/0025-viewport-input-forwarding.md`, `docs/adr/0147-viewport-input-forwarding-explicit-units.md`
- Engine render hook pipeline: `docs/adr/0038-engine-render-hook-and-submission-coordinator.md`
- Viewport tool capture and overlays: `docs/adr/0049-viewport-tools-input-capture-and-overlays.md`, `docs/adr/0168-viewport-tooling-host-helpers-and-arbitration-v1.md`
- Ecosystem helpers: `ecosystem/fret-kit/src/interop/embedded_viewport.rs`
- Examples:
  - `apps/fret-examples/src/imui_editor_proof_demo.rs` (host-recorded embedded surface)
  - `ecosystem/fret-kit/src/interop/embedded_viewport.rs` (`EmbeddedViewportForeignUi`, `set_foreign_ui`, `drive_embedded_viewport_foreign`)
