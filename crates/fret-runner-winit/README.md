# `fret-runner-winit`

`fret-runner-winit` is the winit-based runner glue for Fret.

It is responsible for translating winit window/input events into `fret-core` events, and for
tracking small pieces of window-side state (cursor/IME) that must be applied at the platform
boundary.

This crate is intentionally “runner glue”:

- It should not own UI policy (belongs in ecosystem).
- It should not own GPU rendering (belongs in `fret-render` + runner integration).

## Module ownership map

- `src/lib.rs`: public facade re-exports (stable surface for other crates).
- `src/mapping/`: winit → `fret-core` mapping helpers (cursor/keyboard/modifiers/pointer/wheel/position).
- `src/state/`: small runner-side state containers (`WinitPlatform`, `WinitInputState`, `WinitWindowState`).
- `src/window_registry.rs`: window bookkeeping helpers.
- `src/accessibility.rs`: accessibility integration entrypoints.
- `src/accessibility_accesskit_winit.rs`: AccessKit + winit specifics.
- `src/windows_ime.rs`: Windows IME bridge helpers (`cfg(windows)`).
- `src/state/input/click_tracker_tests.rs`: click tracking regression tests (`cfg(test)`).

## Refactor gates

- Formatting: `cargo fmt`
- Tests: `cargo nextest run -p fret-runner-winit`
