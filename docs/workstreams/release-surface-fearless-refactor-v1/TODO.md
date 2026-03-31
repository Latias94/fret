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

- [x] RELFACE-bootstrap-110 Audit `fret-bootstrap` feature fan-out and separate true onboarding
      defaults from maintainer/advanced integrations.
  - Landed slice:
    - `fret-bootstrap/ui-app-command-palette` now keeps only the driver capability surface
      (toggle + gating + per-window models)
    - shadcn-specific command palette UI now stays on explicit
      `fret-bootstrap/ui-app-command-palette-shadcn`
    - `fret`'s `command-palette` feature now maps directly to that bootstrap feature instead of
      relying on a separate bridge crate
    - `fret-bootstrap/diagnostics` no longer pulls the unused `fret-query` dependency into its
      closure
    - retained canvas cache diagnostics now sit behind explicit
      `fret-bootstrap/diagnostics-canvas`
    - devtools WS transport now stays on explicit `fret-bootstrap/diagnostics-ws` instead of
      requiring a separate bridge crate
    - bootstrap release posture is now feature-first: recommended bootstrap integrations stay on
      `fret-bootstrap` unless they become a truly separate authoring surface
  - Evidence:
    - `ecosystem/fret-bootstrap/Cargo.toml`
    - `ecosystem/fret-bootstrap/src/lib.rs`
    - `ecosystem/fret-bootstrap/src/ui_app_driver.rs`
    - `ecosystem/fret-bootstrap/src/ui_diagnostics_ws_bridge.rs`
    - `ecosystem/fret/Cargo.toml`
    - `ecosystem/fret/src/lib.rs`
    - `cargo check -p fret-bootstrap --features diagnostics`
    - `cargo tree -p fret-bootstrap --features diagnostics -e normal`
    - `cargo check -p fret-bootstrap --features diagnostics-canvas`
    - `cargo check -p fret-bootstrap --features diagnostics-ws`
    - `cargo tree -p fret-bootstrap --features diagnostics-ws -e normal`
    - `cargo check -p fret-bootstrap --features ui-app-command-palette`
    - `cargo check -p fret-bootstrap --features ui-app-command-palette-shadcn`
    - `cargo check -p fret --features command-palette`

- [x] RELFACE-fret-120 Re-audit the `fret` optional feature matrix after Wave 2 and cut anything
      that still feels like an internal repo convenience instead of a real public contract.
  - Landed slice:
    - first-party examples no longer rely on `fret/devloop`; they use `fret-launch/dev-state`
      directly where they actually need dev-state hooks
    - `fret/devloop` and `fret/tracing` are kept only as advanced/maintainer aliases with explicit
      comments instead of being taught as part of the primary root story
    - `fret/material3` and `fret/ui-ai` were removed from the root feature matrix; direct owning
      crates are now the only recommended and supported surface until a stable `fret` root story
      exists
    - `fret/editor` was removed from the root feature matrix; editor-themed apps now depend on
      `fret-ui-editor` directly and install their own preset replay policy
    - `fret/docking` was removed from the root feature matrix; docking demos and cookbook samples
      now depend on `fret-docking` directly
    - `fret` root now distinguishes between taught app-facing lanes and non-primary aliases
  - Evidence:
    - `ecosystem/fret/Cargo.toml`
    - `ecosystem/fret/README.md`
    - `ecosystem/fret/src/lib.rs`
    - `apps/fret-examples/Cargo.toml`
    - `apps/fret-cookbook/Cargo.toml`
    - `apps/fret-ui-gallery/Cargo.toml`
    - `docs/crate-usage-guide.md`
    - `cargo check -p fret`
    - `cargo check -p fret --features state`
    - `cargo check -p fret-examples`
    - `cargo check -p fret-cookbook --features cookbook-docking`
    - `python3 tools/check_layering.py`

## Wave 3 — Release candidate set

- [x] RELFACE-release-200 Freeze the Wave 1/Wave 2 release set.
  - Frozen user-facing set:
    - `fret`
    - `fret-framework`
    - `fret-bootstrap`
    - `fret-ui-kit`
    - `fret-ui-shadcn`
    - `fret-selector`
    - `fret-query`
  - Current progress:
    - `release-plz.toml` now includes obvious support-closure crates that the selected set needs
    - `fret-node` is no longer in the current publish scope
    - `fret-chart` and `delinea` are intentionally included because the supported
      `fret-ui-shadcn/chart` lane stays publishable
    - `python3 tools/release_closure_check.py --config release-plz.toml` now reports
      `internal dependency issues: 0`

- [x] RELFACE-release-210 Confirm tooling-only crates stay out of the minimal library release set.
  - Current non-goal set:
    - `fretboard`
    - `fret-diag`
    - repo-owned harness apps
  - Current progress:
    - `fret-node` is now excluded from `release-plz.toml`
    - `apps/*` crates remain `publish = false`

- [x] RELFACE-release-220 Update release docs/checklists to reflect the selected crate set instead
      of workspace-wide assumptions.
  - Current progress:
    - `docs/release/release-plz-adoption-analysis.md` now distinguishes entry set vs publish
      closure
    - `docs/release/v0.1.0-release-checklist.md` now tracks the zero-blocker closure state
    - `docs/release/v0.1.0-publish-order.txt` now matches the current config
