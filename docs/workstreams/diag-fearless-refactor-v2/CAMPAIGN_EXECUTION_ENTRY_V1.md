# Diag Fearless Refactor v2 - Campaign Execution Entry v1

Status: Draft

Tracking doc: `docs/workstreams/diag-fearless-refactor-v2/README.md`

Related notes:

- `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_CAMPAIGN_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_SUMMARY_SCHEMA_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/IMPLEMENTATION_ROADMAP.md`

## 0) Purpose

This note proposes the first practical execution entry for campaign-oriented diagnostics automation.

## Implementation status (2026-03-06)

The first minimal implementation is now landed in `crates/fret-diag`.

Current shipped surface:

- `fretboard diag campaign list`
- `fretboard diag campaign show <campaign_id>`
- `fretboard diag campaign run <campaign_id>`
- `fretboard diag campaign run --lane <lane> --tag <tag> --platform <platform>`

Current shipped behavior:

- a workspace-backed registry now resolves campaign ids from `tools/diag-campaigns/*.json`, with built-in definitions as fallback,
- campaign `run` now expands to suites and direct script items,
- each run writes under `campaigns/<campaign_id>/<run_id>/`,
- filtered or multi-id runs that resolve to more than one campaign also persist a batch root under
  `campaign-batches/<selection_slug>/<run_id>/`,
- `diag campaign share <campaign_or_batch_root>` can now generate bounded AI-only share zips under
  `<root>/share/` plus a `share.manifest.json` handoff file,
- failed `diag campaign run` executions now also best-effort generate `share/share.manifest.json`
  automatically for campaign and batch roots when aggregate summaries exist,
- share manifest entries now also expose best-effort `triage.json` paths when bundle artifacts are
  present,
- share manifest entries now also expose `screenshots_manifest` when screenshot evidence exists,
- share roots now also best-effort emit one `combined-failures.zip` bundle for handoff,
- suite runs reuse the existing `diag suite` implementation,
- aggregate handoff reuses the existing `diag summarize` implementation,
- the final artifact contract remains `regression.index.json` + `regression.summary.json`.

Known gaps after the first landing:

- canonical manifests now use one ordered `items` list; legacy top-level `suites` / `scripts` is still accepted for compatibility,
- direct script-item support now exists and execution follows ordered `items`,
- no persisted dashboard text/HTML projection yet,
- no richer campaign metadata resolver beyond the workspace manifest registry yet,
- cross-suite launch reuse still follows current `diag suite` behavior rather than a campaign-level runner.

The intent is not to replace existing commands such as:

- `diag run`
- `diag suite`
- `diag repeat`
- `diag matrix`
- `diag perf`
- `diag summarize`
- `diag dashboard`

Instead, the goal is to add one higher-level entry that composes those primitives into a stable maintainer and CI workflow.

## 1) Current baseline

Today the repo already has:

- a strong single-script execution path via `diag run`,
- a named multi-script execution path via `diag suite`,
- aggregate artifact consumers via `diag summarize` and `diag dashboard`,
- DevTools and MCP consumers that can read `regression.index.json` and `regression.summary.json`.

What is still missing is one command that answers all of these together:

- what should run for a named regression lane,
- where the outputs should live,
- how evidence should be left behind,
- what the final aggregate handoff directory is,
- and what DevTools or CI should open next.

## 2) Recommendation

Recommendation: add a first-class campaign entry under `fretboard diag campaign`.

Suggested initial command family:

- `fretboard diag campaign run <campaign_id>`
- `fretboard diag campaign list`
- `fretboard diag campaign show <campaign_id>`

Only `run` is required for the first landing slice.

`list` and `show` are recommended because they make the system self-describing for maintainers and CI authors.

## 3) Why a new top-level entry is justified

### 3.1 Why not overload `diag suite`

`diag suite` is already a useful primitive, but it does not naturally own:

- lane semantics,
- retry and flake policy,
- campaign-level output directory conventions,
- multi-suite orchestration,
- final aggregate summary/index generation,
- campaign metadata discovery for CLI, DevTools, and CI.

