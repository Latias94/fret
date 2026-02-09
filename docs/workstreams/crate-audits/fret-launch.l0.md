# Crate audit (L0) — `fret-launch`

## Crate

- Name: `fret-launch`
- Path: `crates/fret-launch`
- Owners / adjacent crates: `fret-app`, `fret-runtime`, `fret-render-wgpu`, `fret-runner-winit`, `fret-platform-*`
- Current “layer”: runner/launcher wiring (backend adapter + demo entrypoints)

## 1) Purpose (what this crate *is*)

- A “wiring” facade that composes runner + platform + UI runtime + renderer for apps/demos.
- Owns the *hosted* runner entrypoints (`run_app*`) and cross-platform runner glue (desktop + wasm32).
- Intentionally allowed to depend on backend crates (`winit`, `wgpu`, `web-sys`) because this is where the integration happens.

Evidence anchors:

- `crates/fret-launch/src/lib.rs`
- `crates/fret-launch/README.md`

## 2) Public contract surface

- Key exports / stable types:
  - Desktop: `run_app`, `run_app_with_event_loop`, `WinitRunner`, `WinitRunnerConfig`, `WinitAppBuilder` (native-only)
  - Renderer init/hooks: `WgpuInit`, `WinitRenderContext`, `EngineFrameUpdate`
  - Viewport overlays: `ViewportRenderTarget*`, `ViewportOverlay3dHooks*`, `install_viewport_overlay_3d_immediate`, `record_viewport_overlay_3d`, `upload_viewport_overlay_3d_immediate`
- “Accidental” exports to consider removing:
  - The crate currently re-exports a large surface from `runner::*`, which risks turning internal wiring helpers into “public API by default”.
- Feature flags and intent:
  - `hotpatch-subsecond`, `diag-screenshots` are intentionally dev-only (good posture), but should be kept compile-gated and tested.

Evidence anchors:

- `crates/fret-launch/src/lib.rs`
- `crates/fret-launch/Cargo.toml`

## 3) Dependency posture

- Backend coupling:
  - Direct deps include `wgpu`, `winit`, and wasm (`web-sys`, `wasm-bindgen-futures`) under `cfg(target_arch = "wasm32")`.
  - Native-only OS deps are correctly target-gated (`cocoa`/`objc` for macOS, `windows-sys` for Windows).
- Layering policy:
  - Expected: this crate is not a kernel crate; it is allowed to couple to platform/runner/render stacks.
- Compile-time hotspots / heavy deps:
  - Large `src/runner/desktop/mod.rs` suggests high churn risk and slower incremental builds when touched.

Evidence anchors:

- `crates/fret-launch/Cargo.toml`
- `pwsh -NoProfile -File tools/audit_crate.ps1 -Crate fret-launch`

## 4) Module ownership map (internal seams)

- Runner facade + shared helpers
  - Files: `crates/fret-launch/src/runner/mod.rs`, `crates/fret-launch/src/runner/common/*`
- Desktop runner integration (winit + wgpu host loop + OS menus + diagnostics)
  - Files: `crates/fret-launch/src/runner/desktop/mod.rs`, `crates/fret-launch/src/runner/desktop/app_handler.rs`, `crates/fret-launch/src/runner/desktop/dispatcher.rs`, `crates/fret-launch/src/runner/desktop/*_menu.rs`, `crates/fret-launch/src/runner/desktop/diag_screenshots.rs`
- Web runner integration (wasm host loop + effects)
  - Files: `crates/fret-launch/src/runner/web/*`
- Streaming images / YUV conversion utilities (likely used by diagnostics or GPU uploads)
  - Files: `crates/fret-launch/src/runner/streaming_upload.rs`, `crates/fret-launch/src/runner/streaming_images.rs`, `crates/fret-launch/src/runner/yuv*.rs`, `crates/fret-launch/src/runner/yuv_nv12_convert.wgsl`

## 5) Refactor hazards (what can regress easily)

- Runner event loop ordering / determinism (desktop)
  - Failure mode: missed redraw/effect drains, duplicated frames, non-deterministic “settle” behavior for overlays.
  - Existing gates: web-golden conformance in `ecosystem/fret-ui-shadcn/tests/*` (indirect coverage).
  - Missing gate to add: a minimal “runner frame loop invariants” test harness for `fret-launch` (or a `fretboard diag` suite) that asserts redraw scheduling invariants.
- Cross-platform compilation drift (cfg explosion)
  - Failure mode: native-only code accidentally used under wasm32 (or vice versa); CI-only failures.
  - Existing gates: none specific in this crate.
  - Missing gate to add: `cargo check -p fret-launch --target wasm32-unknown-unknown` + at least one native target in CI.
- Public surface drift via broad `pub use`
  - Failure mode: downstream code starts depending on internal wiring helpers, blocking refactors.
  - Existing gates: none.
  - Missing gate to add: a small `public_api` snapshot gate (or at least an explicit re-export policy documented in `README.md`).
- Dev-tool features leaking into normal builds
  - Failure mode: `hotpatch-subsecond` / `diag-screenshots` accidentally referenced in default codepaths.
  - Existing gates: feature flags exist.
  - Missing gate to add: `cargo check -p fret-launch --features hotpatch-subsecond` and `--features diag-screenshots` gates.

## 6) Code quality findings (Rust best practices)

- The dominant maintainability risk is *module size* and “god module” drift in `src/runner/desktop/mod.rs` (5k+ LOC).
- Recommend making the facade file small and moving subsystems into explicit modules, with one “ownership map” section in `README.md`.

Evidence anchors:

- `crates/fret-launch/src/runner/desktop/mod.rs`
- `crates/fret-launch/README.md`

## 7) Recommended refactor steps (small, gated)

1. Split `crates/fret-launch/src/runner/desktop/mod.rs` into a thin facade + submodules (event loop, menus, diagnostics, hotpatch, dispatcher) — outcome: smaller diffs and faster review — gate: `cargo test -p fret-launch --no-run` + `pwsh -NoProfile -File tools/check_layering.ps1`.
2. Add “compile gates” for feature flags + wasm target — outcome: catch cfg/feature regressions early — gate: `cargo check -p fret-launch --features hotpatch-subsecond`, `cargo check -p fret-launch --features diag-screenshots`, `cargo check -p fret-launch --target wasm32-unknown-unknown`.
3. Document the intended public surface (what is stable vs wiring-only) — outcome: fearless refactors without downstream breakage — gate: none initially (follow-up: public API snapshot).

## 8) Open questions / decisions needed

- Should `fret-launch` be purely a wiring crate, with the “stable runner API” living in `fret-runner-winit` / `fret-runner-web`, and `fret-launch` only providing a small set of entrypoints?
- Do we want a policy that runner-facing surfaces must remain executor-agnostic (no Tokio in this layer), leaving async to the app/effects boundary?

