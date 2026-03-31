# Release Surface Fearless Refactor v1 (Milestones)

## Status summary

- `M0` Baseline and blocker identification: **Completed**
- `M1` Publish blocker removal: **In progress**
- `M2` Closure slimming: **In progress**
- `M3` Release candidate freeze: **Not started**

## M0 — Baseline and blocker identification

**Status:** Completed

**What closed**

- release analysis established the distinction between:
  - user-facing entry crates,
  - actual publish closure crates,
  - tooling-only crates.
- the first hard blockers were identified:
  - `fret` depended on `fret-workspace`,
  - `fret` exposed `workspace_shell` despite that surface being editor-only,
  - `fret/router` depended on `fret-router-ui` while that crate was still `publish = false`.

**Evidence**

- `docs/workstreams/release-surface-fearless-refactor-v1/DESIGN.md`
- `ecosystem/fret/Cargo.toml`
- `ecosystem/fret-router-ui/Cargo.toml`

## M1 — Publish blocker removal

**Status:** In progress

**Current landed slice**

- removed `workspace_shell` from the `fret` public root surface,
- removed the direct `fret-workspace` dependency from `fret`,
- promoted `fret-router-ui` to a publishable thin adoption crate,
- verified `fret` with router feature enabled and confirmed its direct deps no longer include
  `publish = false` crates.

**Remaining to close M1**

- extend the same publishability audit from `fret` to the rest of the selected release lane,
- decide whether any additional optional facade lanes should be cut before Wave 2 closure slimming.

**Evidence**

- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/Cargo.toml`
- `ecosystem/fret-router-ui/Cargo.toml`

## M2 — Closure slimming

**Status:** In progress

**Current landed slice**

- `fret-ui-shadcn` moved chart recipes behind an explicit `chart` feature instead of pulling
  `fret-chart` into the default closure.
- `fret-ui-shadcn` moved executor-backed async recipe helpers behind an explicit
  `executor-integration` feature instead of pulling `fret-executor` into the default closure.
- `fret-ui-gallery` now opts into that feature only on `gallery-chart` / `gallery-dev`.
- `fret-bootstrap` no longer binds command palette capability directly to `fret-ui-shadcn`;
  default shadcn command palette UI now lives in `fret-bootstrap-shadcn`.
- `fret`'s `command-palette` feature now pulls in that bridge explicitly, keeping the app-facing
  authoring story unchanged while making the crate boundary honest.
- `fret-bootstrap/diagnostics` no longer pulls an unused `fret-query` edge into the diagnostics
  closure.
- retained canvas cache diagnostics now sit behind explicit
  `fret-bootstrap/diagnostics-canvas` instead of bloating the base diagnostics lane.
- devtools WS transport now sits behind thin `fret-bootstrap-diag-ws` instead of making
  `fret-bootstrap` depend on `fret-diag-ws` directly.

**Remaining to close**

- `fret-bootstrap` feature fan-out is rebalanced so onboarding defaults stay smaller and more
  predictable,
- the `fret` optional feature matrix maps cleanly to publishable, intentional extension seams.

**Evidence**

- `ecosystem/fret-ui-shadcn/Cargo.toml`
- `ecosystem/fret-ui-shadcn/src/lib.rs`
- `ecosystem/fret-ui-shadcn/src/sonner.rs`
- `apps/fret-ui-gallery/Cargo.toml`
- `ecosystem/fret-bootstrap/Cargo.toml`
- `ecosystem/fret-bootstrap/src/ui_app_driver.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics_ws_bridge.rs`
- `ecosystem/fret-bootstrap-diag-ws/src/lib.rs`
- `ecosystem/fret-bootstrap-shadcn/src/lib.rs`
- `ecosystem/fret/Cargo.toml`
- `cargo check -p fret-bootstrap --features diagnostics`
- `cargo check -p fret-bootstrap --features diagnostics-canvas`
- `cargo check -p fret-bootstrap --features diagnostics-ws`

## M3 — Release candidate freeze

**Status:** Not started

**What should close**

- the first public release crate set is explicit and documented,
- docs teach only that set as the recommended path,
- tooling-only crates are explicitly out of the minimal library release.