Overloading `diag suite` until it becomes campaign-like would blur two distinct concepts:

- suite = collection of scripts
- campaign = orchestration policy over suites/scripts plus output expectations

### 3.2 Why not make GUI the orchestration layer

DevTools should stay a consumer and trigger surface, not become the source of truth for regression orchestration.

The CLI entry should remain the canonical path, with DevTools and MCP consuming its outputs.

## 4) First-slice scope

The first slice should intentionally stay small.

### In scope

- define the CLI entry shape,
- define how campaign ids resolve to suites/scripts,
- define the output directory layout,
- define the final aggregate artifacts left behind,
- define the minimum metadata needed for list/show/run,
- make the resulting directory directly usable by `diag dashboard`, `diag summarize`, DevTools, and MCP.

### Out of scope for v1

- a full campaign authoring DSL,
- CI-specific remote storage rules,
- advanced flake quarantine UI,
- a new artifact schema beyond the current aggregate outputs,
- replacing existing suite definitions.

## 5) Recommended command shape

## 5.1 Canonical entry

Recommended first command:

- `fretboard diag campaign run <campaign_id> [--dir <dir>] [--lane <lane>] [--json] [--launch -- <cmd...>]`

Behavior:

1. resolve `<campaign_id>` to a campaign definition,
2. expand that campaign into one or more suites and/or direct script items,
3. run the expanded items using existing primitives,
4. collect per-item outputs,
5. emit aggregate artifacts in one predictable campaign run directory,
6. print a concise summary and the final artifact root.

## 5.2 Optional helper commands

Recommended but not required for slice 1:

- `fretboard diag campaign list`
  - lists known campaign ids, lanes, and a short description.
- `fretboard diag campaign show <campaign_id>`
  - prints the expanded suites/scripts, expected lane, and default evidence profile.

These commands reduce friction substantially and keep campaign definitions discoverable.

## 6) Campaign definition model

For the first slice, prefer a minimal in-repo definition model rather than a large external DSL.

Recommendation:

- keep campaign definitions in a small repo-owned manifest layer,
- start with a format easy to diff and review,
- allow only fields needed by run/list/show.

Recommended definition fields:

- `id`
- `description`
- `default_lane`
- `items`
  - suite ids and/or explicit script paths
- `default_launch`
  - optional launch command template or expected launch mode
- `retry_policy`
  - campaign-level default only
- `evidence_profile`
  - bounded / with_pack / with_screenshots / perf_heavy
- `tags`

Important constraint:

- item-level execution should still reuse existing suite/script primitives rather than duplicating their behavior in a second engine.

## 7) Recommended output layout

The most important design choice is the run directory layout.

Recommendation:

- place campaign runs under a stable root such as:
  - `<base_dir>/campaigns/<campaign_id>/<run_id>/`
- when one `diag campaign run` selects multiple campaigns, also persist:
  - `<base_dir>/campaign-batches/<selection_slug>/<run_id>/`

Suggested contents:

- `campaign.manifest.json`
  - resolved campaign metadata actually used for this run
- `campaign.result.json`
  - campaign-level execution result summary
- `suite-results/`
  - one subdirectory or result file per suite invocation
- `regression.index.json`
  - aggregate consumer index for DevTools/MCP/CLI dashboard
- `regression.summary.json`
  - aggregate merged summary
- `batch.manifest.json`
  - persisted selection metadata for one multi-campaign execution batch
- `batch.result.json`
  - batch-level counters plus stable links back to each per-campaign run root
- `dashboard.txt` or equivalent human-readable dashboard output
- `bundles/` or per-item evidence paths
  - only where produced by the underlying run/suite flow

The key property is not the exact names of every side file.

The key property is this:

- one campaign run directory must be enough for a maintainer to
  - inspect results,
  - open DevTools against it,
  - run `diag dashboard --dir ...`,
  - and copy/share failing evidence.

## 8) Consumer contract

