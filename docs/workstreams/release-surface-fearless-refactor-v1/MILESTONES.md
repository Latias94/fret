# Release Surface Fearless Refactor v1 (Milestones)

## Status summary

- `M0` Baseline and blocker identification: **Completed**
- `M1` Publish blocker removal: **In progress**
- `M2` Closure slimming: **Not started**
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

**Status:** Not started

**What should close**

- `fret-ui-shadcn` no longer pulls obviously non-baseline heavy crates into its default closure,
- `fret-bootstrap` feature fan-out is rebalanced so onboarding defaults stay smaller and more
  predictable,
- the `fret` optional feature matrix maps cleanly to publishable, intentional extension seams.

## M3 — Release candidate freeze

**Status:** Not started

**What should close**

- the first public release crate set is explicit and documented,
- docs teach only that set as the recommended path,
- tooling-only crates are explicitly out of the minimal library release.
