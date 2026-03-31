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

- [x] RELFACE-shadcn-100 Audit unconditional `fret-ui-shadcn` dependencies and split heavy lanes
      behind explicit features where they are not part of the default design-system baseline.
  - Landed slice:
    - chart recipes now sit behind `fret-ui-shadcn/chart`
    - executor-backed async recipe helpers now sit behind
      `fret-ui-shadcn/executor-integration`
    - `fret-ui-gallery` opts in only on `gallery-chart` / `gallery-dev`
  - Evidence:
    - `ecosystem/fret-ui-shadcn/Cargo.toml`
    - `ecosystem/fret-ui-shadcn/src/lib.rs`
    - `ecosystem/fret-ui-shadcn/src/sonner.rs`
    - `apps/fret-ui-gallery/Cargo.toml`
    - `cargo check -p fret-ui-shadcn`
    - `cargo check -p fret-ui-shadcn --features chart`
    - `cargo check -p fret-ui-shadcn --features executor-integration`
    - `cargo tree -p fret-ui-shadcn -e normal`
    - `python3 tools/check_layering.py`

- [~] RELFACE-bootstrap-110 Audit `fret-bootstrap` feature fan-out and separate true onboarding
      defaults from maintainer/advanced integrations.
  - Landed slice:
    - `fret-bootstrap/ui-app-command-palette` now keeps only the driver capability surface
      (toggle + gating + per-window models)
    - shadcn-specific command palette UI moved into a thin `fret-bootstrap-shadcn` bridge crate
    - `fret`'s `command-palette` feature now explicitly depends on that bridge instead of relying
      on a hidden `fret-bootstrap -> fret-ui-shadcn` edge
  - Remaining suspects:
    - diagnostics websocket support
    - optional icon packs
    - ui-assets integration
  - Evidence:
    - `ecosystem/fret-bootstrap/Cargo.toml`
    - `ecosystem/fret-bootstrap/src/ui_app_driver.rs`
    - `ecosystem/fret-bootstrap-shadcn/src/lib.rs`
    - `ecosystem/fret/Cargo.toml`
    - `ecosystem/fret/src/lib.rs`
    - `cargo check -p fret-bootstrap --features ui-app-command-palette`
    - `cargo check -p fret-bootstrap-shadcn`
    - `cargo check -p fret --features command-palette`

- [ ] RELFACE-fret-120 Re-audit the `fret` optional feature matrix after Wave 2 and cut anything
      that still feels like an internal repo convenience instead of a real public contract.

## Wave 3 — Release candidate set

- [ ] RELFACE-release-200 Freeze the Wave 1/Wave 2 release set.
  - Candidate user-facing set:
    - `fret`
    - `fret-framework`
    - `fret-bootstrap`
    - `fret-bootstrap-shadcn`
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
