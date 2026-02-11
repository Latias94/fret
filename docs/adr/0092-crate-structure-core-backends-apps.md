# ADR 0092: Crate Structure (Core / Backends / Apps)

Status: Accepted

## Context

Fret aims to be an editor-grade, cross-platform UI framework:

- desktop-first (Windows/macOS/Linux),
- wasm/WebGPU later,
- multiple windows + docking tear-offs,
- strong keyboard UX and IME-correct text input.

The repository already contains most of the required layers (`fret-core`, `fret-ui`, `fret-render`,
`fret-platform-*`, `fret-runner-*`), but the boundaries are easy to blur:

- “runner” crates can accidentally become “everything crates” (event loop + surfaces + renderer + platform I/O + demo),
- web integration may diverge (winit-on-wasm vs direct browser APIs),
- demos and experiments can leak policy or platform dependencies into core crates.

We want a Bevy-like structure: stable “core crates” plus optional platform backends, with runnable apps living
outside the framework kernel.

Related ADRs:

- Platform backends (native + web): `docs/adr/0090-platform-backends-native-web.md`
- Dependency policy / layering: `docs/dependency-policy.md`
- Framework scope boundary: `docs/adr/0027-framework-scope-and-responsibilities.md`

## Decision

### 1) We distinguish three layers: core, backends, and apps

1. **Core crates** define the framework’s portable contracts and the UI/runtime/render architecture.
2. **Backend crates** adapt platform APIs (winit, browser DOM, OS services) into Fret’s core contracts.
3. **Apps** are runnable binaries / demos / experiments that assemble core + backends into an end-to-end product.

This keeps the “framework kernel” small and portable, while still making it easy to ship working demos.

### 2) Core crates (portable, stable boundaries)

Core crates live in `crates/` and must remain backend-agnostic:

- `fret-core`: IDs, geometry, input/event types, and other portable contracts.
- `fret-runtime`: portable runtime services and value types shared by `fret-ui` and app/runtime.
- `fret-app`: application runtime (models, effects, commands, scheduling).
- `fret-ui`: UI runtime (tree/layout/hit-testing/focus/event routing) producing a backend-agnostic scene.
- `fret-render-core`: portable render-facing contract types.
- `fret-render-wgpu`: wgpu renderer implementation (pipelines + text/svg rasterization + uploads).
- `fret-render`: compatibility facade for the default renderer backend.
- `fret-fonts`: bundled default font bytes for wasm/bootstrap (fed to `Effect::TextAddFonts`).
- `fret-platform`: portable platform I/O contracts (clipboard, file dialogs, external drop reading, open-url).
- `fret`: facade crate (re-exports). It must not pull in backends by default.

### 3) Backend crates (platform integration adapters)

Backend crates also live in `crates/`, but are explicitly platform-/API-specific:

- `fret-platform-native`: native (non-wasm) implementations of `fret-platform`.
- `fret-platform-web`: wasm/browser implementations of `fret-platform` services.
- `fret-runner-winit`: winit adapter responsible for translating winit window/input events into `fret-core::Event`
  and maintaining winit-specific per-window state (cursor, IME cursor area, etc).
- `fret-runner-web`: browser DOM adapter responsible for translating DOM/canvas events into `fret-core::Event`
  and providing RAF scheduling hooks.

Notes:

- On wasm, we prefer using browser APIs (`web-sys`) directly for keyboard/IME fidelity and to avoid coupling the web
  backend to a particular event-loop crate. Using winit-on-wasm is allowed only when it is clearly beneficial, but is
  not the default direction for the web backend.
- Backend crates may depend on core crates, but core crates must not depend on backend crates.

### 4) Apps live in `apps/` (or as explicitly “demo” crates)

Apps are allowed to depend on backends and may own:

- the event loop / RAF scheduling,
- window/surface creation and presentation,
- effect draining (clipboard/file dialogs/etc),
- integrating engine-owned GPU contexts (ADR 0010).

Apps should not be considered part of the framework kernel, even if they are built in CI.

### 4b) We provide a small “launcher” glue crate

To make it easy to write “one codebase, multiple platforms” apps, we allow a small integration crate:

- `crates/fret-launch`: a thin cross-platform launcher facade that depends on backend crates and provides a
  convenient `launch(...)` entry point.

It is not part of the portable framework kernel. Its purpose is to keep application code ergonomic while core
crates remain backend-agnostic.

Transitional note:

- The historical integrated desktop runner crate (`fret-runner-winit-wgpu`) has been folded into `crates/fret-launch`.
  The long-term direction remains: keep backends thin and move “app glue” to `apps/` as it stabilizes.

### 5) Layering invariants (hard rules)

- Core crates (`fret-core`, `fret-runtime`, `fret-app`, `fret-ui`) must not depend on:
  - `winit`, `wgpu`, `web-sys`, or any `fret-platform-*` / `fret-runner-*` backend crates.
- Component/policy crates in `ecosystem/` must not depend on backend crates.
- Backend crates must not depend on component/policy crates.
- Apps may depend on anything, but should avoid introducing new cross-crate contracts without an ADR.

## Consequences

- The framework kernel stays portable and stable while we iterate on platform integrations.
- Desktop and web input/IME can evolve independently, without forcing a single event-loop abstraction.
- “Delete / replace a runner” becomes an app-level change, not a framework rewrite.

## Future Work

- Decide whether to rename `fret-runner-*` crates to `fret-backend-*` to better match their role (pure adapters).
- Move integrated desktop glue (`winit + wgpu + surfaces`) out of the framework kernel and into `apps/`.
- Define a minimal “default app” wiring pattern (Bevy’s `DefaultPlugins`-class concept) in a way that does not pull
  backends into the `fret` facade by default.
