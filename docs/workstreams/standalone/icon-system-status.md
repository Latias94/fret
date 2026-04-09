# Icon System Status

Status: Current execution stance
Last updated: 2026-04-09

This note is the first-stop execution map for the closed workstream cluster around
`docs/adr/0065-icon-system-and-asset-packaging.md`.

Use it to answer:

- which icon lanes are already closed,
- which lane is the right predecessor for a new follow-on,
- and which kinds of future work should stay separated instead of reopening a broad historical
  lane.

## Assumptions-first synthesis

### 1) The base icon contract lane is already closed

- Assumption: `icon-system-extension-v1` is the shipped base contract lane and should not be
  reopened for narrow follow-on work.
- Evidence:
  - `docs/workstreams/icon-system-extension-v1/WORKSTREAM.json`
  - `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`
- Confidence: Confident
- Consequence if wrong: later narrow fixes would blur the boundary between the v1 contract and
  branch-specific follow-ons.

### 2) Install/diagnostics work now has its own closed chain

- Assumption: explicit install semantics and startup failure reporting now belong to a closed
  follow-on chain:
  `icon-install-health-hardening-v1` ->
  `icon-install-error-reporting-v1` ->
  `bootstrap-known-startup-failure-taxonomy-v1`.
- Evidence:
  - `docs/workstreams/icon-install-health-hardening-v1/WORKSTREAM.json`
  - `docs/workstreams/icon-install-error-reporting-v1/WORKSTREAM.json`
  - `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/WORKSTREAM.json`
- Confidence: Confident
- Consequence if wrong: future startup/recovery work would be attached to the wrong lane and
  re-open already-closed semantics.

### 3) Import/generator/presentation work now has its own closed chain

- Assumption: third-party import and presentation-default work belongs to a separate closed chain:
  `iconify-import-pack-generator-v1` ->
  `iconify-acquisition-prestep-v1` ->
  `generated-icon-presentation-defaults-v1` ->
  `iconify-presentation-defaults-suggestion-v1` ->
  `iconify-presentation-defaults-report-v1` ->
  `svg-presentation-analysis-scaffolding-v1`.
- Evidence:
  - each lane's `WORKSTREAM.json` under `docs/workstreams/*/`
- Confidence: Confident
- Consequence if wrong: runtime/bootstrap concerns and import-time heuristics would get mixed into
  one unrecoverable bucket.

### 4) Future work should branch from the latest relevant closed lane, not from the oldest broad one

- Assumption: the right continuation rule is “start from the narrowest latest predecessor that
  already owns the problem”.
- Evidence:
  - `continue_policy` notes in the icon-related `WORKSTREAM.json` files
  - `docs/workstreams/standalone/workstream-state-v1.md`
- Confidence: Likely
- Consequence if wrong: roadmap/index notes would keep accumulating history without giving a clear
  continuation rule.

## Closed lane map

### Base contract

- `icon-system-extension-v1`
  - closes the v1 icon contract:
    semantic IDs, pack metadata/provenance, multicolor authoring surface, and third-party pack
    protocol.

### Install and startup reporting branch

- `icon-install-health-hardening-v1`
  - closes fail-fast explicit install semantics vs best-effort helper fallback.
- `icon-install-error-reporting-v1`
  - closes known icon-install failure reports plus diagnostics-aware panic-time reporting.
- `bootstrap-known-startup-failure-taxonomy-v1`
  - closes the broader bootstrap-level taxonomy that unifies returned startup failures with
    panic-only explicit icon install failures.

### Third-party import and presentation branch

- `iconify-import-pack-generator-v1`
  - closes the reusable local-input generator contract and public CLI.
- `iconify-acquisition-prestep-v1`
  - closes subset-first remote/pinned acquisition as a separate pre-step.
- `generated-icon-presentation-defaults-v1`
  - closes explicit versioned generated-pack presentation defaults.
- `iconify-presentation-defaults-suggestion-v1`
  - closes the thin provenance-driven suggestion helper.
- `iconify-presentation-defaults-report-v1`
  - closes optional versioned review-report output.
- `svg-presentation-analysis-scaffolding-v1`
  - closes conservative local-SVG override scaffolding without changing import defaults.

## Current execution stance on 2026-04-09

- Keep `icon-system-extension-v1` closed as the base contract lane. Reopen it only if fresh
  evidence changes the core icon contract itself, not for narrow bootstrap or generator follow-ons.
- Keep install/runtime/bootstrap startup work on the latest reporting branch. If future work is
  about startup recovery UI, persistent startup diagnostics artifacts, or richer failure handling,
  start a new narrow follow-on from `bootstrap-known-startup-failure-taxonomy-v1`.
- Keep import/generator/presentation-default work on the generator branch. If future work is about
  acquisition provenance, advisory heuristics, review artifacts, or local SVG override scaffolds,
  start from `svg-presentation-analysis-scaffolding-v1` or the nearest narrower predecessor.
- Do not reopen generator lanes for runtime/bootstrap semantics, and do not reopen
  bootstrap/reporting lanes for import-time heuristics. Those branches are intentionally separate.
- If a future request is “pack-specific parity” rather than startup handling or generator
  heuristics, start a new narrow follow-on directly from `icon-system-extension-v1`.

## Shortcut: where should the next follow-on start?

- Problem: startup recovery UI, persistent startup error bundles, richer bootstrap diagnostics
  behavior.
  Start from: `bootstrap-known-startup-failure-taxonomy-v1`
- Problem: import-time provenance, acquisition shape, generated config, advisory presentation
  defaults, local SVG override scaffolding.
  Start from: `svg-presentation-analysis-scaffolding-v1`
- Problem: runtime icon contract, semantic aliasing policy, pack metadata/provenance contract, or a
  new pack-family parity gap.
  Start from: `icon-system-extension-v1`

## Evidence anchors

- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `docs/workstreams/icon-system-extension-v1/WORKSTREAM.json`
- `docs/workstreams/icon-install-health-hardening-v1/WORKSTREAM.json`
- `docs/workstreams/icon-install-error-reporting-v1/WORKSTREAM.json`
- `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/WORKSTREAM.json`
- `docs/workstreams/iconify-import-pack-generator-v1/WORKSTREAM.json`
- `docs/workstreams/iconify-acquisition-prestep-v1/WORKSTREAM.json`
- `docs/workstreams/generated-icon-presentation-defaults-v1/WORKSTREAM.json`
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/WORKSTREAM.json`
- `docs/workstreams/iconify-presentation-defaults-report-v1/WORKSTREAM.json`
- `docs/workstreams/svg-presentation-analysis-scaffolding-v1/WORKSTREAM.json`
- `docs/workstreams/standalone/workstream-state-v1.md`
