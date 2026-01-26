# Fret Architecture (Draft)

Fret is a Rust GUI framework aimed at building a game editor with a **Unity/Unreal/Godot-like** workflow: docking, tear-off windows, multiple viewports, and layered rendering. The long-term goal is **Windows/macOS/Linux first**, then **wasm (WebGPU)**, and eventually mobile.

This document intentionally focuses on decisions that minimize future rewrites.

If you are new to the repository, start with `docs/README.md`.

## Scope

Fret is a **UI framework** for building engine editors, not “the editor itself”.

- Framework scope (in Fret): windowing/event loop boundary, UI runtime, docking UX infrastructure,
  command/keymap system, display list + renderer, viewport embedding contracts.
- Editor app / engine scope (out of Fret): asset pipeline, scene/ECS model, selection/gizmo/tool systems,
  undo/redo history policy, project/build/indexing.

See `docs/adr/0027-framework-scope-and-responsibilities.md`.

## Architecture Decision Records (ADRs)

Framework cross-crate contracts are tracked as ADRs.

Current focus (hard-to-change ADRs that should be treated as “locked contracts” unless explicitly revised):

- `docs/adr/0028-declarative-elements-and-element-state.md` (declarative/composable UI model)
- `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md` (component authoring ergonomics)
- `docs/adr/0029-text-pipeline-and-atlas-strategy.md` (text shaping/atlas strategy)
- `docs/adr/0030-shape-rendering-and-sdf-semantics.md` (shape semantics over SDF)
- `docs/adr/0031-app-owned-models-and-leasing-updates.md` (ownership + borrow-friendly updates)
- `docs/adr/0032-style-tokens-and-theme-resolution.md` (typed theming and style resolution)
- `docs/adr/0033-semantics-tree-and-accessibility-bridge.md` (semantics tree + A11y bridge boundary)
- `docs/adr/0034-timers-animation-and-redraw-scheduling.md` (timers/animation/scheduling)
- `docs/adr/0035-layout-constraints-and-optional-taffy-integration.md` (layout constraints + optional Flex/Grid)
- `docs/adr/0057-declarative-layout-style-and-flex-semantics.md` (Tailwind-friendly sizing + Flex semantics)
- `docs/adr/0036-observability-tracing-and-ui-inspector-hooks.md` (tracing + inspector hooks)
- `docs/adr/0037-workspace-boundaries-and-components-repository.md` (workspace boundaries + `fret-components` repo)
- `docs/adr/0038-engine-render-hook-and-submission-coordinator.md` (engine integration without queue ownership)

For the full, module-oriented ADR index, see `docs/adr/README.md`.

## Goals

- UI runtime core suitable for large editor applications.
- Docking + tear-off windows (Imgui viewports-style UX).
- Multiple engine viewports in one window (and across windows).
- WGPU-based rendering pipeline, compatible with future WebGPU/wasm.
- Clear separation between platform (window/events), UI core (tree/layout/input), and renderer (GPU).

## Non-goals (for the first iterations)

- Full accessibility stack.
- Perfect text fidelity/IME on day one.
- Mobile support.

## High-Level Layering

1. **Platform layer**: OS windows, event loop, input events, clipboard, IME, drag-and-drop.
2. **UI core**: UI runtime (layout, hit-testing, focus/keyboard routing, docking model, display list).
3. **Renderer**: translates the display list to GPU work (wgpu), manages atlas/resources, presents to surfaces.

The renderer must support both hosting topologies via a **host-provided GPU context**:

- **Editor-hosted**: Fret creates `wgpu::Instance/Adapter/Device/Queue` and shares `Device/Queue` with the engine.
- **Engine-hosted**: the engine creates `wgpu::Instance/Adapter/Device/Queue` and passes the context to Fret.

In both cases, surface creation must use the same `wgpu::Instance` as the device (see ADR 0010).

## Crate Layout (Workspace)

Crate boundaries are locked in ADR 0093: `docs/adr/0093-crate-structure-core-backends-apps.md`.

