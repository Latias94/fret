# Diag CLI Fearless Refactor v1

Status: Closeout-ready
Last updated: 2026-03-26

Tracking files:

- `docs/workstreams/diag-cli-fearless-refactor-v1/README.md`
- `docs/workstreams/diag-cli-fearless-refactor-v1/CLOSEOUT.md`
- `docs/workstreams/diag-cli-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/diag-cli-fearless-refactor-v1/OWNERSHIP.md`
- `docs/workstreams/diag-cli-fearless-refactor-v1/PARSER_MODEL.md`
- `docs/workstreams/diag-cli-fearless-refactor-v1/FOLLOWUPS.md`
- `docs/workstreams/diag-cli-fearless-refactor-v1/TODO.md`

Related context:

- Umbrella diagnostics workstream: `docs/workstreams/diag-fearless-refactor-v2/README.md`
- Diagnostics architecture lane: `docs/workstreams/diag-architecture-fearless-refactor-v1/README.md`
- Top-level CLI shell: `apps/fretboard/src/cli.rs`
- Final diagnostics entrypoint: `crates/fret-diag/src/lib.rs`
- Canonical `clap` contract: `crates/fret-diag/src/cli/contracts/`
- Current suite orchestration hotspot: `crates/fret-diag/src/diag_suite.rs`

## 0) Why this workstream exists

`fretboard-dev diag` has outgrown the current hand-written parser shape.

Today the command surface is difficult to evolve safely because parsing, validation, defaults,
cross-flag policy, help text, and command dispatch are still too tightly coupled.

Symptoms:

- the parse surface is concentrated in one large mutable state machine in `crates/fret-diag/src/lib.rs`,
- help output is manually maintained in `apps/fretboard/src/cli.rs`,
- subcommand-local rules are still enforced by far-away post-parse checks,
- argument families such as launch/env/output/checks are duplicated across many command paths,
- user-facing failures can still come from parser shape drift rather than from a deliberate command contract.

The recent `diag suite --script-dir ...` panic was not the main problem. It was evidence that the
current parser architecture is already brittle enough that small surface changes can leak into
runtime failures.

## 1) Main decision

This workstream is a **pre-release hard reset**, not a compatibility migration lane.

Decision:

- we will fully replace the current `diag` parser architecture,
- we will use `clap` as the parser and help-generation framework for the final merged design,
- we will not keep a long-lived dual-parser setup in `main`,
- we will not keep deprecated aliases or fallback parsing paths solely for compatibility,
- we will update first-party docs/tests/scripts in the same migration window,
- we will prefer one clean command model over preserving every historical flag quirk.

Practical reading rule:

- temporary branch-local scaffolding is allowed while a change is being developed,
- merged code should not retain parser-v1 and parser-v2 side by side,
- merged code should not accept legacy spellings unless they are still part of the intended final contract.

## 2) Scope

### In scope

- the full `fretboard-dev diag` command model,
- parser ownership and module boundaries,
- shared argument groups (`launch`, `env`, `out`, `checks`, `sessions`, `pack`, `transport`),
- subcommand validation rules and error messages,
- help generation and example quality,
- parser-focused regression coverage,
- migration of repo-owned callers (docs, tests, scripts, maintainer notes) to the new surface.

### Explicit non-goals

- redesigning diagnostics artifact schemas by default,
- changing diagnostics runtime semantics unless a parser reset exposes a real contract bug,
- refactoring unrelated `fretboard` command families (`assets`, `new`, `dev`, `theme`) in this lane,
- preserving exact string-for-string output compatibility with the current help text.

## 3) End-state requirements

The final shipped state should satisfy all of the following:

