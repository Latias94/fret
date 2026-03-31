# Release Surface Fearless Refactor v1 — TODO Tracker

Status: In progress

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `RELFACE-{area}-{nnn}`

## Wave 1 — Publish blockers

- [x] RELFACE-fret-010 Remove the editor-only `workspace_shell` module from the `fret` public root
      surface.
  - Evidence:
    - `ecosystem/fret/src/lib.rs`
    - `ecosystem/fret/Cargo.toml`

- [x] RELFACE-fret-020 Remove the direct `fret-workspace` dependency from `fret`.
  - Evidence:
    - `ecosystem/fret/Cargo.toml`

- [x] RELFACE-router-030 Promote `fret-router-ui` from `publish = false` to a publishable thin
      adoption crate.
  - Evidence:
    - `ecosystem/fret-router-ui/Cargo.toml`
    - `docs/crate-usage-guide.md`

- [x] RELFACE-fret-040 Run release-oriented preflight on `fret` after the blocker cuts.
  - Evidence:
    - `cargo nextest run -p fret --features router`
    - `cargo check -p fret-router-ui`
    - `cargo metadata --format-version 1 --no-deps` spot check for `fret` direct deps

## Wave 2 — Closure slimming

- [ ] RELFACE-shadcn-100 Audit unconditional `fret-ui-shadcn` dependencies and split heavy lanes
      behind explicit features where they are not part of the default design-system baseline.
  - Starting suspects:
    - `fret-chart`
    - `fret-canvas`
    - `fret-executor`

- [ ] RELFACE-bootstrap-110 Audit `fret-bootstrap` feature fan-out and separate true onboarding
      defaults from maintainer/advanced integrations.
  - Starting suspects:
    - diagnostics websocket support
    - optional icon packs
    - ui-assets integration
    - command palette wiring

- [ ] RELFACE-fret-120 Re-audit the `fret` optional feature matrix after Wave 2 and cut anything
      that still feels like an internal repo convenience instead of a real public contract.

## Wave 3 — Release candidate set

- [ ] RELFACE-release-200 Freeze the Wave 1/Wave 2 release set.
  - Candidate user-facing set:
    - `fret`
    - `fret-framework`
    - `fret-bootstrap`
    - `fret-ui-kit`
    - `fret-ui-shadcn`
    - `fret-selector`
    - `fret-query`

- [ ] RELFACE-release-210 Confirm tooling-only crates stay out of the minimal library release set.
  - Current non-goal set:
    - `fretboard`
    - `fret-diag`
    - repo-owned harness apps

- [ ] RELFACE-release-220 Update release docs/checklists to reflect the selected crate set instead
      of workspace-wide assumptions.
