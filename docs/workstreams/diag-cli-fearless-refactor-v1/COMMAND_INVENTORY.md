# Diag CLI Command Inventory

Status: Draft (inventory baseline)
Last updated: 2026-03-26

Related:

- `docs/workstreams/diag-cli-fearless-refactor-v1/README.md`
- `docs/workstreams/diag-cli-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/diag-cli-fearless-refactor-v1/TODO.md`
- `apps/fretboard/src/cli.rs`
- `crates/fret-diag/src/lib.rs`
- `crates/fret-diag/src/diag_simple_dispatch.rs`

## 0) Purpose

This note is the baseline inventory for the `fretboard-dev diag` CLI reset.

It serves three jobs:

1. enumerate the current command surface,
2. identify the shared flag families that need modular ownership under `clap`,
3. identify repo-owned command-example surfaces that must migrate atomically when the parser resets.

This file is intentionally inventory-first. It is not the final parser design.

## 1) Current dispatch shape

The current `diag` surface is split across three layers:

1. top-level help text in `apps/fretboard/src/cli.rs`,
2. a large global parser + main dispatcher in `crates/fret-diag/src/lib.rs`,
3. a secondary “simple dispatch” layer in `crates/fret-diag/src/diag_simple_dispatch.rs`.

This split is one of the main structural problems:

- the help surface is hand-maintained,
- parser flags are globally flattened,
- command ownership is not obvious from one file,
- some command families are routed in `lib.rs` while others are routed via `diag_simple_dispatch.rs`.

## 2) Current command inventory

## 2.1 Session + filesystem trigger lane

Primary commands:

- `path`
- `poke`
- `latest`
- `sessions clean`

Current routing:

- `path`, `poke`, `latest` are handled in `crates/fret-diag/src/diag_simple_dispatch.rs`
- `sessions` is handled in `crates/fret-diag/src/diag_sessions.rs`

Notes:

- This is a coherent lane and should probably become one parser module.
- Session-specific flags (`--dir`, trigger-path-related flags, cleanup selection flags) should stop leaking into unrelated command definitions.

## 2.2 Execution + orchestration lane

Primary commands:

- `run`
- `repeat`
- `repro`
- `suite`
- `perf`
- `perf-baseline-from-bundles`
- `matrix`
- `campaign`

Current routing:

- `run` → `crates/fret-diag/src/diag_run.rs`
- `repeat` → `crates/fret-diag/src/diag_repeat.rs`
- `repro` → `crates/fret-diag/src/diag_repro.rs`
- `suite` → `crates/fret-diag/src/diag_suite.rs`
- `perf` → `crates/fret-diag/src/diag_perf.rs`
- `perf-baseline-from-bundles` → `crates/fret-diag/src/diag_perf_baseline.rs`
- `matrix` → `crates/fret-diag/src/diag_matrix.rs`
- `campaign` → `crates/fret-diag/src/diag_campaign.rs`

Notes:

- This is the highest-value first migration batch for `clap`.
- These commands share the largest number of flags and the highest-risk validation rules.
- `run`, `suite`, and `perf` are the first parser targets because they combine the most shared flag families.

## 2.3 Artifact + inspection lane

Primary commands:

- `trace`
- `pack`
- `triage`
- `lint`
- `meta`
- `index`
- `test-ids`
- `test-ids-index`
- `frames-index`
- `windows`
- `dock-routing`
- `dock-graph`
- `screenshots`
- `hotspots`
- `bundle-v2`
- `ai-packet`
- `layout-sidecar`
- `layout-perf-summary`
- `memory-summary`
- `stats`
- `compare`
- `dashboard`
- `summarize`
- `resolve`

Current routing:

- `trace`, `resolve`, `pack`, `triage`, `lint`, `artifact lint`, `meta`, `index`, `test-ids`,
  `test-ids-index`, `frames-index`, `windows`, `dock-routing`, `dock-graph`, `screenshots`,
  `hotspots`, `bundle-v2`, `layout-sidecar`, `extensions`, `ai-packet`, `query`, and `slice`
  now enter through `cli/cutover.rs` and dispatch into the existing execution modules.
- `layout-perf-summary`, `memory-summary`, `dashboard`, `summarize`, `stats`, `compare` are still
  routed directly from `lib.rs`

Notes:

- This lane mixes artifact readers, sidecar materialization, report emitters, and query helpers.
- It should be grouped under a smaller number of parser modules, even if execution code stays distributed.
- `artifacts` and `layout_sidecar` are now treated as retired aliases on the migrated surface.
- `diag pack --schema2-only`, `diag triage --frames-index`, `diag triage --from-frames-index`,
  and positional `diag ai-packet <test_id>` are now treated as deleted syntax on the migrated
  surface.
- `trace` no longer owns inline dispatch logic; migrated and legacy entrypoints now share
  `crates/fret-diag/src/commands/trace.rs`.
- `hotspots --lite/--metric` is now part of the explicit migrated contract instead of remaining a
  repo-documented but top-level-help-hidden surface.
- Remaining alias pairs such as `layout-perf-summary` / `layout_perf_summary` still need an
  explicit keep-or-delete decision when that lane migrates.