1. `fretboard-dev diag` has one typed command model and one source of truth for parsing.
2. `clap` is the only parser/help source of truth for `fretboard-dev diag`.
3. Help text is generated from the command model rather than hand-maintained prose blobs.
4. Shared flag families are declared once and reused structurally.
5. Subcommand-local validation lives next to the subcommand model whenever possible.
6. Cross-command semantic checks are small, explicit, and post-parse only when declaration alone is insufficient.
7. The old giant mutable parse loop is deleted from `crates/fret-diag/src/lib.rs`.
8. First-party docs and scripts no longer teach the deleted surface.
9. Any repo document that still shows removed command shapes is either updated or explicitly marked historical.

## 4) Locked technical direction

This workstream now locks `clap` as the parser and help-generation framework for `fretboard-dev diag`.

Reason:

- it covers the hard command-model features this CLI actually needs,
- it lets parsing/help/validation move from ad hoc runtime logic into declared command structure,
- it supports the module-composition style we want for shared argument families.

Relevant `clap` fit points for this repo:

- nested subcommands,
- reusable argument structs,
- argument groups and mutual exclusion,
- `requires` / `conflicts_with` style validation,
- typed enums and value parsers,
- trailing command capture for `--launch -- <cmd...>`.

What is now locked:

- the parser is `clap`,
- the command model is typed,
- the final merged state does not rely on the current manual parser loop.

What is still intentionally open:

- the exact module file layout,
- the exact internal conversion layer shape between `clap` types and execution contexts,
- which small set of historical aliases, if any, survive because they are still part of the intended final contract.

Current design note:

- `docs/workstreams/diag-cli-fearless-refactor-v1/PARSER_MODEL.md` remains the historical design
  baseline, while `OWNERSHIP.md` records the final merged ownership split after the production
  cutover landed.
- `diag run --help`, `diag suite --help`, `diag repeat --help`, `diag repro --help`, and
  `diag perf --help` now render from the new `clap` contract.
- `diag campaign --help`, `diag campaign list --help`, and `diag campaign run --help` now render
  from the new nested `clap` contract instead of falling through to ad hoc runtime errors.
- `diag doctor --help`, `diag doctor scripts --help`, and `diag doctor campaigns --help` now render
  from the new nested `clap` contract.
- `diag list --help`, `diag list scripts --help`, and `diag list sessions --help` now render from
  the new nested `clap` contract.
- `diag script --help`, `diag script normalize --help`, `diag script upgrade --help`, and
  `diag script shrink --help` now render from the new nested `clap` contract.
- `run`, `suite`, `repeat`, and `repro` now dispatch through the new `clap` model for the intended
  shipped surface; residual validation/test hardening is split into a narrow follow-up lane rather
  than keeping parser-v1 compatibility alive.
- `diag perf` now dispatches through the new `clap` model with its real execution surface and no
  longer falls back to parser-v1 for stale per-command flags.
- `diag campaign` now dispatches through the new `clap` model for `list` / `show` / `validate` /
  `share` / `run`, while reusing the current execution modules behind the conversion layer.
- `diag doctor` now dispatches through the new `clap` model for bundle doctor / `scripts` /
  `campaigns`, while reusing `crates/fret-diag/src/commands/doctor*.rs` behind the conversion layer.
- `diag list` now dispatches through the new `clap` model for `scripts` / `suites` / `sessions`,
  while reusing `crates/fret-diag/src/diag_list.rs` behind the conversion layer.
- `diag script` now dispatches through the new `clap` model for direct execution plus
  `normalize` / `upgrade` / `validate` / `lint` / `shrink`, while reusing
  `crates/fret-diag/src/commands/script.rs` behind the conversion layer.
- `diag artifact lint`, `diag meta`, `diag index`, `diag test-ids`, `diag layout-sidecar`, and
  `diag extensions` now dispatch through the new `clap` model while reusing the current
  reporting/artifact execution modules behind the conversion layer.
- `diag trace`, `diag resolve latest`, `diag test-ids-index`, and `diag frames-index` now
  dispatch through the new `clap` model while reusing the current execution modules behind the
  conversion layer.