- `crates/fret-core`: platform-agnostic core (IDs, geometry, docking model, layout/input contracts).
- `crates/fret-runtime`: host-facing runtime boundary traits + portable value types used by `fret-ui` (ADR 0052).
- `crates/fret-app`: app runtime (global services, models/entities, scheduling, command/action dispatch).
- `crates/fret-fonts`: bundled default font bytes for wasm/bootstrap (fed to `Effect::TextAddFonts`).
- `crates/fret-platform`: portable platform I/O contracts (clipboard, external drops, file dialogs, open-url).
- `crates/fret-platform-native`: native implementations for `fret-platform` contracts.
- `crates/fret-platform-web`: wasm/browser implementations for `fret-platform`-adjacent services.
- `crates/fret-runner-winit`: winit platform adapter (event mapping, cursor/modifiers/key normalization, canvas binding on web).
- `crates/fret-runner-web`: compatibility shim re-exporting `fret-platform-web` (dedicated DOM adapter TBD).
- `crates/fret-render`: wgpu-based renderer building blocks (context/device bootstrap, rendering backends).
- `crates/fret-ui`: UI runtime (layout, hit-testing, focus routing, display list builder).
- `crates/fret-launch`: integration glue (desktop now; web/mobile later) that owns presentation/effect draining and drives the frame loop.
- `ecosystem/fret-ui-kit`: component infrastructure (policies, style composition, overlay managers) built on `fret-ui`.
- `ecosystem/fret-ui-shadcn`: shadcn/ui v4 taxonomy surface + recipes built on `fret-ui-kit`.
- `crates/fret-ui-app`: integration convenience layer that binds `fret-ui` to `fret-app::App` for first-party apps/demos.
- `crates/fret`: public facade crate (re-exports).
- `apps/fret-examples`: shared end-to-end harness code (components gallery, docking demos, smoke tests).
- `apps/fret-demo`: native demo binaries (thin shells over `fret-examples`).
- `apps/fret-demo-web`: wasm demo shell (Trunk + `#[wasm_bindgen(start)]`, thin shell over `fret-examples`).
- `apps/fretboard`: dev CLI for running native/web demos and generating starter templates.

## Repository Layout (Future)

To keep the framework stable while allowing editor-grade components to scale, the planned split is:

- Core repo: `fret` (framework + backends + demos).
- Components repo: `fret-components` (multiple `fret-components-*` crates that depend on `fret-ui` but not on platform/render).

`fret-components` is expected to provide an “editor kit” (primitives + patterns), including:

- shadcn-inspired UI primitives/composites,
- tree/inspector/table/command UI patterns,
- optional specialized visualization (charts).

Backend split direction (planned):

- `fret-platform-native` (desktop), `fret-platform-web` (wasm) under the core repo.
- `fret-render-wgpu` (wgpu/WebGPU) under the core repo.

Current workspace note:

- Today these backends live as `crates/fret-platform-native` / `crates/fret-platform-web` and `crates/fret-render` (wgpu).

See `docs/adr/0037-workspace-boundaries-and-components-repository.md`.

## Versioning Notes (Upstream References)

Winit and wgpu evolve quickly, and API changes can make it easy to read the “wrong” source.

- Prefer reading the exact versions used by this workspace (via `Cargo.lock`), or check out matching git tags upstream.
- For convenience, see `docs/repo-ref.md` for pinned local reference checkouts under `repo-ref/`.

## UI Runtime Model: Stable Identity + Invalidation

### Why “retained semantics”

Editors are long-lived, complex, and stateful. Even with declarative authoring (ADR 0028), the runtime must
provide retained **semantics**:

- stable widget identity (important for docking, drag state, selection),
- predictable performance via incremental updates and caching,
- easier cross-window coordination.

Current status:

- The runtime substrate is still a retained tree (`UiTree`) that owns layout/hit-test/paint state and invalidation.
- Most new UI authoring is declarative: each frame builds an element tree (`AnyElement` via `ElementContext`) that is
  mounted into `UiTree`, with cross-frame element state externalized by stable identity (ADR 0028).
- Some legacy/low-level surfaces may still be authored directly as retained widgets, but the long-term direction for
  ecosystem components is “declarative-first / declarative-only”.

### Proposed structure

- The UI is described as a tree with stable identity:
  - retained widgets: `NodeId`,
  - declarative elements: `GlobalElementId` + explicit keys (ADR 0028).
- Each window may be composed from multiple roots (base UI + overlays/popups/modals) with an explicit z-order. (ADR 0011)
  - Roots are layered in the UI runtime and painted in root order.
  - Modal roots can block input from reaching lower roots.
- The runtime maintains:
  - layout state (bounds),
  - event state (hover, focus, capture),
  - optional caching hooks (layout/paint/scene fragments) as an optimization.
- Changes are propagated via **invalidation flags**:
  - `NeedsLayout`
  - `NeedsPaint`
  - `NeedsHitTestRebuild`

### Layout as the source of truth

Fret’s UI runtime uses the rule “**layout writes bounds**”, and both hit-testing and paint follow those bounds.

- Containers position children via an explicit API (e.g. `layout_in(child, rect)`).
- Hit-testing uses the last layout bounds (not last paint).
- Painting children should use stored bounds to stay consistent with scrolling/clipping.

Reference: `docs/adr/0005-retained-ui-tree.md`.

## App Runtime: Globals, Models, and Commands

- `App` owns global services and long-lived state (“models”), avoiding pervasive `Rc<RefCell<_>>`.
- Model updates use borrow-friendly update/lease APIs so handlers can mutate both `&mut App` and `&mut T` safely.
- Widgets enqueue side effects (redraw, window ops, timers) as data; the runner drains them centrally.
- Commands/keymap are first-class and routed through focus scopes.

References: `docs/adr/0001-app-effects.md`, `docs/adr/0031-app-owned-models-and-leasing-updates.md`, `docs/adr/0020-focus-and-command-routing.md`, `docs/adr/0021-keymap-file-format.md`, `docs/adr/0023-command-metadata-menus-and-palette.md`, `docs/adr/0034-timers-animation-and-redraw-scheduling.md`.

