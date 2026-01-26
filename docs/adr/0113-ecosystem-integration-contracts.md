# ADR 0113: Ecosystem Integration Contracts (Commands / Theme / Resources)

Status: Accepted

## Context

Fret intentionally separates:

- **kernel** contracts (`crates/*`, ADR 0093),
- **ecosystem** policy + components (`ecosystem/*`, ADR 0108),
- **tooling** (`apps/*`, e.g. `fretboard`).

The next growth phase is third-party ecosystem development: users may want to publish libraries such as:

- node graph editors,
- charting/plot libraries,
- markdown/rich text renderers,
- docking layouts / workspace shells,
- icon packs and asset helpers.

We already have examples inside this repository (`fret-plot`, `fret-plot3d`, `fret-markdown`, `fret-docking`,
`fret-ui-shadcn`). What’s missing is a shared, user-facing *integration model* that keeps composition smooth while
preserving kernel stability and Subsecond-friendly hot reload boundaries (ADR 0107).

This ADR is intentionally **non-binding** and may evolve as patterns stabilize.

Zed/GPUI code anchors (non-normative):

- command/action dispatch substrate: `repo-ref/zed/crates/gpui/src/action.rs`, `repo-ref/zed/crates/gpui/src/key_dispatch.rs`
- theme registry + schema: `repo-ref/zed/crates/theme`
- asset caching convenience: `repo-ref/zed/crates/gpui/src/asset_cache.rs`, `repo-ref/zed/crates/gpui/src/assets.rs`

## Goals

- Define a stable mental model for how ecosystem crates integrate with apps and with each other.
- Keep dependency selection obvious for third-party authors.
- Preserve portability (native + wasm) and avoid backend coupling in ecosystem crates.
- Clarify “assets/resources” boundaries (UI render assets vs project assets).
- Provide a hotpatch-aware guidance baseline (Subsecond-style patching, ADR 0107).

## Non-goals

- A strict published “ecosystem specification”. This ADR is guidance until the APIs settle.
- Replacing kernel contracts or changing ADR 0093 boundaries.
- Introducing an editor-grade project asset pipeline into the framework (ADR 0026 remains app-owned).

## Decision (Guidance)

### 1) Layering and dependencies (hard constraints)

The following rules are aligned with ADR 0093 / ADR 0108:

- `crates/*` must not depend on `ecosystem/*`.
- Ecosystem crates should avoid backend crates (`fret-launch`, `winit`, `wgpu`, `web-sys`), unless the ecosystem crate
  is explicitly runner/tooling oriented (e.g. `fret-bootstrap`).
- Third-party component libraries should depend on:
  - `fret-core` / `fret-runtime` for contracts and models,
  - `fret-ui` (and optionally `fret-ui-kit`) for rendering retained UI trees,
  - other ecosystem crates for policy/recipes (optional).

This ensures ecosystem libraries are portable and composable across native and wasm.

### 2) Prefer a “two-tier” crate shape for reusable ecosystems

For libraries that might be reused across multiple apps (node editors, advanced tables, chart engines), we recommend
a two-tier organization:

1) **Headless / engine tier** (optional):
   - pure data structures, algorithms, selection models, layout computation,
   - depends on `fret-core` / `fret-runtime` only (or even fewer).

2) **UI integration tier**:
   - renders into `fret-ui` retained elements and integrates with input/commands/themes,
   - depends on `fret-ui` (+ `fret-ui-kit` if it needs common primitives).

This mirrors the pattern used by many scalable UI stacks (including GPUI’s “core runtime + component surfaces”).

### 3) Standardize on the three-pipeline composition model

Ecosystem crates should integrate using the same three pipelines documented in ADR 0112:

- Event pipeline: `Event` → `UiTree::dispatch_event`
- Command pipeline: UI emits `CommandId` → app handles and mutates models
- Effect pipeline: app emits `Effect` → runner flushes → `Event` backflow

Ecosystem crates should avoid “hidden side channels” that bypass these pipelines, because:

- it makes debugging harder,
- it complicates hot reload reset boundaries,
- it tends to require backend knowledge.

### 4) Commands: namespacing + optional type wrappers

