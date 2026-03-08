# Release preflight checklist

Use this note before changing release config or rerunning `release-plz` CI.

## 1) Repository anchors

Check these first:

- `release-plz.toml`
- `.github/workflows/release-plz.yml`
- `.github/workflows/release-guards.yml`
- `docs/release/release-plz-adoption-analysis.md`
- `docs/release/v0.1.0-release-checklist.md`
- `tools/release_closure_check.py`

## 2) Define release scope

Before any dry-run:

1. Decide whether the task is a single crate, a crate group, or a full release wave.
2. Verify the intended crates are explicitly enabled in `release-plz.toml`.
3. If several crates ship together, group them intentionally.
4. Keep app/demo/tooling crates out of the publish whitelist unless explicitly required.

## 3) Preflight checks (must pass before CI release)

1. Verify each release crate is publishable:
   - no private/path-only dependency without a valid crates.io version requirement
   - `Cargo.toml` metadata is complete (`license`, `description`, `repository`, `readme` when applicable)
2. Run focused local dry-runs first:
   - `cargo publish --dry-run -p <crate> --allow-dirty --no-verify`
3. Run release-plz dry planning:
   - `release-plz update --config release-plz.toml --allow-dirty --repo-url <repo-url>`
   - if the workspace is large, validate by package first:
     - `release-plz update --config release-plz.toml --allow-dirty --repo-url <repo-url> --package <crate>`
4. For first-wave multi-crate publishes, prefer a closure/order check over isolated dry-runs:
   - `python3 tools/release_closure_check.py --config release-plz.toml --print-publish-commands`

## 4) SemVer break detection (API-level)

`release-plz` can run an API SemVer break check via `cargo-semver-checks` (controlled by `semver_check` in `release-plz.toml`).

Important limitations:

- it checks public Rust API surface, not runtime behavior/contract semantics
- it typically reflects the default feature set; feature-gated APIs may not be fully covered
- it needs a published baseline version to compare against; first publish has limited signal

## 5) CI publish flow

1. Ensure secrets and permissions are ready:
   - `GITHUB_TOKEN` with PR/content permissions
   - `CARGO_REGISTRY_TOKEN` or trusted publishing with `id-token: write`
2. Keep checkout full history in CI (`fetch-depth: 0`).
3. Execute the release lifecycle:
   - `release-pr` job opens/updates the release PR
   - review versions/changelog/release scope
   - merge the release PR to the default branch
   - `release` job publishes crates and creates tags/releases
