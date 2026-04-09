# Diag CLI Fearless Refactor v1 — Parser Model

Status: Archived baseline (superseded by `OWNERSHIP.md` for the final merged state)
Last updated: 2026-03-26

Related:

- `docs/workstreams/diag-cli-fearless-refactor-v1/README.md`
- `docs/workstreams/diag-cli-fearless-refactor-v1/OWNERSHIP.md`
- `docs/workstreams/diag-cli-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/diag-cli-fearless-refactor-v1/TODO.md`
- `docs/workstreams/diag-cli-fearless-refactor-v1/COMMAND_INVENTORY.md`
- `crates/fret-diag/src/lib.rs`
- `crates/fret-diag/src/cli/contracts/`

## 0) Purpose

This note records the first concrete parser model for the `clap` reset.

Historical note:

- the contract scaffold described below has since become the production `diag` contract
- use `OWNERSHIP.md` for the final merged ownership split

It answers the M1 questions that should be settled before we switch runtime execution over to the
new parser:

- ownership,
- module shape,
- shared arg-family boundaries,
- validation boundaries,
- first-party caller migration rules,
- and the final representation for `--launch -- <cmd...>`.

## 1) First-party caller migration rule

This workstream is a pre-release break window, so first-party callers should migrate by deletion,
not by compatibility layering.

Rules:

- When a command shape changes, delete the old syntax in the same change that introduces the new one.
- Update repo-owned callers atomically:
  - `docs/`,
  - `.agents/`,
  - `tools/`,
  - parser-sensitive tests,
  - maintainer notes and workstream docs that teach executable commands.
- Do not merge a parser fallback path that keeps accepting the deleted spelling.
- Do not keep “just in case” aliases unless the final contract explicitly wants them.
- If a historical command example remains useful for audit reasons, label it `Historical syntax (deleted)` and keep it out of the main copy-paste path.

Practical grep rule for each migration batch:

- grep the repo for `fretboard-dev diag <command>` before the cutover,
- migrate repo-owned callers in the same window,
- grep again after the cutover and remove the final residues of deleted syntax.

## 2) Ownership map

Locked ownership split:

- `apps/fretboard`
  - owns the top-level application shell and non-`diag` command families,
  - should stop owning hand-maintained `diag` command details once the new help is wired.
- `crates/fret-diag`
  - owns the `diag` command model,
  - owns parser-to-execution conversion,
  - owns command execution entrypoints.
- `crates/fret-diag/src/cli/shared/`
  - owns reusable arg families.
- `crates/fret-diag/src/cli/commands/`
  - owns command-local parser structs.
- execution modules such as `diag_run.rs`, `diag_suite.rs`, `diag_perf.rs`
  - remain `clap`-free,
  - consume small internal contexts rather than parser structs.

## 3) Module map

Target production layout:

- `crates/fret-diag/src/cli/mod.rs`
  - parser entrypoint and workspace helpers.
- `crates/fret-diag/src/cli/shared/`
  - shared arg families:
    - output,
    - timing,
    - session,
    - devtools,
    - launch,
    - pack,
    - checks.
- `crates/fret-diag/src/cli/commands/`
  - one module per command family.
- `crates/fret-diag/src/cli/conversions/`
  - typed parser structs to internal execution contexts.

Current incremental scaffold:

- `crates/fret-diag/src/cli/contracts/`
  - test-only `clap` contract scaffold,
  - mirrors the intended final layout,
  - locks the first migration batch (`run` and `suite`) before production cutover.

The contract scaffold is intentionally not the final execution wiring. It exists to prevent
“design by giant parser rewrite” and to force shared-arg boundaries into review early.

## 4) Top-level command tree direction

The top-level tree remains `fretboard-dev diag <subcommand>`.

Immediate first migration batch:

- `run`
- `suite`
- `perf`
- `repro`
- `repeat`
- `campaign`

First utility cutover after that batch:

- `list`
- `doctor`

Recommended structural grouping:

- execution/orchestration:
  - `run`, `suite`, `perf`, `repro`, `repeat`, `matrix`, `campaign`
- artifact/reporting:
  - `pack`, `triage`, `lint`, `meta`, `index`, `test-ids`, `stats`, `compare`, `dashboard`, `summarize`
- session/filesystem:
  - `path`, `poke`, `latest`, `resolve`, `sessions`
- authoring/registry:
  - `script`, `registry`, `doctor`, `list`, `config`, `artifact`, `query`, `slice`
- live inspect:
  - `inspect`, `pick-arm`, `pick`, `pick-script`, `pick-apply`

## 5) Shared arg families

### 5.1 Output and artifact emission

Owns flags such as:

- `--dir`
- `--json`
- `--out`
- `--pack`
- `--pack-out`
- `--ai-packet`
- `--ai-only`
- `--include-all`
- `--include-root-artifacts`
- `--include-triage`
- `--include-screenshots`
- `--pack-schema2-only`

### 5.2 Timing and polling

Owns flags such as:

- `--timeout-ms`
- `--poll-ms`
- `--warmup-frames`

Current cutover note:

- `trace`, `resolve`, `pack`, `triage`, `lint`, `ai-packet`, `windows`, `dock-routing`,
  `dock-graph`, `screenshots`, `hotspots`, and `bundle-v2` now have dedicated typed command
  modules in the `clap` contract, while still reusing the current reporting/artifact execution
  modules through the cutover conversion layer.
- `artifact lint`, `meta`, `index`, `test-ids`, `layout-sidecar`, and `extensions` now reuse a
  dedicated `ReportOutputArgs` (`--json` / `--out`) family, and the warmup-sensitive subset also
  reuses `WarmupFramesArgs`.
- `test-ids-index` and `frames-index` now reuse `WarmupFramesArgs` but intentionally do not expose
  `--out`, which keeps them aligned with the existing sidecar materialization behavior.
- `hotspots` now makes its `--lite` / `--metric` surface explicit in generated help instead of
  relying on drifted repo examples.
- `query` now reuses the same `ReportOutputArgs` / `WarmupFramesArgs` split, but keeps its
  family-local subcommand model for `test-id` / `snapshots` / `overlay-placement-trace` /
  `scroll-extents-observation`.
- `slice` now reuses `ReportOutputArgs` / `WarmupFramesArgs` and declares selector conflicts
  (`--frame-id` vs `--snapshot-seq` vs `--step-index`) structurally in the `clap` model.

### 5.3 Session and transport scope

Owns flags such as:

- `--session-auto`
- `--session`

### 5.4 DevTools transport

Owns flags such as:

- `--devtools-ws-url`
- `--devtools-token`
- `--devtools-session-id`

### 5.5 Launch and process lifecycle

Owns flags such as:

- `--env KEY=VALUE`
- `--launch -- <cmd...>`
- `--launch-high-priority`
- `--launch-write-bundle-json`
- `--keep-open`
- `--reuse-launch`
- `--reuse-launch-per-script`
- `--exit-after-run`

### 5.6 Diagnostics checks

Owns flags such as:

- stale-paint / stale-scene gates,
- pixel-changed / unchanged gates,
- idle-no-paint gates,
- asset-load gates,
- suite/perf-specific check families that are still command-local once they stop being broadly shared.

Rule:

- a flag family belongs in `shared/` only if at least two command families consume it with the same meaning.
- if a flag is strongly lane-specific, keep it in the command-local parser module.

## 6) Validation boundary

Validation should happen in three layers:

1. `clap` declaration layer
   - required positional arguments,
   - mutual exclusion,
   - simple `requires`,
   - basic typed parsing.
2. command-local conversion layer
   - semantic rules that need more than field-level declaration,
   - normalization of alias or implied behavior,
   - conversion into execution contexts.
3. execution layer
   - runtime-only constraints,
   - filesystem and process launch failures,
   - transport-specific availability checks.

Rule of thumb:

- if an invalid invocation can be rejected before touching the filesystem or process launch, it should be rejected before execution.

## 7) Final representation for `--launch -- <cmd...>`

Locked representation:

- parse into `Vec<String>`,
- use a multi-value `clap` option for `--launch`,
- document `--` as the canonical separator between tool args and launched-command args,
- document the canonical spelling as `--launch -- <cmd...>`.

Non-goals:

- no shell-string mode,
- no extra quoting layer invented by Fret,
- no parser fallback that tries to guess whether later flags belong to the tool or the launched command.

Reason:

- `Vec<String>` preserves exact argv shape,
- it matches how `std::process::Command` is constructed,
- it avoids another round of parser ambiguity.

## 8) Alias policy

Keep only aliases that are intentional in the final contract.

Examples of aliases that need an explicit decision rather than silent carry-over:

- hyphenated vs snake_case command spellings,
- historical short spellings introduced by hand-written parsing,
- pack/schema2 naming variants.

Current deleted syntax on the migrated surface:

- `diag pack --schema2-only`
- `diag triage --frames-index`
- `diag triage --from-frames-index`
- positional `diag ai-packet <test_id>`

Default policy:

- prefer one canonical spelling,
- delete accidental duplicates,
- keep an alias only when it improves a deliberate user story or preserves a meaningful naming family.

## 9) Current implementation checkpoint

The repository now has a checked-in `clap` contract scaffold under `crates/fret-diag/src/cli/contracts/`.

What it does today:

- models the initial `run` and `suite` parser surface,
- models the initial `repeat` parser surface,
- models the initial `repro` parser surface,
- models the current `perf` execution surface,
- models the current nested `campaign` execution surface,
- uses shared arg-family modules instead of one mutable state bag,
- locks `diag suite --script-dir ...` as a valid parser shape,
- locks `--launch -- <cmd...>` capture in parser tests,
- renders generated help for `diag run --help`, `diag suite --help`, `diag repeat --help`,
  `diag repro --help`, `diag perf --help`, and nested `diag campaign ... --help`,
- renders generated help for `diag pack --help`, `diag triage --help`, `diag lint --help`, and
  `diag ai-packet --help`,
- renders generated help for `diag trace --help`, `diag resolve --help`,
  `diag resolve latest --help`, `diag test-ids-index --help`, `diag frames-index --help`,
  `diag windows --help`, `diag dock-routing --help`, `diag dock-graph --help`,
  `diag screenshots --help`, `diag hotspots --help`, and `diag bundle-v2 --help`,
- dispatches a supported `run` / `suite` / `repeat` / `repro` subset through the new conversion
  layer in production,
- dispatches `diag perf` through the new conversion layer in production without falling back for
  stale per-command flags,
- dispatches `diag campaign` through the new conversion layer in production,
- dispatches `diag pack`, `diag triage`, `diag lint`, and `diag ai-packet` through the new
  conversion layer in production,
- dispatches `diag trace`, `diag resolve latest`, `diag test-ids-index`, `diag frames-index`,
  `diag windows`, `diag dock-routing`, `diag dock-graph`, `diag screenshots`, `diag hotspots`,
  and `diag bundle-v2` through the new conversion layer in production,
- deletes legacy aliases from the migrated `run` / `suite` contract surface,
- deletes legacy suite/perf flag spellings from the migrated `perf` contract surface,
- deletes `diag pack --schema2-only`, `diag triage --frames-index`,
  `diag triage --from-frames-index`, and positional `diag ai-packet <test_id>` from the migrated
  utility surface,
- provides a place to expand the contract before the full runtime cutover.

What it does not do yet:

- cover the full `run` / `suite` / `repeat` / `repro` flag surface,
- cover the remaining utility-lane command surface,
- replace the existing parser in `crates/fret-diag/src/lib.rs`,
- remove temporary fallback to parser-v1 for unsupported `run` / `suite` / `repeat` / `repro`
  flags,
- cover every `diag` command family.