## 2.4 Live inspect + interactive tooling lane

Primary commands:

- `inspect on|off|toggle|status`
- `pick-arm`
- `pick`
- `pick-script`
- `pick-apply`

Current routing:

- `inspect` → `crates/fret-diag/src/commands/inspect.rs`
- `pick-*` → `crates/fret-diag/src/commands/pick.rs`

Notes:

- This is a coherent live-interaction lane and should not share parser state with report-only commands.

## 2.5 Script authoring + registry/config lane

Primary commands:

- `script`
- `list`
- `registry`
- `config`
- `doctor`
- `artifact`
- `extensions`
- `agent`
- `query`
- `slice`

Current routing:

- `script` → `crates/fret-diag/src/commands/script.rs`
- `list` → `crates/fret-diag/src/diag_list.rs`
- `registry` → `crates/fret-diag/src/commands/registry.rs`
- `config` → `crates/fret-diag/src/commands/config.rs`
- `doctor` → `crates/fret-diag/src/commands/doctor.rs`
- `artifact` → `crates/fret-diag/src/commands/artifact.rs`
- `extensions` → `crates/fret-diag/src/commands/extensions.rs`
- `agent` → `crates/fret-diag/src/commands/agent.rs`
- `query` → `crates/fret-diag/src/commands/query.rs`
- `slice` → `crates/fret-diag/src/commands/slice.rs`

Notes:

- `artifact`, `doctor`, `script`, and `query` are all nested command families and should have their own subcommand parser modules.
- `agent` should remain clearly isolated because it is a consumer-facing helper surface, not a core execution primitive.
- `doctor` is now migrated: typed `clap` parsing and help already dispatch to
  `crates/fret-diag/src/commands/doctor.rs`,
  `crates/fret-diag/src/commands/doctor_scripts.rs`, and
  `crates/fret-diag/src/commands/doctor_campaigns.rs` through the cutover layer.
- `list` is now the first migrated utility family: typed `clap` parsing and help already dispatch to
  the existing `crates/fret-diag/src/diag_list.rs` execution module through the cutover layer.
- `query` is now migrated as a nested `clap` family with canonical subcommands
  `test-id` / `snapshots` / `overlay-placement-trace` / `scroll-extents-observation`; old alias
  spellings are rejected at cutover instead of falling back to parser-v1.
- `slice` is now migrated as a typed single-command surface, including the documented
  `--step-index` selector path.

## 3) Nested command families

Current nested subcommand surfaces:

- `list`
  - `scripts`
  - `suites`
  - `sessions`
- `sessions`
  - `clean`
- `artifact`
  - `lint`
- `doctor`
  - default bundle doctor run
  - `scripts`
  - `campaigns`
- `query`
  - `test-id`
  - `snapshots`
  - `overlay-placement-trace`
  - `scroll-extents-observation`
- `script`
  - direct script push/run helper (`fretboard-dev diag script <script.json>`)
  - `normalize`
  - `upgrade`
  - `validate`
  - `lint`
  - `shrink`

Migration implication:

- these should not be represented as loose string branches inside one global parser loop,
- each family should become its own `clap` subcommand module,
- family-local validation should move next to that module.

## 4) Shared flag families that need module ownership

The current parser flattens nearly every flag into one global state bag. The new `clap` model
should instead group flags into reusable families.

## 4.1 Launch + process lifecycle

Representative flags:

- `--launch -- <cmd...>`
- `--env KEY=VALUE`
- `--reuse-launch`
- `--reuse-launch-per-script`
- `--keep-open`
- `--launch-high-priority`
- `--launch-write-bundle-json`
- `--exit-after-run`

Primary consumers:

- `run`
- `repeat`
- `repro`
- `suite`
- `perf`
- `matrix`
- `script shrink`
- `campaign`

## 4.2 DevTools / transport session

Representative flags:

- `--devtools-ws-url`
- `--devtools-token`
- `--devtools-session-id`
- `--session-auto`
- `--session`

Primary consumers:

- `run`
- `suite`
- `perf`
- `campaign`
- selected artifact-resolution flows

## 4.3 Filesystem path plumbing

Representative flags:

- `--dir`
- `--trigger-path`
- `--script-path`
- `--script-trigger-path`
- `--script-result-path`
- `--script-result-trigger-path`
- `--pick-trigger-path`
- `--pick-result-path`
- `--pick-result-trigger-path`
- `--pick-script-out`

Primary consumers:

- session-trigger commands
- `run`
- `suite`
- `script`
- pick/inspect helpers

## 4.4 Output + artifact emission

Representative flags:

- `--pack`
- `--pack-out`
- `--packet-out`
- `--ai-packet`
- `--ai-only`
- `--include-all`
- `--include-root-artifacts`
- `--include-triage`
- `--include-screenshots`
- `--json`
- `--out`

Primary consumers:

- `run`
- `repro`
- artifact/report lanes

## 4.5 Timing + display

Representative flags:

- `--timeout-ms`
- `--poll-ms`
- `--warmup-frames`
- `--top`
- `--sort`
- `--verbose`
- `--trace`
- `--trace-out`
- `--meta-report`

