# ADR 0162: fret-canvas Modules and Feature Strategy (Portable Canvas Substrate)

Status: Proposed
Date: 2026-01-16
Scope: `ecosystem/fret-canvas` module boundaries + feature-gated integration surfaces.
Related:
- ADR 0128 (Canvas widgets and interactive surfaces)
- ADR 0141 (Declarative Canvas element and painter)
- ADR 0144 (Canvas pan/zoom input mapping v1)
- ADR 0152 (Kurbo geometry backend for canvas hit testing)
- ADR 0161 (Canvas cache policy and hosted resource caches)

## Context

Fret has multiple “editor-grade” interactive surfaces implemented as retained widgets in the ecosystem:

- node graphs (`ecosystem/fret-node`),
- plots (`ecosystem/fret-plot`),
- charts (`ecosystem/fret-chart`),
- and future large surfaces (grids, timelines, canvases with overlays, etc.).

These surfaces repeatedly need the same infrastructure building blocks:

- view transforms (pan/zoom) and stable coordinate terminology,
- hit-testing helpers and optional geometry backends,
- spatial indices for large scenes,
- prepared resource lifecycle and caches (`PathId`, `TextBlobId`, `SvgId`),
- wiring helpers to integrate with `crates/fret-ui` when desired.

Historically, retained canvases have implemented ad-hoc copies of these utilities and caches, which
creates drift, inconsistent performance tuning, and redundant code.

At the same time, Fret’s core design philosophy is to keep interaction policy out of the framework
layer (ADR 0128): the runtime provides mechanisms and safe defaults; ecosystem/app crates own domain
models, gesture maps, tool arbitration, snapping rules, and UX policy.

We need an explicit contract for what `ecosystem/fret-canvas` is (and is not), including how it is
split into modules and how optional dependencies are feature-gated.

## Goals

1. Define a clear, portable, policy-light “canvas substrate” crate for ecosystem canvases.
2. Establish module boundaries that prevent future drift and duplication.
3. Keep default dependencies minimal while allowing optional backends via features.
4. Support both retained and declarative authoring:
   - retained widgets can own their caches and state,
   - declarative `Canvas` remains a runtime mechanism (ADR 0141).
5. Provide stable lifecycle shapes for hosted-resource caches (`begin_frame`/`prepare`/`prune`/`clear`).

## Non-goals

- A single universal data model for all canvases (node graphs, charts, editors).
- Encoding input mapping policy, tool modes, or domain rules in `fret-canvas`.
- Making `fret-ui-kit` depend on `fret-canvas` (to avoid “heavy by default” UI kits).
- Perfect GPU memory accounting in v1 (entry-count budgets are acceptable initially).

## Decision

### 1) `fret-canvas` is a portable, policy-light ecosystem substrate

`ecosystem/fret-canvas` provides reusable infrastructure for “canvas-like” retained widgets:

- portable math/state helpers,
- reusable caches for renderer-hosted resources,
- optional backends for spatial/geometry operations.

It must remain portable:

- no platform-specific dependencies,
- no renderer-backend dependencies (wgpu/webgpu/winit are out of scope),
- only uses `fret-core` primitives and `UiServices` handles when needed.

### 2) Module taxonomy (public surface)

`fret-canvas` is organized into the following public modules:

- `cache`: retained-friendly hosted-resource caches (ADR 0161 alignment)
  - `TextCache` (existing)
  - `PathCache`
  - `SvgCache`
- `scale`: pixel/DPI helpers (e.g. “constant pixel stroke”)
- `view`: view transforms and pan/zoom state helpers
- `drag`: drag-phase helpers (state machines, thresholds; no tool policy)
- `spatial`: lightweight default spatial indexing
- `wires`: reusable “wire routing / hit testing” helpers for node-graph-like surfaces

Guidance:

- Modules must stay mechanism-oriented and reusable across domains.
- Domain policy stays in consumer crates (`fret-node`, `fret-plot`, `fret-chart`, apps).
- “Recipes” (opinionated defaults) are allowed only behind feature-gated integration modules.

### 3) Feature strategy (single crate, opt-in weight)

`fret-canvas` uses a single-crate, feature-gated model:

- Default features: none (portable substrate only).
- `ui`: UI integration helpers (depends on `fret-ui` + `fret-runtime`)
  - declarative wiring helpers, adapter types, and small recipes where appropriate
  - must not pull in policy-heavy ecosystems by default
- `kurbo`: optional geometry backend for hit-testing/path operations (ADR 0152)
- `rstar`: optional spatial index backend
- `declarative`: reserved for a future declarative canvas element surface (ADR 0141/0128 direction)

Locked naming:

- The feature that pulls in `fret-ui` integration is named `ui`.

### 4) Retained cache lifecycle contract (v1)

All retained caches in `fret-canvas::cache` follow the same lifecycle shape:

- `begin_frame()`: advances internal frame counters for age-based eviction.
- `prepare(...) -> (id, metadata)`: prepares and caches hosted resources using stable keys.
- `prune(max_age_frames, max_entries)`: evicts by age and budget (LRU by `last_used_frame`).
- `clear(services)`: releases all hosted resources deterministically.

Optional helper:

- `get(key, constraints) -> Option<(id, metadata)>`: fetches without preparing (used for “prepare in rebuild, render in paint” patterns).

Key hygiene requirements:

- Call sites must provide stable keys derived from domain identity (e.g. mark/series ids) plus a
  variant discriminator (stroke vs fill, segment index, etc.).
- Call sites must include effective resolution intent via `PathConstraints.scale_factor`:
  `scale_factor = dpi * zoom` (the cache namespaces entries by normalized scale factor).

### 5) Layering guidance: declarative vs retained caches

- Declarative `Canvas` in `crates/fret-ui` hosts resources and applies a smooth-by-default cache
  policy vocabulary (ADR 0161).
- Retained widgets should prefer reusing `fret-canvas::cache` rather than embedding ad-hoc caches.
- We do not require the runtime and ecosystem caches to share an implementation; they should share
  lifecycle semantics and tuning vocabulary.

## Consequences

Pros:

- Less duplication across retained canvases; shared optimizations benefit multiple crates.
- Clear “portable by default” story, with opt-in backends/integration.
- Easier cache tuning and migration: consistent lifecycle and knobs.

Cons / Risks:

- If boundaries are unclear, `fret-canvas` can become a “misc utils” bucket.
- Poor key hygiene at call sites can still cause churn (many unique keys).
- Feature creep in `ui` integration could accidentally pull policy or heavy deps.

## Migration Plan (Recommended)

1. Converge retained path/svg caches onto `fret-canvas::cache::{PathCache,SvgCache}`.
2. Add cache observability counters (hits/misses/evictions/prepares) and wire into diagnostics.
3. Optionally introduce a small typed key builder helper to reduce ad-hoc hashing at call sites.

## Open Questions

1. Do we want a shared “cache policy preset” type in `fret-canvas` (e.g. `SmoothDefault`) that can
   be reused by retained widgets, or should all tuning remain numeric at call sites?
2. Should `declarative` stay as a placeholder feature, or be removed until the surface exists?
3. When/if byte-size estimates become available, should caches add optional byte budgets?

## References

- Module entry point: `ecosystem/fret-canvas/src/lib.rs`
- Feature flags: `ecosystem/fret-canvas/Cargo.toml`
- Declarative canvas contract: `docs/adr/0141-declarative-canvas-element-and-painter.md`
- Cache policy vocabulary: `docs/adr/0161-canvas-cache-policy-and-hosted-resource-caches.md`
