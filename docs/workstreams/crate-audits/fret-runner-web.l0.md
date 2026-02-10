# Crate audit (L0) — `fret-runner-web`

## Crate

- Name: `fret-runner-web`
- Path: `crates/fret-runner-web`
- Owners / adjacent crates: `fret-platform-web` (services), `fret-core` (events), `fret-ui`/`fret-runtime` (integration via runners)
- Current “layer”: web runner glue (DOM-adjacent adapters, input mapping)

## 1) Purpose (what this crate *is*)

- A wasm32-only runner glue crate for Fret:
  - re-exports `fret-platform-web` services used by runtime effects
  - provides DOM-adjacent adapters (cursor listener, input event mapping, RAF/timers)
- Keeps non-wasm builds explicit via a stub `RunnerError`.

Evidence anchors:

- `crates/fret-runner-web/src/lib.rs`
- `crates/fret-runner-web/src/native.rs`
- `crates/fret-runner-web/src/events.rs`

## 2) Public contract surface

- wasm32:
  - `WebInputState`, `WebPointerEventKind`, `map_keyboard_event`
  - cursor helpers: `install_canvas_cursor_listener`, `last_cursor_offset_px`, etc.
  - RAF helpers: `request_animation_frame`, `set_timeout_ms`, etc.
  - re-export of `fret-platform-web` services
- non-wasm:
  - `RunnerError` (actionable error message).

Evidence anchors:

- `crates/fret-runner-web/src/lib.rs`
- `crates/fret-runner-web/src/native.rs`

## 3) Dependency posture

- wasm32-only deps: `web-sys`, `wasm-bindgen` + workspace deps `fret-core`, `fret-platform-web`.
- Layering risk: keep wasm deps cfg-gated; do not leak DOM concepts into portable crates.

Evidence anchors:

- `crates/fret-runner-web/Cargo.toml`
- `pwsh -NoProfile -File tools/check_layering.ps1`

## 4) Module ownership map (internal seams)

- Cursor listener + canvas targeting
  - Files: `crates/fret-runner-web/src/cursor.rs`
- Input mapping (PointerEvent/KeyboardEvent/WheelEvent → `fret-core::Event`)
  - Files: `crates/fret-runner-web/src/events.rs`
- RAF/timers helpers
  - Files: `crates/fret-runner-web/src/raf.rs`
- Non-wasm stub
  - Files: `crates/fret-runner-web/src/native.rs`

## 5) Refactor hazards (what can regress easily)

- Click counting and pointer mapping semantics
  - Failure mode: multi-click count drift, wrong pointer id mapping (negative ids), slop/delay behavior drift.
  - Existing gates: none in this crate on non-wasm; recommend wasm-only test coverage once infra is available.
- Modifier mapping and keyboard code parsing
  - Failure mode: incorrect Modifiers (AltGraph), wrong key code mapping, repeat flag drift.
  - Existing gates: none in this crate today.
- Coordinate space assumptions for offsetX/Y
  - Failure mode: wrong coordinate origin when canvas is nested/scaled; mismatched css-px vs device-px.
  - Existing gates: needs integration/diag in a web demo.

Evidence anchors:

- `crates/fret-runner-web/src/events.rs`

## 6) Code quality findings (Rust best practices)

- `cfg(target_arch = "wasm32")` gating is clear, keeping native builds light.
- Recommendation: add a “compile wasm32” gate in CI for this crate and introduce fixture-driven mapping tests in a wasm test environment.

Evidence anchors:

- `crates/fret-runner-web/src/lib.rs`

## 7) Recommended refactor steps (small, gated)

1. Add a CI gate to compile this crate for `wasm32-unknown-unknown` — outcome: prevent accidental breakage in wasm-only modules — gate: `cargo check -p fret-runner-web --target wasm32-unknown-unknown`.
2. Add fixture-driven tests for keyboard code parsing and modifier mapping — outcome: stabilize input contract across browsers — gate: wasm test runner (TBD).
3. Add at least one `fretboard diag` scenario in a web demo for pointer/multi-click behavior — outcome: catch end-to-end regressions — gate: `fretboard diag` (web).

## 8) Open questions / decisions needed

- Should click slop and multi-click delay be a stable contract (documented and tested), or can it remain a runner implementation detail?
- Where should browser differences (e.g. WheelEvent deltaMode) be normalized: runner-web vs core input normalization?

