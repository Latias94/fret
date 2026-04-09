# Release-plz Adoption Analysis (Fret)

## Scope

This note follows the official `release-plz` setup flow and maps it to the current Fret workspace state.

- Official quickstart and config references:
  - `https://release-plz.dev/docs/github/quickstart`
  - `https://release-plz.dev/docs/config`

Date audited: 2026-04-09.

## Workspace facts

- Workspace members include `crates/*`, `ecosystem/*`, and `apps/*`.
- Workspace version is unified at `0.1.0`.
- Workspace MSRV is intentionally `1.92` (aligned with current `wgpu` minimum requirements).
- Most packages currently default to `publish = true` (manifest-level default), while selected apps are already `publish = false`.

## What should be released (v0.1 candidate)

The first publish wave should distinguish between:

- the **user-facing entry set** we actually want to teach, and
- the broader **publish closure** required to keep those entry crates and their supported optional
  lanes valid on crates.io.

User-facing entry set:

- `fret`
- `fret-framework`
- `fret-bootstrap`
- `fret-ui-kit`
- `fret-ui-shadcn`
- `fret-selector`
- `fret-query`
- `fretboard`

Actual publish closure:

- use `release-plz.toml` as the source of truth,
- keep `docs/release/v0.1.0-publish-order.txt` synced from
  `python3 tools/release_closure_check.py --config release-plz.toml --write-order ...`,
- the current closure is `52` crates as of `2026-04-09`,
- the public `fretboard` CLI lands in **Wave 2** because it only closes over `fret-assets`,
- expect the closure to include lower-level runtime/render/platform crates plus explicit optional
  support crates such as:
  - `fret-assets`
  - `fret-router-ui`
  - `fret-webview`
  - `fret-webview-wry`
  - `fret-window-style-profiles`
  - `fret-chart`
  - `delinea`
  - icon-pack / UI-assets support crates used by published optional lanes

## What should NOT be released in v0.1

Reason: app harnesses, tooling, or ecosystem surfaces that are still intentionally outside the
first public teaching story.

- `apps/*` crates (already mostly `publish = false`):
  - `fret-demo`, `fret-demo-web`, `fret-editor`, `fret-examples`, `fretboard-dev`, `fret-ui-gallery`, `fret-ui-gallery-web`, `fret-svg-atlas-stress`

Note:

- The publishable public CLI now lives in `crates/fretboard` (`fretboard` on crates.io).
- The repo-owned maintainer command surface remains in `apps/fretboard` as package `fretboard-dev`.
- Deferred ecosystem/editor/design-system crates:
  - `fret-docking`
  - `fret-code-editor*`
  - `fret-code-view`
  - `fret-markdown`
  - `fret-ui-editor`
  - `fret-ui-material3`
  - `fret-ui-ai`
  - `fret-undo`
  - `fret-gizmo`
  - `fret-workspace`
  - `fret-node`

Note: some crates in this list are technically publishable, but they are intentionally excluded
from the first public teaching surface until the owning story is stable enough to support.

## Important precondition before first publish

Current release closure is now mechanically closed:

- `python3 tools/release_closure_check.py --config release-plz.toml` reports
  `release scope: 52 crates`
- `python3 tools/release_closure_check.py --config release-plz.toml` reports
  `internal dependency issues: 0`
- `metadata warnings: 0`
- the publish order is deterministic and captured in
  `docs/release/v0.1.0-publish-order.txt`

Important note:

- `fret-chart` and `delinea` are intentionally in the publish closure because the supported
  `fret-ui-shadcn/chart` lane remains publishable.
- `fret-docking` stays out of the publish closure because docking now lives only on the owning
  crate path and is no longer proxied through `fret`.

## Adopted release-plz strategy

Implemented files:

- `release-plz.toml`
- `.github/workflows/release-plz.yml`

Strategy summary:

- Workspace defaults to `release = false` and `publish = false`.
- Only selected crates are enabled via `[[package]]` with `release = true` and `publish = true`.
- A single `version_group = "fret-0-1"` keeps all published crates on one version line.
- `release_always = false` to publish only when release PR merge semantics are satisfied.
- `release` and `release-pr` are split into separate jobs in CI.
- Changelog policy is repository-level (`CHANGELOG.md`) plus GitHub Release notes (no per-crate changelog files in v0.1).

## CI / secrets requirements

From the official flow:

1. Ensure GitHub Actions has permission to create PRs.
2. Add `CARGO_REGISTRY_TOKEN` repository secret (or switch to trusted publishing with `id-token: write`).
3. Keep `fetch-depth: 0` in checkout.

Current workflow already includes:

- `fetch-depth: 0`
- job-level permissions
- `config: release-plz.toml`

## Recommended next steps

1. Keep internal workspace `path` dependencies in the release whitelist on explicit
   `version = "0.1.0"` requirements.
2. Run a full local dry run:
   - `release-plz update --config release-plz.toml --repo-url https://github.com/Latias94/fret`
3. Run targeted `cargo publish --dry-run -p <crate>` checks for a few high-level crates in publish
   order context (`fret`, `fret-ui-shadcn`, `fret-bootstrap`) to validate manifest packaging.
4. After dry-run is clean, enable the workflow on `main` and test with `workflow_dispatch` first.