- `diag windows`, `diag dock-routing`, `diag dock-graph`, `diag screenshots`, `diag hotspots`,
  and `diag bundle-v2` now also dispatch through the new `clap` model while reusing the current
  execution modules behind the conversion layer.
- `diag compare`, `diag dashboard`, `diag stats`, and `diag summarize` now render help from the new
  `clap` contract and dispatch through the new cutover path while reusing the current reporting
  modules behind typed execution contexts.
- `diag layout-perf-summary`, `diag memory-summary`, `diag inspect`, and the `diag pick*` helper
  family now also render help from the new `clap` contract and dispatch through the new cutover
  path while reusing the current execution modules behind typed contexts.
- `diag agent`, `diag path`, `diag poke`, `diag latest`, `diag sessions clean`,
  `diag perf-baseline-from-bundles`, `diag matrix`, `diag registry`, and
  `diag config doctor` now also render help from the new `clap` contract and dispatch through the
  new cutover path while reusing the current execution modules behind typed contexts.
- `diag pack`, `diag triage`, `diag lint`, and `diag ai-packet` now dispatch through the new
  `clap` model while reusing the current reporting/artifact execution modules behind the
  conversion layer.
- `diag query` now dispatches through the new nested `clap` model for the canonical
  `test-id` / `snapshots` / `overlay-placement-trace` / `scroll-extents-observation` surface,
  while retiring old alias spellings instead of falling back to parser-v1.
- `diag slice` now dispatches through the new `clap` model, including the documented
  `--step-index` / `--warmup-frames` selector surface, while reusing
  `crates/fret-diag/src/commands/slice.rs` behind the conversion layer.
- The migrated artifact lane now explicitly rejects deleted syntax instead of silently guessing:
  `diag pack --schema2-only`, `diag triage --frames-index`, `diag triage --from-frames-index`,
  and positional `diag ai-packet <test_id>`.
- The newly migrated reporting lane also explicitly rejects deleted or unsupported syntax:
  `diag compare --compare-footprint`, `diag dashboard --warmup-frames`, `diag stats
  --check-prepaint-actions-min`, and `diag summarize --top`.
- The newly migrated utility/helper lane also explicitly rejects deleted or unsupported syntax:
  `diag layout_perf_summary`, `diag memory_summary`, `diag memory-summary --sort_key`, and
  `diag pick-script --warmup-frames`.
- The final remaining utility/control lane now also explicitly rejects unsupported syntax instead
  of falling back to parser-v1: `diag matrix --launch-write-bundle-json`,
  `diag perf-baseline-from-bundles --pack`, and invalid `diag sessions clean --top 0`.
- Every currently shipped `diag` command family now renders help from and dispatches through the
  new `clap` tree.
- `crates/fret-diag/src/lib.rs` no longer contains the old mutable parser loop or legacy simple
  dispatch fallback; `diag_cmd` now delegates directly to the typed `clap` contract dispatcher.
- Execution modules that remain behind the cutover path no longer carry duplicated hand-written
  `diag` usage/help branches for migrated commands; help ownership is centralized in the `clap`
  contract surface.
- `apps/fretboard/src/cli.rs` no longer mirrors the full `diag` usage surface in hand-maintained
  prose; it now points callers at `fretboard-dev diag --help`, which is generated from the executable
  contract.

Closeout note:

- This lane is now closeout-ready.
- Final ownership is recorded in `OWNERSHIP.md`.
- Residual work is split into named follow-up lanes in `FOLLOWUPS.md`.

Latest smoke evidence:

