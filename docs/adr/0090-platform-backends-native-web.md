# ADR 0090: Platform Backends (native + web)

Status: Accepted

## Context

We want `fret` to be a cross-platform UI framework (desktop now, wasm/web and mobile later).
The early repository split tied "desktop platform I/O" to `winit` in the documentation, but the
code evolved differently:

- `winit` is primarily an event loop + window/input abstraction (and also supports wasm targets).
- Platform I/O concerns (clipboard, file dialogs, open-url, external drop payload reading) are
  orthogonal to `winit`.

To avoid repeated "where does platform I/O live?" confusion and to keep runners thin, we need a
stable crate boundary that is consistent across native and web targets.

## Decision

We split platform concerns into:

- `crates/fret-platform`: portable contracts and shared types for platform I/O (backend-agnostic).
- `crates/fret-platform-native`: native (non-wasm) implementations of `fret-platform` contracts.
  This is the default backend for Windows/macOS/Linux builds.
- `crates/fret-platform-web`: wasm32/web implementations for platform services that require browser
  APIs (timers, file dialog reads, clipboard, etc).
- `crates/fret-runner-winit`: winit-specific glue (event mapping + optional Accessibility / AccessKit adapter).
  It is not the primary home for general platform I/O contracts.

Runners remain responsible for event-loop ownership and presentation:

- `crates/fret-runner-winit`: maps `winit` events into `fret-core` input/events.
  - Also owns small, `winit`-specific per-window state (IME cursor area, cursor icon) and exposes a
    `prepare_frame` flush point so runners can apply pending window-side updates deterministically
    (ImGui-style backend split).
  - Owns native `WindowId` -> `AppWindowId` bookkeeping for winit-backed runners.
- `crates/fret-launch`: owns the native event loop + surfaces + renderer glue, and uses
  `fret-platform-native` to execute platform effects.
- Web targets use `fret-platform-web` for browser APIs. Input/event translation may be implemented either via
  `winit` (wasm) or directly via browser DOM events (`web-sys`); the default direction is a dedicated web adapter
  crate (`fret-runner-web`) for maximum IME and keyboard fidelity.

Compatibility shims may exist temporarily to reduce churn:

- `crates/fret-runner-web` re-exports `fret-platform-web`.

## Invariants

- Portable crates (`fret-core`, `fret-runtime`, `fret-app`, `fret-ui`, component crates) must not
  depend on backend crates (`fret-platform-*`, `fret-render`, `fret-runner-*`).
- Backend crates must not depend on UI/component crates.

## Consequences

- Crate responsibilities match the cross-platform story (native vs web), reducing "docs-code drift".
- Adding iOS/Android becomes a matter of adding `fret-platform-ios` / `fret-platform-android`
  without redefining the runner boundary.

## Future Work

- Consider a `fret-platform-default` facade that selects the right backend via `cfg(...)`.
- Keep `fret-runner-winit` responsibilities limited to concerns that are truly `winit`-specific
  (e.g. window a11y bridge), not general OS/browser I/O.
- Re-enable the real AccessKit adapter in `fret-runner-winit` once `winit 0.31` is stable and `accesskit_winit` has a compatible release.
