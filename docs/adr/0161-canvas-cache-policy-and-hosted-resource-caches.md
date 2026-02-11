# ADR 0161: Canvas Cache Policy and Hosted Resource Caches (Smooth-By-Default)

- Status: Proposed
- Date: 2026-01-16
- Scope: `crates/fret-ui` Canvas hosted caches + ecosystem canvas substrate (`ecosystem/fret-canvas`)
- Related:
  - ADR 0004 (Resource handles)
  - ADR 0055 (Cached subtree / replay constraints)
  - ADR 0128 (Canvas widgets and interactive surfaces)
  - ADR 0141 (Declarative Canvas element and painter)
  - ADR 0144 (Canvas pan/zoom input mapping v1)
  - ADR 0152 (Kurbo geometry backend for canvas hit testing)

## Context

Fret supports multiple “canvas-like” interactive surfaces:

- retained editor canvases (node graphs, plots, charts),
- declarative custom draw via the `Canvas` leaf element (ADR 0141),
- canvas-backed virtualization/large surfaces (e.g. data-grid-like canvases).

Many of these surfaces rely on renderer-owned, explicitly released resources:

- `TextBlobId`
- `PathId`
- `SvgId`

Today, caching behavior is inconsistent across call sites:

- Declarative `CanvasPainter` caches hosted resources, but eviction is mostly “drop if not used this
  paint pass” for element-local keys (while shared text has a bounded retention window).
- Retained canvases often implement their own ad-hoc caches (text/path), budgets, and pruning.

This causes long-term drift and makes it hard to tune “smoothness vs memory” consistently across:

- scroll-driven redraw (rapidly changing visible set),
- pan/zoom redraw (stable visible set, but high FPS),
- large editor canvases with long-lived sessions.

We want a single, explicit caching policy vocabulary that:

1) is safe by default (bounded, deterministic cleanup),
2) is smooth by default (reduces prepare/release thrash),
3) remains policy-free at the runtime layer (no domain-specific rules),
4) can be tuned per-canvas by ecosystem/app code.

## Goals

1. Reduce resource prepare/release thrash for canvas-like surfaces.
2. Provide a stable cache policy contract usable by both:
   - declarative Canvas hosted caches (runtime-owned), and
   - retained widget caches (widget-owned, ecosystem).
3. Keep the framework vs ecosystem boundary aligned with ADR 0128/0141:
   - runtime provides mechanisms and safe defaults,
   - ecosystem/app provides interaction policy and domain models.
4. Make cache behavior observable and adjustable without forking per-widget implementations.

## Non-goals

- Unifying all canvases behind a single “Canvas widget” data model.
- Moving gesture maps / tool modes / domain heuristics into `crates/fret-ui`.
- Introducing a hard dependency on a specific renderer backend for cache policy.
- Perfect GPU-memory accounting in v1 (we start with bounded entry counts + time windows).

## Decision

### 1) Introduce a cache policy vocabulary

We introduce a small configuration vocabulary to express retention and budget:

- `CacheRetention`
  - `keep_frames: u64` (how long an unused entry may remain cached)
- `CacheBudget`
  - `max_entries: usize` (hard cap per resource kind)
- `CanvasCachePolicy`
  - `text: CachePolicyKind`
  - `path: CachePolicyKind`
  - `svg: CachePolicyKind`

Where `CachePolicyKind` is one of:

- `Transient`: evict quickly (best-effort, lowest memory).
- `SmoothDefault`: bounded retention intended to reduce thrash in common UIs.
- `Custom { keep_frames, max_entries }`: explicit tuning for editor-grade surfaces.

Locked details:

- All caches are bounded (by entry count at minimum).
- All cached resources must be released deterministically on teardown:
  - declarative: via the element removal cleanup hook (ADR 0141),
  - retained: via `Widget::cleanup_resources` (ADR 0004).

### 2) Runtime: Canvas hosted caches become smooth-by-default and configurable

`crates/fret-ui`’s hosted canvas caches (ADR 0141) adopt the cache policy vocabulary:

- The default policy is **SmoothDefault** for all hosted resource kinds.
- A canvas element may optionally override the policy (by numeric knobs only).

Eviction behavior:

- Each cache entry tracks `last_used_frame`.
- On each paint pass:
  - mark used entries,
  - update `last_used_frame`,
  - evict entries exceeding `keep_frames`,
  - enforce `max_entries` by LRU (oldest `last_used_frame`).

