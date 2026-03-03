---
title: Diag Migration Support (Runbook + Guardrails)
status: draft
date: 2026-02-28
scope: diagnostics, fearless-refactor, migration
---

# Migration support (fearless refactor)

This runbook exists so we can keep moving fast (taxonomy refactors, schema upgrades, transport changes) without breaking
day-to-day debugging workflows.

## Avoid output explosions

Do not `rg` a run artifact like `bundle.json` (or search under `target/fret-diag/**` / `.fret/diag/**`).

Prefer bounded tooling:

- `fretboard diag meta ...`
- `fretboard diag query ...`
- `fretboard diag slice ...`

For repository-wide search, prefer `tools/rg-safe.ps1` (excludes diag artifact directories).

## Script schema: v1 -> v2

Policy:

- Canonical scripts should be schema v2.
- Redirect stubs (`kind: script_redirect`) are allowed to remain schema v1 (tooling-only).
- Tool-launched runs (`--launch` / `--reuse-launch`) are v2-only.

Runbook:

1) Find v1 scripts (promoted/canonical only):

- `cargo run -p fretboard -- diag doctor scripts`

2) Upgrade in-place (review diffs):

- `cargo run -p fretboard -- diag script upgrade --write tools/diag-scripts/**.json`

3) Normalize to keep diffs stable:

- `cargo run -p fretboard -- diag script normalize --write tools/diag-scripts/**.json`

## Script library taxonomy moves

The canonical script library is taxonomy-based (product area + intent). Use redirects to keep path moves safe.

1) Dry-run a plan:

- `python tools/diag-scripts/migrate-script-library.py`

2) Apply moves (recommended) and write redirect stubs:

- `python tools/diag-scripts/migrate-script-library.py --apply --write-redirects`
  - Note: suite manifests (`tools/diag-scripts/suites/**/suite.json`) are rewritten to point at the canonical
    (post-move) paths.

3) Validate suite closure + registry drift:

- `cargo run -p fretboard -- diag doctor scripts`
- `cargo run -p fretboard -- diag registry check`

## Suites

Suites are curated directory inputs:

- `tools/diag-scripts/suites/<suite-name>/suite.json` is a tooling-only suite manifest.
  (Legacy `script_redirect` suite stubs are still supported by tooling, but not preferred in-tree.)

Discoverability:

- `cargo run -p fretboard -- diag list suites`
- `cargo run -p fretboard -- diag list scripts --contains <needle>`

## Screenshots (PNG)

`capture_screenshot` is capability-gated and disabled by default:

- Enable: `FRET_DIAG_GPU_SCREENSHOTS=1` (or config `screenshots_enabled=true`)
- Capability: `diag.screenshot_png`

If a script includes `capture_screenshot`, tooling infers the capability and fails fast when it is missing.

## Bundle artifacts (raw vs compact)

For small-by-default artifacts (recommended for automation / AI loops), prefer:

- `write_bundle_json=false` (omit the large raw `bundle.json`)
- `write_bundle_schema2=true` (write `bundle.schema2.json` + sidecars)

Tool-launched runs (`fretboard diag ... --launch`) typically write these defaults via `diag.config.json`.
If `diag.config.json` cannot be written (permissions / invalid `--dir`), treat it as a tooling/launch error rather than
silently falling back to runtime defaults.
