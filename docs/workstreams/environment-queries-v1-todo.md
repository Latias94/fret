---
title: Environment Queries (v1) — TODO
status: draft
date: 2026-02-09
---

# Environment Queries (v1) — TODO

Workstream entry:

- `docs/workstreams/environment-queries-v1.md`

ADR anchor:

- `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`

## Contract / docs

- [x] Land ADR 0232 (Accepted).
- [x] Add ADR 0232 to `docs/adr/README.md` task jump table near ADR 0231.
- [x] Add a `docs/todo-tracker.md` entry for environment queries (viewport/device).

## Runtime mechanism (`crates/fret-ui`)

- [x] Define a per-window committed environment snapshot storage surface.
- [x] Expose a typed query API that records dependencies during declarative rendering.
- [x] Dependency tracking: record which view roots observed which environment keys.
- [x] Invalidation: environment changes invalidate dependents (coalescing OK).
- [x] Add diagnostics hooks (inspector snapshot / debug logging).
  - Evidence: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (exports `debug.environment` and
    `debug.element_runtime.observed_environment` with deps fingerprints).
  - Evidence: `crates/fret-ui/src/elements/runtime.rs` (`EnvironmentQueryDiagnosticsSnapshot`)
  - Evidence: `crates/fret-ui/src/elements/mod.rs` (re-exported diagnostics types)
- [ ] Add unit tests for:
  - [x] invalidation on viewport bounds change,
  - [x] view-cache key participation via deps fingerprint,
  - [x] revision tracking for pointer capability keys,
  - [x] color scheme revision + invalidation,
  - [x] contrast preference revision + invalidation,
  - [x] forced-colors mode revision + invalidation,
  - [x] safe-area insets revision + invalidation,
  - [x] occlusion insets revision + invalidation.

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

- [x] Web/wasm runner commits prefers-contrast (best-effort, via `prefers-contrast` media queries)
  into the per-window environment snapshot seam (`None` when unsupported).
  - Evidence: `crates/fret-launch/src/runner/web/render_loop.rs`
  - Evidence: `crates/fret-ui/src/elements/cx.rs` (commits from `WindowMetricsService`)
  - Evidence: `crates/fret-core/src/window.rs` (`WindowMetricsService` stores optional preferences)

- [x] Web/wasm runner commits forced-colors mode (`window.matchMedia("(forced-colors: active)")`)
  into the per-window environment snapshot seam (`None` when unsupported).
  - Evidence: `crates/fret-launch/src/runner/web/render_loop.rs`
  - Evidence: `crates/fret-ui/src/elements/cx.rs` (commits from `WindowMetricsService`)
  - Evidence: `crates/fret-core/src/window.rs` (`WindowMetricsService` stores optional preferences)

- [x] Desktop runner commits best-effort accessibility + appearance preferences:
  - prefers-color-scheme (light/dark)
  - prefers-reduced-motion (best-effort; `None` when unavailable)
  - prefers-contrast (best-effort; `None` when unavailable)
  - forced-colors mode (best-effort; `None` when unavailable)
  - Linux: reads preferences from the xdg-desktop-portal Settings API (best-effort; falls back to
    `winit::window::Window::theme()` when available).
  - Linux: listens to xdg-desktop-portal `SettingChanged` signals and wakes the event loop to
    refresh the committed snapshot (reduces polling latency; still keeps a 500ms fallback poll).
  - Evidence: `crates/fret-launch/src/runner/desktop/mod.rs`
  - Evidence: `crates/fret-launch/src/runner/desktop/app_handler.rs` (polling + ThemeChanged refresh)

- [x] Web/wasm runner uses `MediaQueryList` change listeners for environment preferences, writing to
  `WindowMetricsService` only when the relevant media queries change (safe-area / occlusion remain
  per-frame reads).
  - Evidence: `crates/fret-launch/src/runner/web/render_loop.rs`

## Policy helpers (`ecosystem/fret-ui-kit`)

- [x] Add environment query helper surface:
  - [x] viewport breakpoint tokens (Tailwind-aligned labels, optional),
  - [x] pointer capability gates (hover vs touch-first),
  - [x] reduced-motion preference helpers (if available),
  - [x] color-scheme preference helpers (if available),
  - [x] contrast preference helpers (if available),
  - [x] forced-colors mode helpers (if available),
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

- [x] (Optional policy) Allow shadcn theme to follow the environment color scheme (light/dark) at
  app/runner integration boundaries, without pushing theme policy into the runtime.
  - Evidence: `ecosystem/fret-ui-shadcn/src/app_integration.rs` (`sync_theme_from_environment`)
  - Evidence: `ecosystem/fret-bootstrap/src/ui_app_driver.rs` (`on_global_changes_middleware`)
  - Evidence: `ecosystem/fret-kit/src/lib.rs` (golden-path wiring)

## Sweep / hygiene

- [x] Avoid ad-hoc viewport reads (`cx.bounds`) in overlay outer-bounds helpers: derive outer bounds
  from the committed environment snapshot so view-cache keys/invalidation stay correct under reuse.
  - Evidence: `ecosystem/fret-ui-kit/src/overlay.rs` (`outer_bounds_with_window_margin_for_environment`)
  - Evidence: `ecosystem/fret-ui-shadcn/src/*` overlay placements now call the environment helper.
