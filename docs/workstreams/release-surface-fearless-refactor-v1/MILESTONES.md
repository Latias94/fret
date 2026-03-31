# Release Surface Fearless Refactor v1 (Milestones)

## Status summary

- `M0` Baseline and blocker identification: **Completed**
- `M1` Publish blocker removal: **Completed**
- `M2` Closure slimming: **Completed**
- `M3` Release candidate freeze: **Completed**

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

**Status:** Completed

**Current landed slice**

- removed `workspace_shell` from the `fret` public root surface,
- removed the direct `fret-workspace` dependency from `fret`,
- promoted `fret-router-ui` to a publishable thin adoption crate,
- verified `fret` with router feature enabled and confirmed its direct deps no longer include
  `publish = false` crates.

**Evidence**

- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/Cargo.toml`
- `ecosystem/fret-router-ui/Cargo.toml`

## M2 — Closure slimming

**Status:** Completed

**Current landed slice**

- `fret-ui-shadcn` moved chart recipes behind an explicit `chart` feature instead of pulling
  `fret-chart` into the default closure.
- `fret-ui-shadcn` moved executor-backed async recipe helpers behind an explicit
  `executor-integration` feature instead of pulling `fret-executor` into the default closure.
- `fret-ui-gallery` now opts into that feature only on `gallery-chart` / `gallery-dev`.
- `fret-bootstrap` no longer binds command palette capability directly to `fret-ui-shadcn`;
  the default shadcn command palette UI now sits on explicit
  `fret-bootstrap/ui-app-command-palette-shadcn`.
- `fret`'s `command-palette` feature now maps directly to that explicit bootstrap feature.
- `fret-bootstrap/diagnostics` no longer pulls an unused `fret-query` edge into the diagnostics
  closure.
- retained canvas cache diagnostics now sit behind explicit
  `fret-bootstrap/diagnostics-canvas` instead of bloating the base diagnostics lane.
- devtools WS transport now stays on explicit `fret-bootstrap/diagnostics-ws` instead of requiring
  a separate bridge crate.
- Wave 2 now treats `fret-bootstrap` as a feature-first app-kit crate rather than splitting every
  optional integration into its own published bridge.
- `fret` root no longer teaches maintainer/niche lanes as first-class release surfaces:
  `devloop` and `tracing` now stay documented as advanced aliases, while `material3` and `ui-ai`
  now stay as discoverability-only aliases and no longer proxy their owning crates into the
  release closure.
- `fret` root no longer proxies editor theming replay through an `editor` feature; editor policy
  now stays entirely on `fret-ui-editor`, and first-party examples depend on that crate directly.
- `fret` root no longer proxies docking through a `docking` feature; first-party examples and the
  cookbook now depend on `fret-docking` directly, matching the owning policy crate boundary.

**Evidence**

- `ecosystem/fret-ui-shadcn/Cargo.toml`
- `ecosystem/fret-ui-shadcn/src/lib.rs`
- `ecosystem/fret-ui-shadcn/src/sonner.rs`
- `apps/fret-ui-gallery/Cargo.toml`
- `ecosystem/fret-bootstrap/Cargo.toml`
- `ecosystem/fret-bootstrap/src/lib.rs`
- `ecosystem/fret-bootstrap/src/ui_app_driver.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics_ws_bridge.rs`
- `ecosystem/fret/Cargo.toml`
- `ecosystem/fret/README.md`
- `ecosystem/fret/src/lib.rs`
- `apps/fret-examples/Cargo.toml`
- `apps/fret-cookbook/Cargo.toml`
- `cargo check -p fret-bootstrap --features ui-app-command-palette-shadcn`
- `cargo check -p fret-bootstrap --features diagnostics`
- `cargo check -p fret-bootstrap --features diagnostics-canvas`
- `cargo check -p fret-bootstrap --features diagnostics-ws`
- `cargo check -p fret-examples`
- `cargo check -p fret-cookbook --features cookbook-docking`

## M3 — Release candidate freeze

**Status:** Completed

**Current landed slice**

- `release-plz.toml` now includes obvious support-closure crates that were missing from the publish
  whitelist (`fret-assets`, `fret-router-ui`, `fret-webview`, `fret-webview-wry`,
  `fret-window-style-profiles`).
- `fret-node` was removed from the current release-plz scope because it is not part of the first
  public teaching surface or required support closure.
- `fret-chart` and `delinea` are now intentionally part of the publish closure because the
  supported `fret-ui-shadcn/chart` lane remains publishable.
- `docs/release/v0.1.0-publish-order.txt` is now regenerated from the current `release-plz.toml`
  state.
- `python3 tools/release_closure_check.py --config release-plz.toml` now reports zero internal
  dependency issues and zero metadata warnings.

**Evidence**

- `release-plz.toml`
- `docs/release/v0.1.0-publish-order.txt`
- `docs/release/release-plz-adoption-analysis.md`
- `docs/release/v0.1.0-release-checklist.md`
- `python3 tools/release_closure_check.py --config release-plz.toml`
