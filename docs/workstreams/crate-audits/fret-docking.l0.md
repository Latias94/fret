# Crate audit (L0) — `fret-docking`

## Crate

- Name: `fret-docking`
- Path: `ecosystem/fret-docking`
- Owners / adjacent crates: `fret-core` (dock graph/ops/persistence), `crates/fret-ui` (mechanism substrate), `ecosystem/fret-dnd` (drag/drop), runner/app shells that host multi-window
- Current “layer”: ecosystem docking UI + interaction policy

## 1) Purpose (what this crate *is*)

- Docking UI and interaction policy built on top of the `fret-ui` substrate, aligned with ADR 0075.
- Owns the “editor-grade” behaviors that are hard to change later: split/resize, tab bars, panel activation, tear-off/viewport integration (as applicable).
- Keeps the dock graph/ops/persistence in `fret-core` while leaving `fret-ui` mechanism-only.

Evidence anchors:

- `ecosystem/fret-docking/src/lib.rs`
- ADR 0075: `docs/adr/0075-docking-layering-b-route-and-retained-bridge.md` (referenced by crate docs)

## 2) Public contract surface

- Key exports / stable types:
  - App surface: `DockSurface`, `DockHostOptions`
  - Panels: `DockPanel`, `DockPanelElementRegistry`, `ViewportPanel`
  - Viewport integration: `DockViewportLayout`, `DockViewportOverlayHooks`
  - Policy: `DockingPolicy`
  - Advanced low-level access: `fret_docking::advanced::{DockSurfaceDriver, DockRuntimeCommand, DockManager, DockWorkspace, DockPanelCatalog, ...}`
- Public-surface rule:
  - Ordinary apps should use `DockSurface` for panel registration, host mounting, layout import/export, and policy installation.
  - Host/runtime integrations should opt into `DockSurface::driver()` for graph root construction, runtime command handoff, window-created callbacks, and before-close merge.
  - Manager/workspace/catalog access is intentionally explicit under `advanced`; free runtime helpers and service globals are crate-private/internal.
- Feature flags and intent:
  - `imui` feature pulls in `fret-authoring` (optional) and should remain strictly opt-in.

Evidence anchors:

- `ecosystem/fret-docking/src/lib.rs`
- `ecosystem/fret-docking/Cargo.toml`

## 3) Dependency posture

- Backend coupling risks:
  - No direct platform deps; depends on `fret-ui` and `fret-runtime` plus `fret-dnd`.
  - The old `fret-ui/unstable-retained-bridge` coupling has exited; `fret-docking` no longer enables that feature.
- Layering policy compliance:
  - Expected for docking policy crate, but must avoid drifting into backend-specific behavior.
- Compile-time hotspots / heavy deps:
  - Large declarative/event test surface split across `src/dock/tests/{dock_space,split}.rs`.
  - Docking behavior is now split across facade/runtime/workspace/drop-resolve/declarative event modules; cross-module changes still need focused gates.

Evidence anchors:

- `ecosystem/fret-docking/Cargo.toml`
- `python tools/audit_crate.py --crate fret-docking`

## 4) Module ownership map (internal seams)

- App surface + model authority
  - Files: `ecosystem/fret-docking/src/facade.rs`, `ecosystem/fret-docking/src/dock/manager.rs`
- Declarative host + interaction arbitration
  - Files: `ecosystem/fret-docking/src/dock/declarative.rs`, `ecosystem/fret-docking/src/dock/declarative/events/*`, `ecosystem/fret-docking/src/dock/declarative/interaction/arbitration.rs`
- Drop transaction seam
  - Files: `ecosystem/fret-docking/src/dock/drop_resolve/{target,intent,transaction,diagnostics}.rs`
- Painting and geometry helpers
  - Files: `ecosystem/fret-docking/src/dock/paint.rs`, `ecosystem/fret-docking/src/dock/tab_bar_geometry.rs`, `ecosystem/fret-docking/src/dock/hit_test.rs`
- Runtime binding / event handling
  - Files: `ecosystem/fret-docking/src/runtime.rs`, `ecosystem/fret-docking/src/runtime/*`, `ecosystem/fret-docking/src/invalidation.rs`
- Test harness host utilities
  - Files: `ecosystem/fret-docking/src/test_host.rs`, `ecosystem/fret-docking/src/dock/tests.rs`

## 5) Refactor hazards (what can regress easily)

- Dock graph ↔ UI binding invariants (selection/activation/focus)
  - Failure mode: wrong panel activated, focus loops, broken keyboard navigation after dock ops.
  - Existing gates: large Rust tests in `src/dock/tests.rs` (but reviewability is low due to size).
  - Missing gate to add: fixture-driven harness for dock op sequences (JSON fixtures + thin runner) to reduce churn.
- Split/resize geometry and hit-testing
  - Failure mode: pixel drift, incorrect divider targeting, jitter under repeated layout passes.
  - Existing gates: likely covered implicitly by tests; unclear at L0.
  - Missing gate to add: a small deterministic geometry test suite (fixture cases for key split layouts).
- Runtime command handoff and window lifecycle
  - Failure mode: duplicate OS windows, stale `window_created` callbacks, canceled pending requests, or before-close paths losing panels.
  - Existing gates: `fret-docking` runtime/facade tests around duplicate suppression, stale/canceled callbacks, degradation, auto-close, and before-close merge.
  - Missing gate to add: a native scripted lifecycle diagnostic that covers create -> match `window_created` -> before-close merge -> auto-close in one run.
- Drop preview/commit consistency
  - Failure mode: preview and committed graph op disagree on target/zone/insert index/policy.
  - Existing gates: resolved drop transaction tests and declarative dock-space diagnostics assertions.

## 6) Code quality findings (Rust best practices)

- The biggest maintainability issue is still the breadth of docking behavior, but it now has clearer seams than the old single-host module shape.
- Recommend explicitly separating:
  - pure geometry/layout math,
  - interaction policy (pointer/keyboard routing),
  - runtime binding/invalidation,
  - and test fixtures/harness.

Evidence anchors:

- `ecosystem/fret-docking/src/dock/declarative/interaction/arbitration.rs`
- `ecosystem/fret-docking/src/dock/drop_resolve/transaction.rs`
- `ecosystem/fret-docking/src/dock/tests/{dock_space,split}.rs`

## 7) Recommended refactor steps (small, gated)

1. Convert `ecosystem/fret-docking/src/dock/tests.rs` into a fixture-driven harness — outcome: stable, reviewable matrices — gate: `cargo nextest run -p fret-docking`.
2. Continue shrinking declarative event files behind typed arbitration/drop-transaction adapters — outcome: fewer merge conflicts and clearer ownership — gate: docking fixture tests + `python tools/check_layering.py`.
3. Add a minimal docking interaction diag suite — outcome: catch regressions that unit tests miss — gate: `fretboard-dev diag` suite (name TBD).

## 8) Open questions / decisions needed

- What is the intended contract for tear-off/multi-window docking vs “viewport panels” (and where is that contract recorded—ADR vs workstream)?