The campaign entry should not invent a new consumer-specific model.

Instead:

- CLI should print the final campaign run directory and key counters,
- `diag dashboard` should work directly against that directory,
- DevTools should open the same directory through its existing aggregate consumer path,
- MCP should expose the same aggregate summary/index resources from that directory.

This preserves one shared handoff surface:

- `regression.index.json`
- `regression.summary.json`

## 9) Relationship to existing commands

The campaign entry should be implemented as orchestration over existing commands/engines, not as a replacement.

Recommended mapping:

- script item -> existing `diag run` logic
- suite item -> existing `diag suite` logic
- aggregate merge -> existing `diag summarize` logic
- human summary -> existing `diag dashboard` logic

This keeps the first slice small and makes the system easier to trust.

## 10) First-slice policy decisions

To keep v1 landable, make the following policy choices explicit.

### 10.1 Retry policy

For slice 1:

- allow only a simple campaign-level retry mode,
- prefer `none` or `retry_once`,
- defer richer flake classification policy to later slices.

### 10.2 Evidence policy

For slice 1:

- failures should leave bounded evidence by default,
- pack generation may remain opt-in or campaign-configurable,
- aggregate artifacts must always be emitted.

### 10.3 Launch model

For slice 1:

- reuse current `--launch` behavior,
- do not invent a second launch abstraction,
- keep command forwarding compatible with current `diag run` and `diag suite` flows.

## 11) Recommended landing plan

## Slice A - Command surface and manifest resolution

Deliver:

- `diag campaign run <campaign_id>`
- one minimal campaign registry or manifest resolver
- campaign run directory creation

Status:

- Done for the first landing via a built-in registry in `crates/fret-diag/src/diag_campaign.rs`.

Acceptance:

- a known campaign id expands deterministically into suites/scripts,
- the command prints the final run directory.

## Slice B - Aggregate output handoff

Deliver:

- campaign-level `regression.index.json`
- campaign-level `regression.summary.json`
- one concise human-readable dashboard output

Acceptance:

- `diag dashboard --dir <campaign_run_dir>` works,
- DevTools can read the resulting directory without a special adapter.

Status:

- Partially done: `diag campaign run` now emits `regression.index.json` and `regression.summary.json`
  by delegating to `diag summarize`.
- Newly done: filtered or multi-id campaign selection now emits one persisted batch artifact root
  with `batch.manifest.json`, `batch.result.json`, `regression.index.json`, and
  `regression.summary.json`.
- Newly done: `diag campaign share` turns a campaign or batch root into one bounded share surface
  by generating AI-only zips and `share/share.manifest.json`.
- Newly done: failed campaign or batch runs now best-effort export `share/share.manifest.json`
  automatically and record the path in campaign/batch result aggregates.
- Still open: persisting one explicit human-readable dashboard artifact in the campaign run directory.

## Slice C - Discoverability

Deliver:

- `diag campaign list`
- `diag campaign show <campaign_id>`
- short maintainer documentation for authoring and running campaigns

Acceptance:

- a new contributor can discover a valid campaign id without grepping Rust source.

Status:

- Done for the minimal surface (`list` + `show` are implemented).
- Still open: authoring documentation once campaign definitions stop being Rust-only.

## 12) Initial recommendation for naming

Preferred naming:

- keep `suite` as the primitive collection unit,
- use `campaign` for the orchestration layer,
- keep `regression` as the consumer/output vocabulary.

Concrete recommendation:

- command: `diag campaign run`
- artifacts: `regression.index.json` and `regression.summary.json`
- UI label: `Regression Workspace`

This matches the current repo direction more cleanly than introducing a second top-level noun.

## 13) Success criteria

This design is successful when:

- maintainers can name one campaign and run it locally,
- CI can invoke the same campaign entry without a pile of wrapper scripts,
- the run leaves one predictable artifact root,
- DevTools and MCP consume that root without special-case logic,
- future automation work can build on one orchestration entry instead of many ad-hoc shells.