- `target/debug/fretboard-dev diag --help`
- `target/debug/fretboard-dev diag agent --help`
- `target/debug/fretboard-dev diag path --help`
- `target/debug/fretboard-dev diag poke --help`
- `target/debug/fretboard-dev diag latest --help`
- `target/debug/fretboard-dev diag campaign --help`
- `target/debug/fretboard-dev diag campaign list --help`
- `target/debug/fretboard-dev diag campaign run --help`
- `target/debug/fretboard-dev diag campaign list --lane smoke --json`
- `target/debug/fretboard-dev diag campaign show ui-gallery-smoke --json`
- `target/debug/fretboard-dev diag campaign validate --json`
- `target/debug/fretboard-dev diag campaign run ui-gallery-smoke --timeout-ms 1 --poll-ms 1`
- `target/debug/fretboard-dev diag campaign run --with tracy`
- `target/debug/fretboard-dev diag doctor --help`
- `target/debug/fretboard-dev diag doctor scripts --help`
- `target/debug/fretboard-dev diag doctor campaigns --help`
- `target/debug/fretboard-dev diag doctor campaigns --json`
- `target/debug/fretboard-dev diag doctor scripts --json`
- `target/debug/fretboard-dev diag doctor campaigns --fix`
- `target/debug/fretboard-dev diag doctor target/fret-diag/sessions/1774403458054-77810 --json`
- `target/debug/fretboard-dev diag list --help`
- `target/debug/fretboard-dev diag list scripts --help`
- `target/debug/fretboard-dev diag list scripts --contains ui-gallery --top 1 --json`
- `target/debug/fretboard-dev diag list suites --contains ui-gallery --top 2 --json`
- `target/debug/fretboard-dev diag list sessions --dir target/fret-diag --top 1 --json`
- `target/debug/fretboard-dev diag list scripts --dir target/fret-diag`
- `target/debug/fretboard-dev diag sessions --help`
- `target/debug/fretboard-dev diag sessions clean --help`
- `target/debug/fretboard-dev diag sessions clean --keep 1 --json`
- `target/debug/fretboard-dev diag path --dir target/fret-diag-clap-smoke`
- `target/debug/fretboard-dev diag poke --dir target/fret-diag-clap-smoke`
- `target/debug/fretboard-dev diag latest --dir target/fret-diag-clap-smoke`
- `target/debug/fretboard-dev diag script --help`
- `target/debug/fretboard-dev diag script normalize --help`
- `target/debug/fretboard-dev diag script upgrade --help`
- `target/debug/fretboard-dev diag script shrink --help`
- `target/debug/fretboard-dev diag script tools/diag-scripts/ui-gallery-intro-idle-screenshot.json --dir target/fret-diag-script-smoke`
- `target/debug/fretboard-dev diag script validate tools/diag-scripts/ui-gallery-intro-idle-screenshot.json --dir target/fret-diag-script-validate-smoke --json`
- `target/debug/fretboard-dev diag script lint tools/diag-scripts/ui-gallery-intro-idle-screenshot.json --dir target/fret-diag-script-lint-smoke --json`
- `target/debug/fretboard-dev diag script shrink tools/diag-scripts/ui-gallery-intro-idle-screenshot.json --reuse-launch` (expected parser rejection)
- `target/debug/fretboard-dev diag artifact lint --help`
- `target/debug/fretboard-dev diag pack --help`
- `target/debug/fretboard-dev diag triage --help`
- `target/debug/fretboard-dev diag lint --help`
- `target/debug/fretboard-dev diag ai-packet --help`
- `target/debug/fretboard-dev diag meta --help`
- `target/debug/fretboard-dev diag index --help`
- `target/debug/fretboard-dev diag test-ids --help`
- `target/debug/fretboard-dev diag test-ids-index --help`
- `target/debug/fretboard-dev diag frames-index --help`
- `target/debug/fretboard-dev diag windows --help`
- `target/debug/fretboard-dev diag dock-routing --help`
- `target/debug/fretboard-dev diag dock-graph --help`
- `target/debug/fretboard-dev diag screenshots --help`
- `target/debug/fretboard-dev diag hotspots --help`
- `target/debug/fretboard-dev diag bundle-v2 --help`
- `target/debug/fretboard-dev diag layout-sidecar --help`
- `target/debug/fretboard-dev diag extensions --help`
- `target/debug/fretboard-dev diag trace --help`
- `target/debug/fretboard-dev diag resolve --help`
- `target/debug/fretboard-dev diag resolve latest --help`
- `target/debug/fretboard-dev diag registry --help`
- `target/debug/fretboard-dev diag registry check --json`
- `target/debug/fretboard-dev diag config --help`
- `target/debug/fretboard-dev diag config doctor --help`
- `target/debug/fretboard-dev diag config doctor --mode manual --report-json`
- `target/debug/fretboard-dev diag query --help`
- `target/debug/fretboard-dev diag query snapshots --help`
- `target/debug/fretboard-dev diag slice --help`
- `target/debug/fretboard-dev diag perf-baseline-from-bundles --help`
- `target/debug/fretboard-dev diag matrix --help`
- `target/debug/fretboard-dev diag artifact lint target/fret-diag-ai-model-selector-focus-gate/1774159915361 --out target/fret-diag-clap-smoke/artifact.lint.json`
- `target/debug/fretboard-dev diag pack target/fret-diag-ai-model-selector-focus-gate/1774159915361 --warmup-frames 4 --pack-out target/fret-diag-clap-smoke/demo.zip`
- `target/debug/fretboard-dev diag triage target/fret-diag-ai-model-selector-focus-gate/1774159915361 --warmup-frames 4 --top 8 --sort time --json --out target/fret-diag-clap-smoke/triage.json`
- `target/debug/fretboard-dev diag triage target/fret-diag-ai-model-selector-focus-gate/1774159915361 --warmup-frames 4 --lite --metric total --json --out target/fret-diag-clap-smoke/triage.lite.json`
- `target/debug/fretboard-dev diag lint target/fret-diag-ai-model-selector-focus-gate/1774159915361 --warmup-frames 4 --json --out target/fret-diag-clap-smoke/check.lint.json`
- `target/debug/fretboard-dev diag ai-packet target/fret-diag-ai-model-selector-focus-gate/1774159915361 --warmup-frames 4 --test-id ai-model-selector-root --packet-out target/fret-diag-clap-smoke/ai.packet`
- `target/debug/fretboard-dev diag meta target/fret-diag-ai-model-selector-focus-gate/1774159915361 --out target/fret-diag-clap-smoke/bundle.meta.json`
- `target/debug/fretboard-dev diag index target/fret-diag-ai-model-selector-focus-gate/1774159915361 --out target/fret-diag-clap-smoke/bundle.index.json`
- `target/debug/fretboard-dev diag test-ids target/fret-diag-ai-model-selector-focus-gate/1774159915361 --max-test-ids 20 --out target/fret-diag-clap-smoke/test_ids.index.json`
- `target/debug/fretboard-dev diag test-ids-index target/fret-diag-ai-model-selector-focus-gate/1774159915361 --warmup-frames 4 --json`
- `target/debug/fretboard-dev diag frames-index target/fret-diag-ai-model-selector-focus-gate/1774159915361 --warmup-frames 4 --json`
- `target/debug/fretboard-dev diag compare --help`
- `target/debug/fretboard-dev diag dashboard --help`
- `target/debug/fretboard-dev diag stats --help`
- `target/debug/fretboard-dev diag summarize --help`
- `target/debug/fretboard-dev diag layout-perf-summary --help`
- `target/debug/fretboard-dev diag memory-summary --help`
- `target/debug/fretboard-dev diag inspect --help`
- `target/debug/fretboard-dev diag pick-arm --help`
- `target/debug/fretboard-dev diag pick --help`
- `target/debug/fretboard-dev diag pick-script --help`
- `target/debug/fretboard-dev diag pick-apply --help`
- `target/debug/fretboard-dev diag stats target/fret-diag-ai-model-selector-focus-gate/1774159915361 --top 5 --json`
- `target/debug/fretboard-dev diag stats --diff target/fret-diag-ai-model-selector-focus-gate/1774159915361 target/fret-diag-ai-model-selector-focus-gate/1774159915361 --top 5 --json`
- `target/debug/fretboard-dev diag stats --stats-lite-checks-json`
- `target/debug/fretboard-dev diag compare target/fret-diag-ai-model-selector-focus-gate/1774159915361 target/fret-diag-ai-model-selector-focus-gate/1774159915361 --json`
- `target/debug/fretboard-dev diag compare target/fret-diag-ai-model-selector-focus-gate/1774159915361 target/fret-diag-ai-model-selector-focus-gate/1774159915361 --footprint --json`
- `target/debug/fretboard-dev diag summarize target/fret-diag/campaigns/ui-gallery-smoke/1774499171270 --dir target/fret-diag-clap-smoke/summarize --json`
- `target/debug/fretboard-dev diag dashboard target/fret-diag/campaigns/ui-gallery-smoke/1774499171270/regression.index.json --top 3 --json`
- `target/debug/fretboard-dev diag layout-perf-summary target/fret-diag-ai-model-selector-focus-gate/1774159915361 --top 3 --json`
- `target/debug/fretboard-dev diag memory-summary target/fret-diag --top 3 --json`
- `target/debug/fretboard-dev diag inspect status`
- `target/debug/fretboard-dev diag inspect toggle --consume-clicks false`
- `target/debug/fretboard-dev diag pick-arm`
- `target/debug/fretboard-dev diag perf-baseline-from-bundles tools/diag-scripts/ui-gallery-image-object-fit-perf-steady.json target/fret-diag-ai-model-selector-focus-gate/1774159915361 --perf-baseline-out target/fret-diag-clap-smoke/perf.baseline.json --json`
- `target/debug/fretboard-dev diag matrix ui-gallery --dir target/fret-diag-matrix-smoke --timeout-ms 1 --poll-ms 1 --launch -- cargo run`
- `target/debug/fretboard-dev diag compare target/fret-diag-ai-model-selector-focus-gate/1774159915361 target/fret-diag-ai-model-selector-focus-gate/1774159915361 --compare-footprint` (expected parser rejection)
- `target/debug/fretboard-dev diag dashboard --warmup-frames 4` (expected parser rejection)
- `target/debug/fretboard-dev diag stats target/fret-diag-ai-model-selector-focus-gate/1774159915361 --check-prepaint-actions-min 1` (expected parser rejection)
- `target/debug/fretboard-dev diag summarize --top 7` (expected parser rejection)
- `target/debug/fretboard-dev diag layout_perf_summary --help` (expected retired-alias rejection)
- `target/debug/fretboard-dev diag memory_summary --help` (expected retired-alias rejection)
- `target/debug/fretboard-dev diag memory-summary --sort_key macos_physical_footprint_peak_bytes` (expected parser rejection)
- `target/debug/fretboard-dev diag pick-script --warmup-frames 4` (expected parser rejection)
- `target/debug/fretboard-dev diag matrix ui-gallery --launch-write-bundle-json --launch -- cargo run` (expected parser rejection)
- `target/debug/fretboard-dev diag perf-baseline-from-bundles tools/diag-scripts/ui-gallery-image-object-fit-perf-steady.json target/fret-diag --pack` (expected parser rejection)
- `target/debug/fretboard-dev diag windows target/fret-diag-ai-model-selector-focus-gate/1774159915361 --warmup-frames 4 --json`
- `target/debug/fretboard-dev diag dock-routing target/fret-diag-ai-model-selector-focus-gate/1774159915361 --warmup-frames 4 --json`
- `target/debug/fretboard-dev diag dock-graph target/fret-diag-ai-model-selector-focus-gate/1774159915361 --json`
- `target/debug/fretboard-dev diag screenshots target/fret-diag-prompt-input-docs-screenshot-fixed/screenshots/1774173600166-ui-gallery-ai-prompt-input-docs-tooltip-zinc-dark/manifest.json --json`
- `target/debug/fretboard-dev diag hotspots target/fret-diag-ai-model-selector-focus-gate/1774159915361 --warmup-frames 4 --lite --metric total --json --out target/fret-diag-clap-smoke/hotspots.lite.json`
- `target/debug/fretboard-dev diag bundle-v2 target/fret-diag-ai-model-selector-focus-gate/1774159915361 --mode changed --pretty --json --out target/fret-diag-clap-smoke/bundle.schema2.changed.json`
- `target/debug/fretboard-dev diag layout-sidecar target/fret-diag/data-table-guide-header-zinc-light-rerun2/1774491032828-ui-gallery-data-table-guide-header-zinc-light --print --out target/fret-diag-clap-smoke/layout.taffy.v1.json`
- `target/debug/fretboard-dev diag extensions target/fret-diag-snippet-docs-smoke/1774399732191-ui-gallery-ai-snippet-docs-smoke/bundle.schema2.json --out target/fret-diag-clap-smoke/extensions.json --json`
- `target/debug/fretboard-dev diag trace target/fret-diag-ai-model-selector-focus-gate/1774159915361 --trace-out target/fret-diag-clap-smoke/trace.chrome.json`
- `target/debug/fretboard-dev diag resolve latest --dir target/fret-diag-ai-model-selector-focus-gate --json`
- `target/debug/fretboard-dev diag query test-id target/fret-diag-ai-model-selector-focus-gate/1774159915361 ai --out target/fret-diag-clap-smoke/query.test-id.json --json`
- `target/debug/fretboard-dev diag query snapshots target/fret-diag-ai-model-selector-focus-gate/1774159915361 --test-id ai-model-selector-root --top 5 --out target/fret-diag-clap-smoke/query.snapshots.json --json`
- `target/debug/fretboard-dev diag slice target/fret-diag-ai-model-selector-focus-gate/1774159915361 --test-id ai-model-selector-root --out target/fret-diag-clap-smoke/slice.test-id.json --json`
- `target/debug/fretboard-dev diag meta target/fret-diag-ai-model-selector-focus-gate/1774159915361 --dir target/fret-diag` (expected parser rejection)
- `target/debug/fretboard-dev diag artifacts lint` (expected retired-alias rejection)
- `target/debug/fretboard-dev diag layout_sidecar --help` (expected retired-alias rejection)
- `target/debug/fretboard-dev diag pack --schema2-only` (expected retired-flag rejection)
- `target/debug/fretboard-dev diag triage target/fret-diag-ai-model-selector-focus-gate/1774159915361 --frames-index` (expected retired-flag rejection)
- `target/debug/fretboard-dev diag ai-packet ai-model-selector-root` (expected positional test-id rejection)
- `target/debug/fretboard-dev diag test-ids-index target/fret-diag-ai-model-selector-focus-gate/1774159915361 --out target/fret-diag-clap-smoke/test_ids.index.json` (expected parser rejection)
- `target/debug/fretboard-dev diag frames-index target/fret-diag-ai-model-selector-focus-gate/1774159915361 --out target/fret-diag-clap-smoke/frames.index.json` (expected parser rejection)
- `target/debug/fretboard-dev diag windows target/fret-diag-ai-model-selector-focus-gate/1774159915361 --out target/fret-diag-clap-smoke/window.map.json` (expected parser rejection)
- `target/debug/fretboard-dev diag dock-graph target/fret-diag-ai-model-selector-focus-gate/1774159915361 --warmup-frames 4` (expected parser rejection)
- `target/debug/fretboard-dev diag screenshots target/fret-diag-prompt-input-docs-screenshot-fixed/screenshots/1774173600166-ui-gallery-ai-prompt-input-docs-tooltip-zinc-dark/manifest.json --warmup-frames 4` (expected parser rejection)
- `target/debug/fretboard-dev diag hotspots target/fret-diag-ai-model-selector-focus-gate/1774159915361 --top 9` (expected parser rejection)
- `target/debug/fretboard-dev diag bundle-v2 target/fret-diag-ai-model-selector-focus-gate/1774159915361 --warmup-frames 4` (expected parser rejection)
- `target/debug/fretboard-dev diag query test_ids ai` (expected retired-alias rejection)