We keep `CommandId` string-based for flexibility (palette, scripting, plugins), but recommend:

- **namespacing**: `crate.scope.action` (e.g. `plot.zoom_in`, `node_graph.add_node`),
- **helpers** for parameterized commands:
  - `fn toggle_cmd(id: NodeId) -> CommandId`,
  - `fn parse_toggle(cmd: &CommandId) -> Option<NodeId>`.

Ecosystem crates may also provide an optional typed layer (macros/constants) to improve discoverability without
removing the dynamic `CommandId` surface.

### 5) Theme tokens: key-driven styling (no hard-coded palettes)

Ecosystem crates should:

- resolve colors/metrics via theme keys (e.g. `"plot.axis.fg"`, `"node_graph.node.radius"`),
- provide a documented list of keys and recommended defaults,
- avoid embedding a single palette as an invariant.

This keeps ecosystem crates composable across design systems (`fret-ui-shadcn`, custom themes, etc.).

### 6) Resources and “assets”: UI render assets only

We explicitly separate:

- **UI render assets** (ADR 0004): icons/images/SVGs registered via effects at flush points; UI holds stable IDs.
- **Project/editor assets** (ADR 0026): GUID identity, import pipeline, dependency graphs — app-owned.

Guidance for ecosystem crates:

- Icons: depend on `fret-icons` and register sources into `IconRegistry` (data-only).
- Images/SVGs: depend on `fret-ui-assets` (or a future `UiAssets` facade) to:
  - key/dedupe,
  - budget/evict,
  - drive state via `handle_event`.

Avoid creating bespoke “GPU resource” APIs in ecosystem crates; keep them portable via stable IDs and effects.

### 7) Async/background work: message passing into the UI thread

Ecosystem crates that need async work (markdown fetchers, asset downloading, indexing) should:

- treat `App`/`ModelStore` as main-thread only,
- perform background work off-thread or in an async runtime,
- communicate results via pure data messages into the UI thread (channel/inbox),
- apply model updates on the UI thread and request redraw.

Prefer the shared execution and wake surface (`docs/adr/0190-execution-and-concurrency-surface-v1.md`)
over bespoke thread pools or ad-hoc wake logic in ecosystem crates.

This matches ADR 0112’s recommended patterns and keeps wasm compatibility.

### 8) Hotpatch (Subsecond) compatibility: keep long-lived registries disposable

Ecosystem crates should assume that dev hotpatch may trigger a “hard reset” boundary (ADR 0107):

- action hook registries are disposable caches,
- overlay registries are disposable caches,
- long-lived callback surfaces should be ID/registry based rather than storing closures in retained nodes.

This does not forbid closures inside view construction; it discourages storing captured closures as long-lived state
that must survive across patches.

## Implementation Notes (Current Examples)

This repository already contains exemplars:

- Plotting: `ecosystem/fret-plot`, `ecosystem/fret-plot3d`
- Markdown: `ecosystem/fret-markdown`
- Docking policy: `ecosystem/fret-docking`
- Design system surface: `ecosystem/fret-ui-shadcn`
- UI render asset caches: `ecosystem/fret-ui-assets` (re-export wrapper over `fret-asset-cache`)

As these stabilize, we can tighten this ADR into a more prescriptive guide and/or extract a “third-party checklist”.

## References

- Kernel/backends/apps layering: `docs/adr/0093-crate-structure-core-backends-apps.md`
- Ecosystem bootstrap/tools story: `docs/adr/0108-ecosystem-bootstrap-ui-assets-and-dev-tools.md`
- Golden-path driver/pipelines: `docs/adr/0112-golden-path-ui-app-driver-and-pipelines.md`
- Execution and concurrency surface: `docs/adr/0190-execution-and-concurrency-surface-v1.md`
- Resource handles + flush point: `docs/adr/0004-resource-handles.md`
- Editor project assets (out of scope): `docs/adr/0026-asset-database-and-import-pipeline.md`
- Action hooks registries: `docs/adr/0074-component-owned-interaction-policy-and-runtime-action-hooks.md`
- Dev hotpatch boundaries: `docs/adr/0107-dev-hotpatch-subsecond-and-hot-reload-safety.md`
