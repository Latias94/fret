---
title: Environment Queries (v1) — TODO
status: draft
date: 2026-02-09
---

# Environment Queries (v1) — TODO

Workstream entry:

- `docs/workstreams/environment-queries-v1.md`

ADR anchor:

- `docs/adr/1171-environment-queries-and-viewport-snapshots-v1.md`

## Contract / docs

- [x] Land ADR 1171 (Accepted).
- [x] Add ADR 1171 to `docs/adr/README.md` task jump table near ADR 1170.
- [x] Add a `docs/todo-tracker.md` entry for environment queries (viewport/device).

## Runtime mechanism (`crates/fret-ui`)

- [x] Define a per-window committed environment snapshot storage surface.
- [x] Expose a typed query API that records dependencies during declarative rendering.
- [x] Dependency tracking: record which view roots observed which environment keys.
- [x] Invalidation: environment changes invalidate dependents (coalescing OK).
- [x] Add diagnostics hooks (inspector snapshot / debug logging).
- [ ] Add unit tests for:
  - [x] invalidation on viewport bounds change,
  - [x] view-cache key participation via deps fingerprint,
  - [x] revision tracking for pointer capability keys,
  - [x] safe-area insets revision + invalidation,
  - [x] occlusion insets revision + invalidation,
  - [ ] stability under resize jitter (optional epsilon/hysteresis at policy layer).

## Runner plumbing (best-effort field sources)

- [x] Web/wasm runner commits safe-area insets (CSS `env(safe-area-inset-*)`) and viewport occlusion
  insets (`window.visualViewport`) into the per-window environment snapshot seam.

- [x] Web/wasm runner commits prefers-reduced-motion (`window.matchMedia("(prefers-reduced-motion: reduce)")`)
  into the per-window environment snapshot seam (best-effort; `None` when unsupported).
  - Evidence: `crates/fret-launch/src/runner/web/render_loop.rs`
  - Evidence: `crates/fret-ui/src/elements/cx.rs` (commits insets from `WindowMetricsService`)
  - Evidence: `crates/fret-core/src/window.rs` (`WindowMetricsService` stores optional insets + preferences)

- [x] Web/wasm runner commits prefers-color-scheme (`window.matchMedia("(prefers-color-scheme: dark)")`)
  into the per-window environment snapshot seam (best-effort; `None` when unsupported).
  - Evidence: `crates/fret-launch/src/runner/web/render_loop.rs`
  - Evidence: `crates/fret-ui/src/elements/cx.rs` (commits from `WindowMetricsService`)
  - Evidence: `crates/fret-core/src/window.rs` (`WindowMetricsService` stores optional preferences)

## Policy helpers (`ecosystem/fret-ui-kit`)

- [x] Add environment query helper surface:
  - [x] viewport breakpoint tokens (Tailwind-aligned labels, optional),
  - [x] pointer capability gates (hover vs touch-first),
  - [x] reduced-motion preference helpers (if available),
  - [x] safe-area insets helpers (future mobile).
  - [x] occlusion insets helpers (virtual keyboard / transient obstructions).
- [x] Add unit tests for hysteresis / non-oscillation where applicable.

## Ecosystem adoption (initial targets)

- [x] Migrate `Combobox(responsive)` to use environment query helpers instead of `cx.bounds` magic
  numbers for the mobile shell decision (Drawer vs Popover).
- [x] Add a regression gate (test or `fretboard diag` script) for the migration.
  - Evidence: `fret-ui-shadcn::web_vs_fret_overlay_placement::fret_combobox_responsive_drawer_blocks_underlay_scroll_on_mobile`
- [x] Migrate `SidebarProvider` “mobile/offcanvas shell” to infer from environment queries by
  default (override allowed).
  - Evidence: `fret-ui-shadcn::sidebar::tests::sidebar_provider_infers_mobile_from_viewport_width_when_unset`

- [x] Gate hover-driven affordances on pointer capability queries (touch-first should not open
  hover-only UI):
  - Evidence: `ecosystem/fret-ui-shadcn/src/tooltip.rs`
  - Evidence: `ecosystem/fret-ui-shadcn/src/hover_card.rs`

- [x] Apply safe-area + occlusion insets to “mobile shell” overlays (Drawer / Sheet) to avoid
  system UI + virtual keyboard overlap:
  - Evidence: `ecosystem/fret-ui-shadcn/src/sheet.rs`

## Sweep / hygiene

- [x] Avoid ad-hoc viewport reads (`cx.bounds`) in overlay outer-bounds helpers: derive outer bounds
  from the committed environment snapshot so view-cache keys/invalidation stay correct under reuse.
  - Evidence: `ecosystem/fret-ui-kit/src/overlay.rs` (`outer_bounds_with_window_margin_for_environment`)
  - Evidence: `ecosystem/fret-ui-shadcn/src/*` overlay placements now call the environment helper.