Smoke note:

- `diag doctor scripts --json` currently exits non-zero on a real repo drift, not on parser
  failure: one canonical script is still checked in at
  `tools/diag-scripts/ui-gallery-toggle-group-spacing-screenshots-zinc-light-dark.json` instead of
  being a redirect stub.

## 5) Ownership direction

Final merged ownership is recorded in `docs/workstreams/diag-cli-fearless-refactor-v1/OWNERSHIP.md`.

Preferred ownership split:

- `apps/fretboard` owns the top-level application shell,
- `crates/fret-diag` owns the `diag` command model and execution entrypoints,
- individual diagnostics command modules own their local argument/state shapes,
- shared diagnostics argument families live in focused shared modules rather than in one central blob.

This keeps the `diag` surface close to the crate that actually implements it while still allowing
`fretboard` to remain the single user-facing binary.

Recommended modular shape:

- `cli/` for `clap` types only,
- `cli/shared/` for reusable arg families such as launch/output/check/session/transport,
- `cli/commands/` for subcommand-local parser structs,
- a small conversion layer from `clap` types into internal command contexts,
- execution modules that do not directly depend on `clap`.

## 6) Working rules for this refactor

### 6.1 No compatibility lane

- Do not keep `diag_cmd(args: Vec<String>)` manual parsing as a retained fallback path.
- Do not merge parser-v2 while parser-v1 still handles real production inputs.
- Do not add "legacy alias" flags unless the final contract explicitly wants them.

