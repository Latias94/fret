# Release-plz Adoption Analysis (Fret)

## Scope

This note follows the official `release-plz` setup flow and maps it to the current Fret workspace state.

- Official quickstart and config references:
  - `https://release-plz.dev/docs/github/quickstart`
  - `https://release-plz.dev/docs/config`

Date audited: 2026-02-08.

## Workspace facts

- Workspace members include `crates/*`, `ecosystem/*`, and `apps/*`.
- Workspace version is unified at `0.1.0`.
- Workspace MSRV is intentionally `1.92` (aligned with current `wgpu` minimum requirements).
- Most packages currently default to `publish = true` (manifest-level default), while selected apps are already `publish = false`.

## What should be released (v0.1 candidate)

The first publish set should be limited to the public framework and its required internal dependencies.

Primary public surfaces:

- `fret`
- `fret-ui-kit`
- `fret-ui-shadcn`
- `fret-selector`

Required transitive workspace crates for those surfaces:

- `fret-a11y-accesskit`
- `fret-app`
- `fret-authoring`
- `fret-canvas`
- `fret-core`
- `fret-dnd`
- `fret-executor`
- `fret-fonts`
- `fret-i18n`
- `fret-icons`
- `fret-launch`
- `fret-node`
- `fret-platform`
- `fret-platform-native`
- `fret-platform-web`
- `fret-query`
- `fret-render`
- `fret-render-core`
- `fret-render-wgpu`
- `fret-router`
- `fret-runner-web`
- `fret-runner-winit`
- `fret-runtime`
- `fret-ui`
- `fret-ui-headless`
- `fret-ui-app`
- `fret-viewport-tooling`

## What should NOT be released in v0.1

Reason: app harnesses, incubating editor modules, diagnostics tools, or crates outside the minimal public story.

- `apps/*` crates (already mostly `publish = false`):
  - `fret-demo`, `fret-demo-web`, `fret-editor`, `fret-examples`, `fretboard`, `fret-ui-gallery`, `fret-ui-gallery-web`, `fret-svg-atlas-stress`
- Incubating ecosystem/editor crates:
  - `fret-docking`, `fret-code-editor*`, `fret-code-view`, `fret-markdown`, `fret-plot*`, `fret-chart`, `delinea`, `fret-ui-material3`, `fret-ui-ai`, `fret-undo`, `fret-gizmo`, `fret-workspace`, `fret-renderdoc`, `fret-asset-cache`, `fret-bootstrap`, `fret-ui-assets`, `fret-i18n-fluent`, `fret-icons-lucide`, `fret-icons-radix`

Special note for `v0.1`:

- `fret-node` and `fret-router` are included intentionally for the first public wave by product decision, even though they are still evolving.

Current recommendation for v0.1:

- Keep `fret-kit` out of the first publish wave.

Reason:

- `fret-kit` currently depends on `fret-bootstrap` and related integration crates (`fret-ui-assets`, icon packs),
  which are intentionally excluded from the narrow 0.1 release footprint.

Note: some crates in this list are technically publishable, but excluded intentionally for a narrow and stable 0.1 release footprint.

## Important precondition before first publish

Current manifests rely on path-only workspace dependencies (no explicit version requirements for many internal deps).

This causes `cargo publish --dry-run` to fail before release:

- Example: `fret` currently fails with
  - `all dependencies must have a version requirement specified when publishing`

Therefore, before enabling the release pipeline, all crates in the release set need publishable dependency declarations (path + version strategy compatible with crates.io).

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

1. (Done in this branch) Convert release-set manifests to publishable dependency version declarations.
   - Internal workspace `path` dependencies in the current release whitelist now include explicit `version = "0.1.0"`.
2. Mark all non-release crates explicitly as `publish = false` to avoid accidental publish attempts.
3. Run a full local dry run:
   - `release-plz update --config release-plz.toml --repo-url https://github.com/Latias94/fret`
4. Decide whether `fret-kit` is in wave-1 or wave-2:
   - wave-1 => include `fret-bootstrap`/assets/icon crates in release scope;
   - wave-2 => keep `fret-kit` private until integration crates are ready.
4. After dry-run is clean, enable the workflow on `main` and test with `workflow_dispatch` first.
