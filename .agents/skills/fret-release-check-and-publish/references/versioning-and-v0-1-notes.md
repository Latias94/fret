# Versioning and v0.1 release notes

Use this note when the hard part is not the command itself, but the release policy.

## 1) Decide your versioning model

Two viable strategies:

- Single group (lockstep): all published crates share the same exact `x.y.z`.
- Multiple groups (independent patch): split into multiple `version_group`s so one group can publish `0.y.(z+1)` without bumping the other group.

If you adopt multiple groups, enforce two extra invariants:

- all released crates stay on the same compatibility line (`0.y`) unless the release is intentionally breaking
- internal `path` dependencies use a Cargo version requirement that matches the intended policy:
  - allow any patch within the compatibility line: `version = "0.y"`
  - require a minimum patch when using new APIs: `version = "0.y.z"`

Treat `tools/release_closure_check.py` as the source of truth for these invariants.

## 2) Fret v0.1 practice notes

- Keep the first wave focused on framework public surfaces and mandatory transitive crates.
- For this repo, `fret-node` and `fret-router` are intentionally included in the `v0.1` wave by product decision.
- Keep release scope and rationale documented in `docs/release/release-plz-adoption-analysis.md`.

## 3) Output checklist for release tasks

Leave behind:

- updated `release-plz.toml` whitelist
- updated release analysis doc with include/exclude rationale
- local dry-run evidence (`cargo publish --dry-run` or `release-plz update`)
- CI workflow validated (`release-pr` then `release`)
- publish-order output or reproducible closure/order command

## 4) Common failure signatures

- `all dependencies must have a version requirement specified when publishing`
  - cause: internal dependency declared as path-only without publishable version requirement
  - fix: add explicit semver requirement for publishable crates
- GitHub API 401/403 in `release-plz release`
  - cause: missing token scope, fork context, or owner/repo mismatch
  - fix: run in canonical repo and verify workflow permissions and secrets
- Release PR generated but no publish
  - cause: wrong release conditions or release commit not merged
  - fix: verify `release_always`, branch target, and merged release PR state
- Version drift across crates
  - cause: missing shared `version_group`
  - fix: assign all same-wave crates into one `version_group`