### 6.2 Atomic first-party migration

- When the new parser lands, repo-owned examples/docs/tests should move with it.
- Do not leave the repo teaching deleted command shapes.
- If a document contains executable command examples, update those commands in the same migration window when they are touched by the reset.
- If a command example is intentionally kept for history, mark it as historical rather than letting it silently drift.

### 6.3 Parse first, execute second

- The first hard boundary to clean up is parse/model/validation.
- Execution refactors are allowed only when needed to support the new command model.

### 6.4 Prefer structural reuse over global state

- Shared argument families should be modeled once and reused by composition.
- Avoid reintroducing a new central "all flags live here" blob behind a different library.

### 6.5 Lock behavior with parser tests

- Invalid invocation shapes should fail in parser tests, not by accident in runtime smoke tests.
- Help output and representative examples should have snapshot coverage.

## 7) Deliverables

This workstream is complete only when the repo has:

- a new typed `diag` command model defined with `clap`,
- a deleted manual parser blob for `diag`,
- generated/help-driven usage text,
- parser regression tests for common valid and invalid invocations,
- updated first-party docs/examples/scripts and command snippets,
- a short closeout note describing the final command-model ownership.

## 8) Success criteria

The reset is successful when:

- contributors can add a new `diag` flag without editing a giant global match loop,
- users can tell from `--help` which combinations are valid,
- invalid command combinations fail immediately and intentionally,
- repo-owned diagnostics docs stop drifting away from the executable surface,
- there is no retained parser compatibility debt waiting to be cleaned up "later".

## 9) Closeout State

This workstream is no longer the place for broad parser cleanup.

The parser reset itself is complete:

- parser-v1 is deleted
- `clap` is the only parser/help source of truth
- migrated execution modules no longer own duplicated help branches

Residual work is intentionally split out:

- main execution lane hardening:
  - `docs/workstreams/diag-cli-main-lanes-hardening-v1/README.md`
- first-party caller migration:
  - `docs/workstreams/diag-cli-first-party-migration-v1/README.md`
- help snapshots and smoke gates:
  - `docs/workstreams/diag-cli-help-and-gates-v1/README.md`
