# Crate audit (L0) — `fret-runner-winit`

## Crate

- Name: `fret-runner-winit`
- Path: `crates/fret-runner-winit`
- Owners / adjacent crates: `fret-core` (portable events/types), platform crates, renderer crates, `fret-a11y-accesskit`
- Current “layer”: native runner glue (windowing + event mapping + IME + accessibility)

## 1) Purpose (what this crate *is*)

- Winit-based runner glue for native platforms:
  - maps winit `WindowEvent`s to `fret-core::Event`
  - manages window-side state (cursor icon, IME enable/cursor area) and prepares per-frame requests
  - integrates accessibility plumbing (AccessKit)
  - supports external drag/drop token flows at the runner boundary

Evidence anchors:

- `crates/fret-runner-winit/src/lib.rs`
- `crates/fret-runner-winit/src/windows_ime.rs`
- `crates/fret-runner-winit/src/window_registry.rs`

## 2) Public contract surface

- Key exports / stable types (observed):
  - `WinitPlatform` / `WinitInputState` (window event handling and input state)
  - event mapping helpers (wheel delta, pointer id/type mapping, key mapping, text sanitization)
  - `accessibility`, `window_registry`, and (Windows-only) `windows_ime`
- “Accidental” exports to consider removing (L0 hypothesis):
  - Keep the crate-root surface explicit; avoid “helper sprawl” at root by moving mapping helpers into a dedicated module facade once the public surface is mapped.

Evidence anchors:

- `crates/fret-runner-winit/src/lib.rs`
- `crates/fret-runner-winit/Cargo.toml`

## 3) Dependency posture

- External deps: `winit`, `accesskit`, `tracing`, plus `windows-sys` on Windows.
- Layering posture: this crate is intentionally backend/platform-coupled; keep those deps from leaking into core contract crates.

Evidence anchors:

- `crates/fret-runner-winit/Cargo.toml`
- `pwsh -NoProfile -File tools/check_layering.ps1`

## 4) Module ownership map (internal seams)

- Input/event mapping and state machine (pointer/buttons/click counting/modifiers/wheel)
  - Files: `crates/fret-runner-winit/src/mapping/*`, `crates/fret-runner-winit/src/state/input/mod.rs`
- Window-side state (cursor + IME request dedupe)
  - Files: `crates/fret-runner-winit/src/state/window.rs`
- Accessibility (AccessKit integration)
  - Files: `crates/fret-runner-winit/src/accessibility.rs`
  - Note: `crates/fret-runner-winit/src/accessibility_accesskit_winit.rs` is intentionally disabled while we are on `winit` beta and waiting for `accesskit_winit` compatibility.
- External drag/drop glue
  - Files: `crates/fret-runner-winit/src/external_drag.rs`
- Window registry/bookkeeping
  - Files: `crates/fret-runner-winit/src/window_registry.rs`
- Windows IME integration (platform-specific)
  - Files: `crates/fret-runner-winit/src/windows_ime.rs`

## 5) Refactor hazards (what can regress easily)

- Input mapping and click counting semantics
  - Failure mode: incorrect multi-click counts, “click vs drag” detection drift, pointer id mapping drift.
  - Existing gates: unit tests in `mapping/*` and `state/*` (pointer id mapping, IME cursor quantization, click slop behavior).
  - Missing gate to add: fixture-driven event-sequence tests once we have a stable harness for winit `WindowEvent` construction.
- IME lifecycle correctness and cursor-area deduplication
  - Failure mode: cursor rect updates missed or spammed; enable/disable ordering issues; scale-factor change regressions.
  - Existing gates: unit tests for quantization/dedupe logic.
  - Missing gate to add: an end-to-end IME regression script in a demo (`fretboard diag`) once available.
- Accessibility integration boundary
  - Failure mode: stale semantics tree, focus mismatch, incorrect node ids.
  - Existing gates: not audited at L0.
  - Missing gate to add: at least one deterministic semantics snapshot test for a small UI tree (if feasible without a full window).
- Module wiring drift
  - Failure mode: refactors add new modules/files that are not wired into the crate root (or are feature-gated incorrectly), leading to dead code and missing gates.
  - Recommended action: keep `src/lib.rs` as a thin facade that explicitly declares internal modules and re-exports a stable surface.

Evidence anchors:

- `crates/fret-runner-winit/src/lib.rs`
- `crates/fret-runner-winit/src/mapping/mod.rs`
- `crates/fret-runner-winit/src/state/mod.rs`

## 6) Code quality findings (Rust best practices)

- Keep `src/lib.rs` as a thin facade and avoid re-growing a “god module”.

Evidence anchors:

- `crates/fret-runner-winit/src/lib.rs`
- `crates/fret-runner-winit/src/mapping/mod.rs`
- `crates/fret-runner-winit/src/state/mod.rs`

## 7) Recommended refactor steps (small, gated)

1. Wire `mapping/` + `state/` into the crate module tree and remove duplicated implementations from `lib.rs` — outcome: one source of truth — gate: `cargo nextest run -p fret-runner-winit`.
   - Status: landed.
2. Keep mapping helpers pure and unit-tested (no window handles) — outcome: easy regression gates — gate: unit tests + `nextest`.
3. Add one deterministic `fretboard diag` for an IME + text-input flow on Windows once runner-level diag is available — outcome: catch end-to-end regressions — gate: diag suite (future).

## 8) Open questions / decisions needed

- Should click slop and multi-click delay be treated as a stable, documented contract (shared across runners), or remain runner-specific?
- Where should normalization policy live long-term: `fret-core` input normalization or runner-specific mapping?
