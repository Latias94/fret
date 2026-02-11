# ADR 0106: Ecosystem Bootstrap, UI-Assets Convenience Layer, and Dev Tools

Status: Accepted

## Context

ADR 0092 locks the core/backends/apps crate structure, keeping the portable framework kernel small and stable.
ADR 0037 further states that policy-heavy component surfaces may eventually move to a separate repository.

As the repository grows, we need to make "how do I build an app with Fret?" obvious:

- which crates are stable contracts vs fast-moving defaults,
- where "use_asset" style UI resource conveniences live,
- how to provide a "golden path" startup experience without polluting kernel crates,
- how dev workflows (native vs wasm, hotpatch vs reload) integrate cleanly.

Today there is a mild cognition issue in the ecosystem area:

- `ecosystem/fret-asset-cache` provides `use_asset`-style caching for render resources (Image/SVG) and budgeting/eviction.
  The name can be misread as "editor asset pipeline", which is explicitly out-of-scope for the framework kernel
  (ADR 0027 / `docs/architecture.md`).
- Demo apps often re-implement the same startup steps: settings load, theme selection, icon pack registration, optional
  SVG preload, resource budget tuning, and dev toggles.

Separately, we want a developer tool entry point:

- a CLI (`fretboard`) that can run native/web demos with consistent flags and (optionally) enable hotpatch mode,
  without making the framework crates depend on toolchain logic.

## Goals

- Make dependency selection obvious:
  - kernel vs ecosystem vs tools.
- Provide a "golden path" startup layer as an ecosystem crate that composes existing primitives, without adding new core contracts.
- Rename or reframe "asset cache" to clearly mean "UI render assets", not "editor asset pipeline".
- Define how dev tools (CLI) fit into the layering story.
- Keep web (wasm) developer experience ergonomic without pushing toolchain concerns into library crates.

## Non-goals

- Changing ADR 0092's core/backends/apps boundaries.
- Introducing an editor-grade asset database/import pipeline into framework crates (ADR 0026 remains out-of-scope for kernel).
- Forcing a single bootstrap style; advanced apps may assemble crates manually.

## Constraints (Alignment With Existing ADRs)

- Kernel crates (`fret-core`, `fret-runtime`, `fret-app`, `fret-ui`) must remain independent of ecosystem defaults and dev tools.
- Backend crates must not depend on ecosystem policy crates.
- The `fret` facade must not pull in backends or ecosystem defaults by default (ADR 0092).
- Resource ownership stays in the renderer with handle-based IDs (ADR 0004).
- `fret-ui-app` is an allowed "core-to-core integration convenience" crate, but must remain backend-agnostic.

## Decision

### 0) Terminology: "UI render assets" vs "editor project assets"

To avoid scope drift, we standardize terms:

- **UI render assets**: bytes/resources used to render UI (icons, images, SVGs, glyph atlases), registered via
  effect-driven flush points and referenced by stable IDs (ADR 0004).
- **Editor project assets**: engine/editor assets with GUID identity, import pipelines, dependency graphs, and derived
  artifacts (ADR 0026; out-of-scope for the framework kernel).

### 1) Introduce an ecosystem "golden path" startup crate: `fret-bootstrap`

We add `ecosystem/fret-bootstrap` as a composition layer that makes app/demo startup ergonomic, without changing core contracts.

Responsibilities (opinionated but optional):

- configure a default runner (`fret-launch` builder wiring),
- load common settings files (e.g. `.fret/settings.json`) and apply to runner config,
- register icon packs and apply theme presets,
- configure UI render-asset caches (budgets, limits),
- provide opt-in dev features:
  - enable hotpatch mode (ADR 0105 integration),
  - enable debug overlays / inspector hooks where applicable.

Non-responsibilities:

- `fret-bootstrap` must not become a new "runtime". It only wraps/configures existing `fret-launch` and ecosystem helpers.
- `fret-bootstrap` must not define new cross-crate contracts that belong in kernel crates; if needed, add a separate ADR.

Implementation note:

- This crate is allowed to be implemented incrementally and stay "thin": it should primarily assemble `fret-launch`
  configuration + a small set of opt-in ecosystem defaults. It should not become a second app runtime.

### 2) Define a dedicated "UI render assets" convenience layer (rename or alias)

We explicitly classify `use_asset`-style caching as **UI render assets** (not editor project assets).

We introduce a clearer naming surface:

- Preferred crate name: `fret-ui-assets` (ecosystem).
- It provides:
  - `ImageAssetCache` + `SvgAssetCache` (and future render-asset caches),
  - budgets, LRU/eviction rules, stats for debug overlays,
  - unified "drive caches from runner events" helpers.

Migration path:

- Short term: keep `fret-asset-cache` as-is, but document it as "UI render assets". (Implemented.)
- Medium term: add `fret-ui-assets` as a new crate that re-exports `fret-asset-cache` modules, then migrate call sites. (Implemented; migration in progress.)
- Long term: deprecate `fret-asset-cache` name if desired, keeping API paths stable where possible.

### 3) Remove `fret-app-kit` and keep responsibilities split

`fret-app-kit` historically mixed "app-level defaults" and "UI render-asset convenience". Since this repository
is still pre-OSS and has no external users yet, we remove it entirely and keep the story crisp:

- App defaults / startup glue live in `ecosystem/fret-bootstrap`.
- UI render-asset caches and helpers live in `ecosystem/fret-ui-assets` (re-export surface over `fret-asset-cache`).

