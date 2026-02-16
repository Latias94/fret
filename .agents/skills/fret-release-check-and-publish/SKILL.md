---
name: fret-release-check-and-publish
description: "This skill should be used when the user asks to \"prepare a release\", \"publish crates\", \"debug release-plz failures\", or \"validate release-plz config/workflows\". Provides a release readiness and publish workflow for Fret using `release-plz` + crates.io (scope selection, dry runs, CI troubleshooting)."
---

# Fret release check and publish

## When to use

Use this skill when:

- Preparing a release wave (`v0.1+`) and deciding what to publish.
- Debugging release-plz PR/publish failures.
- Validating `release-plz.toml` and `.github/workflows/release-plz.yml`.
- Introducing or enforcing a versioning policy (single vs multiple `version_group`s).

## Inputs to collect (ask the user)

Ask these up front so “release scope” is explicit and the run is reversible:

- Release wave: which version(s) and which crate set (what is in-scope vs out-of-scope)?
- Publish strategy: single `version_group` or multiple groups?
- CI mode: are we using `CARGO_REGISTRY_TOKEN` or trusted publishing (`id-token: write`)?
- Preflight expectations: do we need a publish order/closure check, or just validate config?
- Failure context (if debugging): release-pr stage or release/publish stage; what error text?

Defaults if unclear:

- Keep workspace defaults conservative and publish only a small, explicit `[[package]]` whitelist in one `version_group`.

## Smallest starting point (one command)

- `python3 tools/release_closure_check.py --config release-plz.toml --print-publish-commands`

## One-command preflight (recommended)

Use the cross-platform preflight runner to avoid forgetting individual gates:

- `python3 tools/pre_release.py --release-config release-plz.toml`

## Quick intent

- Keep release scope explicit and conservative.
- Validate publishability before opening release PRs.
- Use `release-plz` as the single release automation entry.

## Quick start

1. Decide the publish set in `release-plz.toml` (keep it small).
2. (Recommended) Run a release closure + publish order check:
   - `python3 tools/release_closure_check.py --config release-plz.toml --write-order docs/release/v0.1.0-publish-order.txt --print-publish-commands`
   - Or via one-command preflight:
     - `python3 tools/pre_release.py --release-config release-plz.toml --release-write-order docs/release/v0.1.0-publish-order.txt --release-print-publish-commands`
3. Run planning: `release-plz update --config release-plz.toml --allow-dirty --repo-url <repo-url>`.
4. Optionally run a per-crate dry run (may fail before first-wave dependencies exist on crates.io):
   - `cargo publish --dry-run -p <crate> --allow-dirty --no-verify`

## Workflow

### Repository anchors

- Release config: `release-plz.toml`
- CI workflow: `.github/workflows/release-plz.yml`
- Preflight gate workflow (recommended): `.github/workflows/release-guards.yml`
- Release analysis and scope notes: `docs/release/release-plz-adoption-analysis.md`
- Operational checklist (v0.1): `docs/release/v0.1.0-release-checklist.md`

### Step 1: Define release scope

1. Start from `release-plz.toml` workspace defaults:
   - `release = false`
   - `publish = false`
2. Add only target crates with `[[package]]` entries and set:
   - `release = true`
   - `publish = true`
3. Put wave-aligned crates into one `version_group` (for example `fret-0-1`) to keep versions synchronized.
4. Keep app/demo/tooling crates out of the publish whitelist unless explicitly required.

### Step 1.5: Decide your versioning model (single vs multiple version groups)

Two viable strategies:

- Single group (lockstep): all published crates share the same exact `x.y.z`.
- Multiple groups (independent patch): split into multiple `version_group`s (for example core vs ecosystem) so
  one group can publish `0.y.(z+1)` without bumping the other group.

If you adopt multiple groups, enforce two extra invariants:

- All released crates stay on the same compatibility line (`0.y`) unless the release is intentionally breaking.
- Internal `path` dependencies use a Cargo version requirement that matches the intended policy:
  - allow any patch within the compatibility line: `version = "0.y"`
  - require a minimum patch when using new APIs: `version = "0.y.z"`

The closure checker (`tools/release_closure_check.py`) should be treated as the source of truth for these invariants.

