---
name: fret-release-check-and-publish
description: 'This skill should be used when the user asks to "prepare a release", "publish crates", "debug release-plz failures", or "validate release-plz config/workflows". Provides a release readiness and publish workflow for Fret using `release-plz` + crates.io (scope selection, dry runs, CI troubleshooting).'
---

# Fret release check and publish

Use this skill when the task is about preparing, validating, or troubleshooting a Fret release wave.

## When to use

- You are preparing a new workspace release wave.
- `release-plz` is failing in CI and you need to find the blocking issue.
- You need to check which crates should be in scope for publishing.
- You need to validate `release-plz.toml`, version groups, or release workflow permissions.

## Inputs to collect (ask the user)

- Which release scope is intended (single crate, crate group, full wave)?
- Is this a dry-run/preflight, or are we actually trying to publish?
- Are versions expected to move in lockstep, or should there be multiple version groups?
- Which CI/workflow run is failing, and what is the exact error surface?
- Do we need local evidence artifacts (publish order, planning output, dry-run commands)?

Defaults if unclear:

- Start with a dry-run/preflight and make release scope explicit before touching config.

## Smallest starting point (one command)

- `cargo run -p release-plz -- --help`

## Quick start

1. Read the relevant reference note first.
2. Make release scope explicit (`release-plz.toml`, publish whitelist, wave members).
3. Run local preflight and closure/order checks before blaming CI.
4. Capture planning output and workflow evidence so the failure is reproducible.

## Workflow

### 0) Read the relevant reference note first

Use these notes to keep the main skill lean:

- Preflight commands, CI flow, and SemVer check boundaries:
  - `.agents/skills/fret-release-check-and-publish/references/release-preflight-checklist.md`
- Version-group strategy, v0.1 practice notes, and release-task outputs:
  - `.agents/skills/fret-release-check-and-publish/references/versioning-and-v0-1-notes.md`

### 1) Define release scope explicitly

Before changing config or rerunning CI:

- decide which crates are in scope,
- confirm whether they should publish together,
- keep apps/demos/tooling out of the publish wave unless intentionally required.

### 2) Run local preflight before CI

At minimum, validate:

- publishable manifests,
- dependency version requirements,
- closure/order of the release wave,
- `release-plz` planning output.

### 3) Treat CI as execution, not discovery

By the time CI runs, you should already know:

- the intended release scope,
- the expected publish order,
- the version-group policy,
- the likely failure surface if credentials or permissions are wrong.

### 4) Leave bounded release evidence

Keep the release task reviewable by leaving:

- exact commands used,
- planning output or reproducible planning command,
- publish-order evidence,
- the workflow/config files that define release behavior.

## Definition of done (what to leave behind)

- Minimum deliverables (3-pack): Repro (closure/order), Gate (CI-ready config), Evidence (planning output). See `fret-skills-playbook`.
- `release-plz.toml` scope is explicit and version-group intent is documented.
- A closure/order artifact exists for the wave.
- CI workflow expectations are validated against the canonical release workflows.
- If publishing: the release PR is merged and the publish job completes.

## Examples

- Example: debug a `release-plz` failure
  - User says: "release-plz failed—what do we fix?"
  - Actions: run preflight, identify manifest/version-group/workflow issues, and verify publish permissions.
  - Result: a clear fix list before re-running CI.

## Common pitfalls

- Treating CI as the first place to discover release scope.
- Publishing crates with path-only internal dependencies and no crates.io version requirement.
- Letting version groups drift without writing the intended policy down.
- Mixing demos/apps/tooling into the publish whitelist by accident.

## Troubleshooting

- Symptom: publish fails due to missing credentials.
  - Fix: confirm CI secrets/permissions first, then rerun bounded preflight.
- Symptom: workspace versioning becomes inconsistent.
  - Fix: use the configured version-group rules; avoid ad-hoc per-crate bumps.

## Evidence anchors

- `release-plz.toml`
- `.github/workflows/release-plz.yml`
- `.github/workflows/release-guards.yml`
- `docs/release/release-plz-adoption-analysis.md`
- `docs/release/v0.1.0-release-checklist.md`
- `tools/release_closure_check.py`
- This skill’s references:
  - `.agents/skills/fret-release-check-and-publish/references/release-preflight-checklist.md`
  - `.agents/skills/fret-release-check-and-publish/references/versioning-and-v0-1-notes.md`

## Related skills

- `fret-framework-maintainer-guide`
- `fret-diag-workflow`
- `fret-boundary-checks`