Notes:

- The runtime remains policy-light: it does not attempt to infer domain identity; callers still
  provide stable keys (ADR 0141).
- The runtime does not infer zoom from transforms; callers still supply `scale_factor = dpi * zoom`
  when higher-resolution preparation is desired (ADR 0141 / ADR 0144).

### 3) Ecosystem: Provide reusable retained caches in `ecosystem/fret-canvas`

`ecosystem/fret-canvas` is extended with a `cache` module:

- `TextCache` already exists and is retained-friendly (begin frame, prepare, prune, clear).
- Add `PathCache` and `SvgCache` with the same lifecycle shape:
  - `begin_frame()`
  - `prepare(...) -> (id, metrics/metadata)`
  - `prune(max_age_frames, max_entries)`
  - `clear(services)`

This enables retained canvases to converge on a shared cache implementation rather than embedding
one-off caches per widget.

Locked details:

- `ecosystem/fret-canvas` stays portable:
  - no platform deps,
  - no renderer/backend deps,
  - uses only `fret-core` primitives and `UiServices`.

### 4) Guidance: which layer owns what

- `crates/fret-ui`:
  - provides hosted caching mechanisms and safe defaults for declarative `Canvas`,
  - exposes only numeric cache policy knobs (no gesture policy).
- `ecosystem/fret-canvas`:
  - provides policy-light retained cache building blocks (`TextCache`, `PathCache`, `SvgCache`),
  - provides reusable canvas math/geometry/spatial helpers.
- `ecosystem/fret-canvas` UI integration (feature-gated) and higher layers:
  - may choose cache presets for specific canvas recipes (e.g. “EditorCanvasLarge”),
  - owns input mapping policy and tool arbitration (ADR 0144).

## Proposed Defaults (Non-normative)

We default to “smooth within bounds” rather than “evict immediately”:

- `SmoothDefault` (suggested starting point)
  - `keep_frames = 60` (≈ 1 second at 60 FPS)
  - `max_entries`:
    - `text: 4096`
    - `path: 2048`
    - `svg: 256`

Rationale:

- Reduces prepare/release thrash during scrolling and panning.
- Keeps worst-case memory bounded by entry count.
- Allows editor canvases to opt into larger budgets explicitly.

## Observability (Recommended)

To make cache tuning actionable, expose per-canvas counters (debug/diagnostics):

- prepares per frame / per second,
- cache hit rates by resource kind,
- current entry counts by kind,
- eviction counts by reason (age vs budget).

This should integrate with existing diagnostics tooling rather than introducing bespoke logging.

Implementation note:

- `ecosystem/fret-canvas` caches now expose lightweight counters via `CacheStats`
  (`ecosystem/fret-canvas/src/cache/mod.rs`) and per-cache `stats()` / `reset_stats()` methods.
  Wiring these into diagnostics bundles is recommended but remains ecosystem/app-owned.

## Consequences

Pros:

- More consistent and smoother behavior across ecosystem canvases.
- Reduced code duplication (retained caches converge in `fret-canvas`).
- Explicit tuning knobs for “editor-grade” canvases without hardcoding policy in the runtime.

Cons / Risks:

- Higher default memory usage vs “evict immediately”.
- Entry-count budgets are an approximation (no byte-accurate GPU accounting in v1).
- Requires careful key hygiene at call sites to avoid unbounded unique-key churn.

## Open Questions

1) Should policy also include an optional **byte budget** once renderer services can provide
   per-resource size estimates?
2) Should the runtime provide a stable “prepaint” hook (GPUI-style) for expensive precomputation,
   or should apps rely on model-derived caches and `begin_frame` patterns?
3) Should `SmoothDefault` vary by platform (web vs native) or capability tier (renderer budgets)?

## References

- Canvas guidance and layering: `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- Declarative Canvas element and hosted painter: `docs/adr/0141-declarative-canvas-element-and-painter.md`
- Canvas pan/zoom policy placement: `docs/adr/0144-canvas-pan-zoom-input-mapping-v1.md`
- Resource handle lifecycle: `docs/adr/0004-resource-handles.md`
- GPUI/Zed canvas reference: `repo-ref/zed/crates/gpui/src/elements/canvas.rs`
