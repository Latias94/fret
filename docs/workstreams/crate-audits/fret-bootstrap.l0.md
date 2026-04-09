# Crate audit (L0) — `fret-bootstrap`

## Crate

- Name: `fret-bootstrap`
- Path: `ecosystem/fret-bootstrap`
- Owners / adjacent crates: `fret-launch`, `fret-app`, `fret-ui-app`, `fret-ui-kit`, `fret-diag-protocol`, `fretboard`
- Current “layer”: ecosystem “golden path” bootstrap + diagnostics integration (not a portable kernel contract)

## 1) Purpose (what this crate *is*)

- Provides an opinionated bootstrap/builder wrapper for native app startup (`BootstrapBuilder`, config files, i18n defaults).
- Hosts “app defaults” wiring for UI apps (`ui_app_driver` feature).
- Hosts the in-app diagnostics runtime integration (`ui_diagnostics` module; scripts/picking/inspector overlay) behind `diagnostics*` and `ui-app-driver` feature gates.

Evidence anchors:

- `ecosystem/fret-bootstrap/src/lib.rs`
- `ecosystem/fret-bootstrap/src/ui_app_driver.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs`

## 2) Public contract surface

- Intended exports (golden path / app integration):
  - `BootstrapBuilder`, `apply_settings`, `install_default_i18n_backend`
  - `ui_app(...)` helpers and `ui_app_driver` module (feature-gated)
- Diagnostics implementation details should remain internal:
  - `ui_diagnostics` is a diagnostics-only surface and should avoid leaking stable “app API” contracts.

Evidence anchors:

- `ecosystem/fret-bootstrap/src/lib.rs`
- `ecosystem/fret-bootstrap/Cargo.toml`

## 3) Dependency posture

- Expected heavy coupling (acceptable for bootstrap):
  - Depends on runner glue (`fret-launch`) and renderer integration (`fret-render`).
  - Depends on UI stack under `ui-app-driver` (`fret-ui`, `fret-ui-app`, `fret-ui-kit`).
- Diagnostics surfaces are feature-gated:
  - `diagnostics` / `diagnostics-ws` add additional deps (`fret-diag-ws`, `fret-query`, `fret-canvas`).
- Layering policy risk:
  - This crate is ecosystem-level, so coupling is acceptable, but avoid turning it into a “god crate” that all apps must depend on for non-bootstrap needs.

Evidence anchors:

- `ecosystem/fret-bootstrap/Cargo.toml`
- `docs/dependency-policy.md`

## 4) Module ownership map (internal seams)

- Bootstrap + config defaults
  - Files: `ecosystem/fret-bootstrap/src/lib.rs`, `ecosystem/fret-bootstrap/src/config_files.rs` (if present)
- UI driver integration
  - Files: `ecosystem/fret-bootstrap/src/ui_app_driver.rs`
- UI diagnostics runtime (scripts, bundle export, inspector overlay)
  - Files: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`, `ecosystem/fret-bootstrap/src/ui_diagnostics/*`
  - Key seams (recently tightened): `InspectController` + `InspectOverlayModel` snapshot

Evidence anchors:

- `ecosystem/fret-bootstrap/src/ui_diagnostics/inspect_controller.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/inspect.rs`

## 5) Refactor hazards (what can regress easily)

- Inspector state machine drift (pick/inspect/help/tree) during refactors
  - Failure mode: stale overlay, stuck “armed pick”, or scripts mutating inspector state directly.
  - Existing gates: unit tests in `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` + scripted suite gates.
  - Missing gate to add: keep at least one stable “inspector smoke” diag suite in `tools/diag-scripts/suites/ui-gallery-overlay-steady/`.
- Script engine determinism (off-window predicates, cached test-id bounds, timeouts)
  - Failure mode: scripts stall when windows are occluded or migrate incorrectly.
  - Existing gates: `cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics`.
  - Missing gate to add: a small, targeted script that asserts cached-test-id behavior and migration rules in a multi-window scenario.
- File-size hotspots (review friction)
  - `src/ui_diagnostics.rs` (~3.5k LOC) and `src/ui_app_driver.rs` (~2.7k LOC) are “mixed responsibility” risks.
  - Failure mode: changes become hard to review and hard to regression-gate.

Evidence anchors:

- `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
- `ecosystem/fret-bootstrap/src/ui_app_driver.rs`

## 6) Code quality findings (Rust best practices)

- Feature gating is doing real work: diagnostics and UI-driver integration are opt-in, keeping the default crate usable as a bootstrap layer.
- The diagnostics module should keep “mechanism vs policy” boundaries explicit:
  - Avoid pushing interaction policy into `crates/fret-ui`.
  - Prefer data artifacts (bundle + JSON scripts) for shareable repros.

Evidence anchors:

- `ecosystem/fret-bootstrap/Cargo.toml` (features)
- `docs/workstreams/ui-diagnostics-inspector-v1/ui-diagnostics-inspector-v1.md`
- `docs/ui-diagnostics-and-scripted-tests.md`

## 7) Recommended refactor steps (small, gated)

1. Split the largest UI diagnostics entrypoint file for reviewability (e.g. `ui_diagnostics.rs` → `ui_diagnostics/mod.rs` + focused submodules) — gate: `cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics`.
2. Keep inspector read/write surfaces centralized:
   - Reads via `InspectOverlayModel` snapshot; mutations via `InspectController` APIs — gate: existing inspector scripted suites.
3. Add one additional “multi-window script migration” diag suite (small, deterministic) — gate: `fretboard-dev diag run ...` and keep it in a stable suite.

## 8) Open questions / decisions needed

- Should `ui_diagnostics` eventually move into a dedicated crate (e.g. `ecosystem/fret-ui-diagnostics`) to keep `fret-bootstrap` narrowly “bootstrap”, or is co-location with the UI driver the intended long-lived home?
- If script authoring needs control-flow (loops/conditionals), should we extend the JSON protocol with deterministic control-flow steps (preferred), rather than making Rust the primary scripting surface?