### 4) Introduce a dev-tools layer as a separate distribution: `fretboard` (CLI)

We define a developer tool entry point as a separate crate/binary:

- Name: `fretboard` (CLI), distributed on crates.io as a binary tool.
- Responsibilities:
  - `fretboard dev native`: run a chosen demo/app with consistent flags and environment,
  - `fretboard dev web`: run the wasm harness via a devserver (e.g. `trunk serve`),
  - `fretboard dev native --hotpatch`: enable hotpatch mode by selecting the appropriate features/args (ADR 0105).
- Non-responsibilities:
  - The CLI is not a runtime contract; framework crates must not depend on it.
  - The CLI does not define new UI behaviors; it orchestrates build/run workflows.

Implementation status (in this workspace):

- `apps/fretboard` provides a minimal `fretboard` CLI suitable for local development.

### 4b) Implementation Status (Current)

Implemented:

- `ecosystem/fret-ui-assets` exists as the preferred UI render-asset surface (re-export wrapper over `fret-asset-cache`).
- `ecosystem/fret-bootstrap` exists as the ecosystem "golden path" startup layer (wrapper over `fret-launch`).
- `apps/fretboard` exists as a dev-tools binary for running native/web demos with consistent flags.
- `fretboard init todo` provides a starter template for new apps (see `docs/examples/todo-app-golden-path.md`).

In progress / next:

- Migrate remaining demos to the bootstrap "golden path" (optional but recommended).
- Keep `fret-bootstrap` and `fret-ui-assets` small and composable; avoid re-introducing a mixed "app kit" crate.

### 5) Layering rules (hard)

- `crates/*` must not depend on `ecosystem/*`.
- `ecosystem/*` may depend on `crates/*` but should avoid backend crates (unless the ecosystem crate is explicitly a runner-oriented helper like `fret-bootstrap`).
- `fretboard` (CLI) may depend on `fret-bootstrap` but not vice versa.
- `fretboard` is allowed to shell out to platform-specific toolchains (e.g. `trunk`) because it is tooling, not a library.

## Suggested API Surfaces (Non-binding)

This ADR does not lock exact APIs, but recommends a minimal set of patterns to keep bootstrapping consistent:

- `fret-bootstrap` provides:
  - `BootstrapBuilder` wrapping `fret_launch::WinitAppBuilder`,
  - `.with_settings_file(path)`,
  - `.with_icon_packs(...)`,
  - `.with_theme_preset(...)`,
  - `.with_ui_assets_budget(...)`,
  - `.enable_hotpatch_file_trigger_env(...)` (feature-gated; file-based polling marker),
  - `.enable_hotpatch_subsecond_devserver_env(...)` (feature-gated; devserver websocket + Subsecond JumpTable),
  - `.enable_hotpatch_subsecond_devserver_env_with_build_id(...)` (feature-gated; avoids cross-process patch confusion).

- `fret-ui-assets` provides:
  - unified `UiAssets::handle_event(app, window, event)` (drives `ImageAssetCache`, etc.),
  - `stats()` for overlays.

### Web (wasm32) workflow expectation

- Default workflow: devserver rebuild + reload (not Subsecond) (ADR 0105).
- Recommended (tooling-layer) integration:
  - `fretboard dev web` runs a chosen `apps/*` wasm target via `trunk serve` (or equivalent),
  - `fretboard` is responsible for flags, feature selection, and environment variable plumbing,
  - library crates do not depend on `trunk` or other external toolchain components.

## Alternatives Considered

### A) Put bootstrap inside `crates/fret-launch`

Rejected.

`fret-launch` is glue and already a dependency for apps; baking opinionated defaults and UI-kit concerns into it would
increase churn and make it harder to keep the glue stable and portable.

### B) Keep `fret-asset-cache` name and do nothing

Partially acceptable short-term, but rejected as the long-term direction.

The name strongly suggests an editor/project asset pipeline, which is explicitly out-of-scope for the framework kernel.
Clear naming improves cognition and reduces downstream architectural confusion.

### C) Provide a single "everything crate" for user ergonomics

Rejected.

This tends to collapse layering boundaries, increases compile times, and makes it hard to evolve defaults without
breaking stable contracts.

## Consequences

### Benefits

- Users get a clear golden path without kernel pollution.
- "Assets" are clarified as "UI render assets", reducing scope confusion with editor project asset pipelines.
- Dev workflows become a separate layer (CLI/tooling) that can evolve rapidly without impacting library contracts.

### Costs

- Additional crates and naming decisions increase up-front documentation work.
- Migration from existing call sites may require re-exports and deprecations.

## Migration Plan

1) Add `ecosystem/fret-bootstrap` and update demos to use it (optional but recommended).
2) Introduce `fret-ui-assets` as a re-export wrapper around `fret-asset-cache` (or rename directly if early enough).
3) Add `fretboard` CLI once the bootstrap patterns stabilize.
4) Add a starter template command (`fretboard init`) once the crate boundaries prove stable. (Implemented: `fretboard init todo`.)

## References

- Crate structure: `docs/adr/0092-crate-structure-core-backends-apps.md`
- Framework scope boundary: `docs/adr/0027-framework-scope-and-responsibilities.md`
- Resource handles & ownership: `docs/adr/0004-resource-handles.md`
- Workspace boundaries / components repo direction: `docs/adr/0037-workspace-boundaries-and-components-repository.md`
- Dev hotpatch integration (this repo): `docs/adr/0105-dev-hotpatch-subsecond-and-hot-reload-safety.md`