### Step 2: Preflight checks (must pass before CI release)

1. Verify each release crate is publishable:
   - No private/path-only dependency without a valid crates.io version requirement.
   - `Cargo.toml` metadata is complete (`license`, `description`, `repository`, `readme` when applicable).
2. Run focused local dry-runs first:
   - `cargo publish --dry-run -p <crate> --allow-dirty --no-verify`
3. Run release-plz dry planning:
   - `release-plz update --config release-plz.toml --allow-dirty --repo-url <repo-url>`
   - If workspace is large, validate by package first:
     - `release-plz update --config release-plz.toml --allow-dirty --repo-url <repo-url> --package <crate>`
4. For first-wave multi-crate publishes, prefer a closure/order check over isolated dry-runs:
   - `python3 tools/release_closure_check.py --config release-plz.toml --print-publish-commands`

### Step 2.5: SemVer break detection (API-level)

`release-plz` can run an API SemVer break check via `cargo-semver-checks` (controlled by `semver_check` in
`release-plz.toml`).

Important limitations:

- It checks *public Rust API surface*, not runtime behavior/contract semantics.
- It typically reflects the default feature set; feature-gated APIs may not be fully covered.
- It needs a published baseline version to compare against (first publish has limited signal).

### Step 3: CI publish flow

1. Ensure secrets and permissions are ready:
   - `GITHUB_TOKEN` with PR/content permissions.
   - `CARGO_REGISTRY_TOKEN` (or trusted publishing with `id-token: write`).
2. Keep checkout full history in CI (`fetch-depth: 0`).
3. Execute release lifecycle:
   - `release-pr` job opens/updates release PR.
   - Review versions/changelog/release scope.
   - Merge release PR to default branch.
   - `release` job publishes crates and creates tags/releases.

## Definition of done (what to leave behind)

- Minimum deliverables (3-pack): Repro (closure/order), Gate (CI ready), Evidence (planning output). See `fret-skills-playbook`.
- `release-plz.toml` scope is explicit (only intended crates enabled) and wave crates share a `version_group` as intended.
- A closure/order artifact exists for the wave:
  - publish order file (e.g. `docs/release/v0.1.0-publish-order.txt`) and/or printed publish commands.
- `release-plz update ...` planning output is captured (or at least reproducible by command).
- CI is validated against `.github/workflows/release-plz.yml` (canonical repo guard, `fetch-depth: 0`, permissions).
- If publishing: the release PR is merged and the publish job completes (tags/releases visible).

## Common pitfalls

- `all dependencies must have a version requirement specified when publishing`
  - Cause: internal dependency declared as path-only without publishable version requirement.
  - Fix: add explicit semver requirement for publishable crates.

- GitHub API 401/403 in `release-plz release`
  - Cause: missing token scope, fork context, or owner/repo mismatch.
  - Fix: run in canonical repo, verify workflow permissions and secrets.

- Release PR generated but no publish
  - Cause: wrong release conditions or release commit not merged.
  - Fix: verify `release_always`, branch target, and merged release PR state.

- Version drift across crates
  - Cause: missing shared `version_group`.
  - Fix: assign all same-wave crates into one `version_group`.

## Fret v0.1 practice notes

- Keep first wave focused on framework public surfaces and mandatory transitive crates.
- For this repo, `fret-node` and `fret-router` are intentionally included in the `v0.1` wave by product decision.
- Keep release scope and rationale documented in `docs/release/release-plz-adoption-analysis.md`.

## Output checklist for release tasks

- Updated `release-plz.toml` whitelist.
- Updated release analysis doc with include/exclude rationale.
- Local dry-run evidence (`cargo publish --dry-run` or `release-plz update`).
- CI workflow validated (`release-pr` then `release`).

## Evidence anchors (where to look)

- `release-plz.toml`
- `.github/workflows/release-plz.yml`
- `.github/workflows/release-guards.yml`
- `docs/release/release-plz-adoption-analysis.md`
- `docs/release/v0.1.0-release-checklist.md`
- `tools/release_closure_check.py`

## Related skills

- `fret-diag-workflow` (when a release is blocked by a reproducible bug and you need artifacts)