Primary consumers:

- execution lanes
- artifact/report lanes

## 4.6 Suite input selection

Representative flags:

- `--script-dir`
- `--glob`
- `--prewarm-script`
- `--prelude-script`
- `--prelude-each-run`

Primary consumers:

- `suite`
- `perf`
- `campaign`

## 4.7 Compare + perf baseline

Representative flags:

- `--repeat`
- `--no-compare`
- `--compare-eps-px`
- `--compare-ignore-bounds`
- `--compare-ignore-scene-fingerprint`
- `--perf-baseline`
- `--perf-baseline-out`
- `--perf-baseline-headroom-pct`
- `--perf-baseline-seed-preset`
- `--perf-baseline-seed`
- `--perf-threshold-agg`
- `--check-perf-hints`
- `--check-perf-hints-deny`
- `--check-perf-hints-min-severity`

Primary consumers:

- `repeat`
- `perf`
- `compare`
- `perf-baseline-from-bundles`

## 4.8 Script tooling + normalization

Representative flags:

- `--write`
- `--check`
- `--check-out`
- `--shrink-out`
- `--shrink-any-fail`
- `--shrink-match-reason-code`
- `--shrink-match-reason`
- `--shrink-min-steps`
- `--shrink-max-iters`

Primary consumers:

- `script normalize`
- `script upgrade`
- `script validate`
- `script lint`
- `script shrink`

## 4.9 Diagnostics checks and thresholds

Representative flags:

- correctness gates such as `--check-stale-paint`, `--check-pixels-changed`, `--check-wheel-scroll`
- suite/report gates such as `--check-hover-layout`, `--check-overlay-synthesis-min`
- resource thresholds such as `--max-working-set-bytes`, `--max-renderer-gpu-images-bytes-estimate`
- render/text/editor-specific thresholds and correctness checks

Notes:

- this is the biggest shared family in the current parser,
- it must be modularized under `clap`,
- it should probably become multiple reused groups rather than one “all checks” mega-struct.

## 5) Repo-owned command-example migration surface

Current inventory snapshot from repo search for `fretboard-dev diag `:

- `docs/`: 134 files
- `.agents/`: 8 files
- `tools/`: 19 files

These are the main doc/example surfaces that must migrate atomically when command shapes change.

## 5.1 Core docs

Representative files:

- `docs/ui-diagnostics-and-scripted-tests.md`
- `docs/debugging-playbook.md`
- `docs/docking-imgui-parity-matrix.md`
- `docs/shadcn-conformance-matrix.md`

Risk:

- these are user-facing and maintainer-facing reference surfaces,
- stale commands here will immediately undermine the reset.

## 5.2 Workstreams and historical planning docs

Representative files:

- `docs/workstreams/diag-fearless-refactor-v2/README.md`
- `docs/workstreams/diag-architecture-fearless-refactor-v1/TODO.md`
- `docs/workstreams/diag-simplification-v1/diag-simplification-v1.md`
- `docs/workstreams/example-suite-fearless-refactor-v1/design.md`

Risk:

- this is the largest migration surface by file count,
- some files should be updated,
- some purely historical files may instead need an explicit “historical command surface” note.

## 5.3 Maintainer skills and operational references

Representative files:

- `.agents/skills/fret-diag-workflow/SKILL.md`
- `.agents/skills/fret-diag-workflow/references/launch-and-artifact-hygiene.md`
- `.agents/skills/fret-diag-workflow/references/triage-and-maintainer-notes.md`
- `.agents/skills/fret-framework-maintainer-guide/references/contract-change-checklist.md`

Risk:

- these files actively teach maintainers and agents what to run,
- they must move with the command surface, not after it.

## 5.4 Tools, wrappers, and gate helpers

Representative files:

- `tools/diag-scripts/suites/README.md`
- `tools/diag-configs/README.md`
- `tools/perf/diag_perf_baseline_select.py`
- `tools/diag_gate_action_first_authoring_v1.py`
- `tools/diag_gate_interaction_kernel_v1.py`

Risk:

- some of these wrap actual invocations,
- some of them embed help text or expectations about CLI shape,
- they need migration testing, not just text edits.

## 5.5 User-visible command strings inside code

Representative sources:

- `apps/fretboard/src/cli.rs`
- `crates/fret-diag/src/diag_run.rs`
- `crates/fret-diag/src/diag_suite.rs`
- `crates/fret-diag/src/commands/doctor.rs`
- `crates/fret-diag/src/commands/query.rs`

Risk:

- these are not docs, but they are user-visible guidance surfaces,
- parser reset must update them together with the executable surface.

## 6) Immediate implications for the `clap` reset

The first parser migration batch should target:

1. `run`
2. `suite`
3. `perf`

Reason:

- they cover the heaviest shared flag families,
- they exercise launch/session/output/check composition,
- they are the command families most likely to reveal whether the new module split is correct.

The first inventory-driven follow-up docs should be:

1. `COMMAND_EXAMPLE_INVENTORY.md` or an equivalent per-directory checklist if the example migration grows larger,
2. a parser module map once the first `clap` types are introduced,
3. help snapshot coverage for the most important command families.