## Docking: Multi-Window, Tear-Off, and Cross-Window Drag

- Docking is app-owned state (global dock manager), not a single-window concern.
- Layout is pure data + explicit operations + versioned persistence (splits, tabs, tear-off roots).
- Multi-window tear-off and cross-window drag are first-class (requires platform window lifecycle + overlays).

References: `docs/adr/0013-docking-ops-and-persistence.md`, `docs/adr/0017-multi-window-display-and-dpi.md`, `docs/adr/0011-overlays-and-multi-root.md`.

## Rendering: Scene/Display List + SDF Quads

- `fret-ui` emits an ordered `Scene`/display list; draw order correctness is guaranteed by contract.
- The renderer may batch only when order is preserved (adjacent batching).
- Rounded rects/borders/shadows evolve behind stable semantics; SDF is an implementation detail.
- Text is handled via opaque `TextBlobId` + metrics boundary.

References: `docs/adr/0002-display-list.md`, `docs/adr/0009-renderer-ordering-and-batching.md`, `docs/adr/0030-shape-rendering-and-sdf-semantics.md`, `docs/adr/0006-text-system.md`, `docs/adr/0029-text-pipeline-and-atlas-strategy.md`.

## Windowing & Surfaces

- Each OS window owns a presentable surface; multi-window is required for tear-off docking.
- Platform backends are implementation details (`winit` now; web later) behind the `fret-platform-*` boundary.
- The runner is responsible for window lifecycle, event translation, scheduling (timers/raf), and draining effects.

References: `docs/adr/0003-platform-boundary.md`, `docs/adr/0017-multi-window-display-and-dpi.md`, `docs/adr/0034-timers-animation-and-redraw-scheduling.md`.

## Resource Handles & Engine Viewports

- UI code never touches `wgpu` types: it deals only in stable IDs/handles (`ImageId`, `FontId`, `RenderTargetId`, `TextBlobId`).
- The renderer owns GPU resources and resolves handles.
- Engine viewports are registered as render targets and painted via scene ops.

References: `docs/adr/0004-resource-handles.md`, `docs/adr/0007-viewport-surfaces.md`, `docs/adr/0010-wgpu-context-ownership.md`, `docs/adr/0015-frame-lifecycle-and-submission-order.md`, `docs/adr/0025-viewport-input-forwarding.md`.

## Layout Engine

- Layout stays editor-friendly: explicit layout for docking/splits, optional Flex/Grid via a layout engine.
- `fret-core` must remain layout-engine-free; layout engines are implementation details in `fret-ui`/`fret-components-*`.

Reference: `docs/adr/0035-layout-constraints-and-optional-taffy-integration.md`.

## Theme & Styling (shadcn-inspired)

Use typed style tokens and explicit theme resolution instead of a CSS parser; theme content remains app-owned.

Reference: `docs/adr/0032-style-tokens-and-theme-resolution.md`.

## Text Strategy

- Keep a stable text boundary in `fret-core` (metrics + `TextBlobId`), and evolve implementation behind it.

References: `docs/adr/0006-text-system.md`, `docs/adr/0012-keyboard-ime-and-text-input.md`, `docs/adr/0029-text-pipeline-and-atlas-strategy.md`.

## Async & Scheduling

- No hard dependency on a specific async runtime; background work communicates via messages/effects.
- Default scheduling is event-driven (idle when nothing is dirty), with explicit continuous mode when requested.
- Side effects are drained centrally in a bounded loop to keep multi-window behavior deterministic.

References: `docs/adr/0001-app-effects.md`, `docs/adr/0008-threading-logging-errors.md`, `docs/adr/0190-execution-and-concurrency-surface-v1.md`, `docs/adr/0034-timers-animation-and-redraw-scheduling.md`.

## Settings & Configuration (settings-ui-inspired)

- Fret provides file-based settings infrastructure (loading/layering/change propagation); schema and UI remain app-owned.

Reference: `docs/adr/0014-settings-and-configuration-files.md`.

## Plugin & Component Boundaries

- Plugins contribute UI-level integration points (panels/commands/menus); discovery/policy and editor-domain logic remain app-owned.

References: `docs/adr/0016-plugin-and-panel-boundaries.md`, `docs/adr/0037-workspace-boundaries-and-components-repository.md`.

## Next Reading

- Roadmap and current status markers: `docs/roadmap.md`
- Module-oriented ADR index: `docs/adr/README.md`
- Pinned upstream references: `docs/repo-ref.md`
- Current focus ADRs (decision gates): `docs/adr/0028-declarative-elements-and-element-state.md`, `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md`, `docs/adr/0038-engine-render-hook-and-submission-coordinator.md`, `docs/adr/0031-app-owned-models-and-leasing-updates.md`, `docs/adr/0034-timers-animation-and-redraw-scheduling.md`, `docs/adr/0036-observability-tracing-and-ui-inspector-hooks.md`, `docs/adr/0037-workspace-boundaries-and-components-repository.md`
