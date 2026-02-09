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
- ADR 0075: `docs/adr/0075-docking-layering.md` (referenced by crate docs)

## 2) Public contract surface

- Key exports / stable types:
  - Dock space: `DockSpace`, `DockSpaceMount`, `DockViewportLayout`, `DockViewportOverlayHooks*`
  - Panels: `DockPanel`, `DockPanelRegistry*`, `ViewportPanel`
  - Runtime integration: `DockingRuntime`, `handle_dock_*` helpers
- “Accidental” exports to consider removing:
  - Surface is fairly explicit via `pub use` list; risk is “policy helpers” becoming public by convenience.
- Feature flags and intent:
  - `imui` feature pulls in `fret-authoring` (optional) and should remain strictly opt-in.

Evidence anchors:

- `ecosystem/fret-docking/src/lib.rs`
- `ecosystem/fret-docking/Cargo.toml`

## 3) Dependency posture

- Backend coupling risks:
  - No direct platform deps; depends on `fret-ui` and `fret-runtime` plus `fret-dnd`.
  - Notable: enables `fret-ui` feature `unstable-retained-bridge` (high refactor hazard; should be intentional and tracked).
- Layering policy compliance:
  - Expected for docking policy crate, but must avoid drifting into backend-specific behavior.
- Compile-time hotspots / heavy deps:
  - Very large test module: `src/dock/tests.rs` (~5.8k LOC).
  - Large implementation module: `src/dock/space.rs` (~4.6k LOC).

Evidence anchors:

- `ecosystem/fret-docking/Cargo.toml`
- `pwsh -NoProfile -File tools/audit_crate.ps1 -Crate fret-docking`

## 4) Module ownership map (internal seams)

- Dock space UI + layout + interaction
  - Files: `ecosystem/fret-docking/src/dock/space.rs`, `ecosystem/fret-docking/src/dock/layout.rs`, `ecosystem/fret-docking/src/dock/manager.rs`
- Painting and geometry helpers
  - Files: `ecosystem/fret-docking/src/dock/paint.rs`, `ecosystem/fret-docking/src/dock/tab_bar_geometry.rs`, `ecosystem/fret-docking/src/dock/split_stabilize.rs`
- Runtime binding / event handling
  - Files: `ecosystem/fret-docking/src/runtime.rs`, `ecosystem/fret-docking/src/facade.rs`, `ecosystem/fret-docking/src/invalidation.rs`
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
- `unstable-retained-bridge` coupling to `fret-ui`
  - Failure mode: changes in retained bridge invalidate docking assumptions; regressions only show up in apps.
  - Existing gates: none obvious at L0.
  - Missing gate to add: a minimal `fretboard diag` suite that exercises drag/split/tab activation across a couple of representative layouts.

## 6) Code quality findings (Rust best practices)

- The biggest maintainability issue is *module size* (both implementation and tests).
- Recommend explicitly separating:
  - pure geometry/layout math,
  - interaction policy (pointer/keyboard routing),
  - runtime binding/invalidation,
  - and test fixtures/harness.

Evidence anchors:

- `ecosystem/fret-docking/src/dock/space.rs`
- `ecosystem/fret-docking/src/dock/tests.rs`

## 7) Recommended refactor steps (small, gated)

1. Convert `ecosystem/fret-docking/src/dock/tests.rs` into a fixture-driven harness — outcome: stable, reviewable matrices — gate: `cargo nextest run -p fret-docking`.
2. Split `ecosystem/fret-docking/src/dock/space.rs` into submodules by responsibility (layout, hit-testing, drag ops, tab bar, viewport overlay integration) — outcome: fewer merge conflicts and clearer ownership — gate: docking fixture tests + `pwsh -NoProfile -File tools/check_layering.ps1`.
3. Add a minimal docking interaction diag suite — outcome: catch regressions that unit tests miss — gate: `fretboard diag` suite (name TBD).

## 8) Open questions / decisions needed

- What is the intended contract for tear-off/multi-window docking vs “viewport panels” (and where is that contract recorded—ADR vs workstream)?

