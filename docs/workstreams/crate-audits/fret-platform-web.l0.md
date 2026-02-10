# Crate audit (L0) — `fret-platform-web`

## Crate

- Name: `fret-platform-web`
- Path: `crates/fret-platform-web`
- Owners / adjacent crates: `fret-runtime` (effects), `fret-core` (events/tokens), `fret-runner-web` (input/event mapping), web demo shells
- Current “layer”: web/wasm32 platform services (browser API integration)

## 1) Purpose (what this crate *is*)

- Browser API integrations used by `fret-runtime::Effect`s (timers, clipboard, file inputs, IME bridge).
- Intentionally does **not** implement input/event mapping; runner crates own the event layer.
- Exposes a stub `PlatformError` on non-wasm targets to keep the workspace buildable.

Evidence anchors:

- `crates/fret-platform-web/src/lib.rs`
- `crates/fret-platform-web/src/wasm/mod.rs`
- `crates/fret-platform-web/src/native/mod.rs`

## 2) Public contract surface

- Key exports / stable types (wasm32):
  - `WebPlatformServices` (effect handler + queued events + IME bridge/timers state).
- Non-wasm:
  - `PlatformError` (actionable error message).

Evidence anchors:

- `crates/fret-platform-web/src/wasm/mod.rs`
- `crates/fret-platform-web/src/native/mod.rs`

## 3) Dependency posture

- wasm32-only deps: `web-sys`, `wasm-bindgen`, `wasm-bindgen-futures`, `js-sys` plus workspace deps `fret-core`/`fret-runtime`.
- Layering risk: ensure wasm deps remain cfg-gated and do not leak into portable crates.

Evidence anchors:

- `crates/fret-platform-web/Cargo.toml`
- `pwsh -NoProfile -File tools/check_layering.ps1`

## 4) Module ownership map (internal seams)

- Effect integration + service state
  - Files: `crates/fret-platform-web/src/wasm/mod.rs`
- IME bridge (DOM textarea mount, composition events)
  - Files: `crates/fret-platform-web/src/wasm/ime.rs`
- Timers (timeouts/intervals mapped to `TimerToken`)
  - Files: `crates/fret-platform-web/src/wasm/timers.rs`
- File dialog / input element bridges
  - Files: `crates/fret-platform-web/src/wasm/file_dialog.rs`
- Non-wasm stub
  - Files: `crates/fret-platform-web/src/native/mod.rs`

## 5) Refactor hazards (what can regress easily)

- IME correctness and lifecycle (composition start/update/end)
  - Failure mode: missing text events, cursor rect drift, focus/blur edge cases.
  - Existing gates: none in this crate on non-wasm; recommend adding a `fretboard diag` or browser automation suite once infrastructure is ready.
- Re-entrancy and ordering with `Rc<RefCell<...>>` queues
  - Failure mode: missed wakeups, stale queued events, timer ordering drift.
  - Existing gates: minimal non-wasm test ensuring actionable error; wasm gates still needed.
- Capability gating (`PlatformCapabilities`) for clipboard/timers
  - Failure mode: effects silently drop or enqueue wrong fallback events.

Evidence anchors:

- `crates/fret-platform-web/src/wasm/mod.rs`
- `crates/fret-runtime/src/effect.rs`

## 6) Code quality findings (Rust best practices)

- `cfg(target_arch = "wasm32")` gating is clean and keeps native workspace builds light.
- Recommendation: keep wasm-only behavior covered by at least a “wasm build” gate (CI) even if runtime/browser tests are not ready yet.

Evidence anchors:

- `crates/fret-platform-web/src/lib.rs`

## 7) Recommended refactor steps (small, gated)

1. Add a CI gate to compile this crate for `wasm32-unknown-unknown` — outcome: prevent accidental breakage in wasm-only code paths — gate: `cargo check -p fret-platform-web --target wasm32-unknown-unknown`.
2. Add a fixture-driven test harness for timers/queued events logic that can run in wasm test environments — outcome: stabilize ordering assumptions — gate: wasm test runner (TBD).
3. Keep IME behavior aligned with the ADR (0195) and add at least one scripted diag in a web demo — outcome: prevent regressions in composition — gate: `fretboard diag` (web).

## 8) Open questions / decisions needed

- Where should the canonical “web platform capabilities defaults” live: this crate, runner-web, or app shells?
- Do we want a uniform error taxonomy for browser API failures (structured kinds), or is “best-effort + fallback events” sufficient?

