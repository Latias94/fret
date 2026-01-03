# ADR 0091: Platform Backends (native + web)

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
- `crates/fret-platform-winit`: winit-specific glue (currently Accessibility / AccessKit). It is
  not the primary home for general platform I/O contracts.

Runners remain responsible for event-loop ownership and presentation:

- `crates/fret-runner-winit`: maps `winit` events into `fret-core` input/events.
- `crates/fret-runner-winit-wgpu`: owns the native event loop + surfaces + renderer, and uses
  `fret-platform-native` to execute platform effects.
- Web targets use `winit` on wasm for input/events and `fret-platform-web` for browser APIs.

Compatibility shims may exist temporarily to reduce churn:

- `crates/fret-platform-desktop` re-exports `fret-platform-native`.
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
- Extend `fret-platform-winit` responsibilities only for concerns that are truly `winit`-specific
  (e.g. window a11y bridge), not general OS/browser I/O.

