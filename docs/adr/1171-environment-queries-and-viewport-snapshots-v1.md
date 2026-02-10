# ADR 1171: Environment Queries and Viewport Snapshots (v1)

Status: Accepted

## Context

Fret’s container query contract (ADR 1170) addresses **panel/container-driven** responsiveness for
editor-grade UIs (docking splits, inspector panes, resizable panels). However, upstream recipes
also rely on **environment-driven** responsiveness and capability gating:

- Viewport width / height (“mobile vs desktop” shells; drawer vs popover patterns).
- Pointer capabilities (coarse vs fine; hover availability; touch-first affordances).
- Safe-area insets (future mobile targets).
- User preferences (reduced motion) and platform limitations (single-window platforms, etc.).

Today, these decisions tend to leak into component code as ad-hoc reads of `cx.bounds` (viewport
rect) and magic numbers (e.g. `>=768px` for `md`). This has several problems:

- There is no stable, typed mechanism to **observe** environment values and participate in view
  caching / invalidation decisions.
- Diagnostics bundles cannot easily answer “which environment value caused this branch?” without
  scraping logs.
- The boundary between *container queries* and *viewport/device queries* is easy to blur, which
  risks migrating the wrong behaviors when doing ecosystem sweeps.

We want a contract that keeps the runtime mechanism-only (ADR 0066), remains deterministic, and is
portable across native + wasm runners, while leaving policy tables and thresholds to ecosystem
crates.

## Decision

### D1 — Define a first-class “environment query” mechanism

The runtime provides an **environment query** mechanism that allows declarative elements to read
a **committed** per-window environment snapshot (or selected fields from it) during rendering.

The snapshot is defined as:

- Per-window (keyed by `AppWindowId`).
- Updated by the runner at frame boundaries (or earlier) and treated as stable for the duration of
  a frame.
- Independent from the UI layout solve (so reading it cannot cause layout recursion).

This is a mechanism-only contract. It does not define breakpoint tables or “responsive variants”.

### D2 — Separate axes: container queries vs environment queries

We explicitly separate:

- **Container queries** (ADR 1170): “adapt to the width/height of a local container/panel”.
- **Environment queries** (this ADR): “adapt to window/viewport/device capabilities and user
  preferences”.

Rule of thumb:

- If the correct behavior must follow **panel width**, it must use a container query.
- If the correct behavior must follow **device/viewport** or **input capability**, it must use an
  environment query.

Example:

- `Combobox(responsive)` choosing `Drawer` vs `Popover` is device/viewport-driven and should be
  expressed as an environment query (not a container query), because the behavior is about the
  *interaction shell*, not the local panel width.

### D3 — Environment queries participate in dependency tracking and view-cache keys

Observing an environment value is treated like observing a `Model<T>`:

- Views/elements that read environment values record dependencies.
- When relevant environment fields change, dependents are invalidated according to the requested
  invalidation level (Layout/Paint/HitTest).

For view caching, environment dependencies must be representable as a stable fingerprint so cache
keys can incorporate them (similar to ADR 1170’s layout-query deps fingerprint).

### D4 — Diagnostics requirements (v1)

The runtime must be able to report, in diagnostics snapshots/bundles (best-effort is acceptable in
v1):

- The committed environment snapshot (or a summarized subset).
- Which view roots/elements observed which environment fields (and the invalidation level).
- A stable “deps fingerprint” for each observer, suitable for debugging cache-key mismatches.

### D5 — Non-goals (v1)

This ADR explicitly does **not** introduce:

- A CSS media query parser (`(min-width: ...) and (pointer: coarse)`).
- A style cascade system.
- A single global policy table for breakpoints (policy stays in ecosystem crates).
- Mandatory same-frame updates (frame-lag is allowed if it simplifies determinism and caching).

## Proposed Snapshot Shape (non-normative)

This section is illustrative; the exact schema may evolve without changing the contract’s intent.

Per-window snapshot fields:

- `viewport_bounds_logical: Rect` (or width/height only).
- `scale_factor: f32` (DPI).
- `pointer: { primary_kind, coarse, hover }` (capability summary).
- `safe_area_insets_logical: Option<Edges<Px>>` (future mobile).
- `prefers_reduced_motion: Option<bool>` (runner-provided or app-provided).

## v1 Implementation Status (as of 2026-02-09)

The current implementation provides a small set of typed environment query keys via
`fret-ui::ElementContext`:

- `viewport_bounds_logical: Rect` (key: `ViewportSize`)
- `scale_factor: f32` (key: `ScaleFactor`)
- `prefers_reduced_motion: Option<bool>` (key: `PrefersReducedMotion`)
  - Best-effort: on web/wasm, the runner commits this preference via
    `window.matchMedia("(prefers-reduced-motion: reduce)")` when supported. On native desktop, this
    is currently `None` unless committed by a runner/app integration.
- `primary_pointer_type: PointerType` (key: `PrimaryPointerType`)
  - Best-effort: `PointerType::Unknown` is returned until a pointer event is observed for the
    window (native and wasm).
- `safe_area_insets_logical: Option<Edges>` (key: `SafeAreaInsets`)
  - Best-effort: on web/wasm, the runner commits safe-area insets derived from CSS
    `env(safe-area-inset-*)` (when supported). On native desktop, this is currently `None` unless a
    runner commits safe-area insets for the window (future mobile targets).
- `occlusion_insets_logical: Option<Edges>` (key: `OcclusionInsets`)
  - Best-effort: on web/wasm, the runner commits viewport occlusion insets derived from
    `window.visualViewport` (e.g. virtual keyboard / browser UI changes). On native desktop, this is
    currently `None` unless a runner commits viewport occlusion insets (future mobile targets).

Policy remains in ecosystem crates. `ecosystem/fret-ui-kit` exposes small helpers that derive
`primary_pointer_can_hover` and `primary_pointer_is_coarse` from `primary_pointer_type` so recipes
can gate hover-only affordances (tooltips / hover cards) without embedding policy in the runtime.

## Consequences

- We can migrate ad-hoc viewport breakpoints (magic numbers) to a single, typed query surface,
  while keeping container-query-driven responsiveness separate and correct for docking/panels.
- Diagnostics bundles can capture environment-caused branches without relying on logs.
- Future mobile support has a stable seam for safe area and coarse-pointer behavior without
  rewriting recipe code.

## Implementation Notes (non-normative)

Likely layering:

- `crates/fret-core`: define portable capability vocabulary (pointer kinds, safe-area types).
- `crates/fret-ui`: store per-window committed snapshot + dependency tracking + diagnostics export.
- `ecosystem/fret-ui-kit`: policy helpers (breakpoint tokens, “mobile shell” gates, reduced-motion
  defaults, etc.).
- `ecosystem/fret-ui-shadcn`: recipes use helpers instead of `cx.bounds` magic numbers for
  viewport/device-driven variants.

Workstream tracking:

- `docs/workstreams/environment-queries-v1.md`
- `docs/workstreams/environment-queries-v1-milestones.md`
- `docs/workstreams/environment-queries-v1-todo.md`
