# Diag CLI Fearless Refactor v1 — Milestones

Status: Closeout-ready
Last updated: 2026-03-26

Tracking doc: `docs/workstreams/diag-cli-fearless-refactor-v1/README.md`

## M0 — Scope freeze and break policy

Outcome:

- The repo has one explicit statement that this is a pre-release hard reset with no compatibility lane.

Deliverables:

- Workstream docs for scope, milestones, and tasks.
- A first-pass inventory of current `diag` subcommands, shared flags, and repo-owned callers.
- A written break policy for docs/tests/scripts migration.
- An explicit decision that `clap` is the parser/help framework for the final merged state.

Exit criteria:

- A maintainer can answer which surfaces may intentionally break and which first-party callers must be migrated atomically.

## M1 — Command model and ownership map

Outcome:

- The repo has one agreed structural command model for `fretboard diag`.

Deliverables:

- A typed `clap` subcommand tree design.
- Shared argument-family boundaries for launch/output/check/session/pack/transport/check flags.
- A clear ownership split between `apps/fretboard` and `crates/fret-diag`.
- A modular file-layout direction for parser-only code versus execution code.
- A checked-in parser-model note plus a test-only contract scaffold that locks the first migration
  batch before production cutover.

Exit criteria:

- Every current `diag` subcommand and shared flag family has an obvious home in the new model.

## M2 — Parser reset for the main execution lanes

Outcome:

- The highest-churn `diag` execution lanes are parsed by the new declarative command model.

Target commands:

- `run`
- `suite`
- `repro`
- `repeat`
- `perf`
- `campaign`

Deliverables:

- New `clap`-backed typed parsing for the main execution lanes.
- Parser-level validation for the most common argument constraints.
- Updated help/examples for those lanes.

Exit criteria:

- The main diagnostics execution paths no longer depend on the legacy manual parser.
- Residual main-lane validation/test hardening is tracked outside this lane in
  `docs/workstreams/diag-cli-main-lanes-hardening-v1/README.md`.

## M3 — Utility lane migration and parser-v1 deletion

Outcome:

- The remaining diagnostics utility commands are migrated and the old parser is gone.

Target commands:

- list/reporting: `list`, `dashboard`, `summarize`, `compare`, `stats`, `resolve`, `trace`, `doctor`
- artifact/reporting helpers: `artifact`, `pack`, `triage`, `lint`, `meta`, `index`, `test-ids`,
  `test-ids-index`, `frames-index`, `windows`, `dock-routing`, `dock-graph`, `screenshots`,
  `hotspots`, `bundle-v2`, `layout-sidecar`, `extensions`, `layout-perf-summary`, `memory-summary`,
  `ai-packet`, `query`, `slice`
- authoring/live helpers: `inspect`, `pick`, `pick-arm`, `pick-script`, `pick-apply`, `script`
- utility/control surfaces: `agent`, `path`, `poke`, `latest`, `sessions clean`,
  `perf-baseline-from-bundles`, `matrix`, `registry`, `config`

Current progress:

- `list` is already migrated to the new `clap` shell and cutover path.
- `doctor` is also migrated, including its nested `scripts` and `campaigns` helper commands.
- `script` is also migrated, including direct execution plus `normalize` / `upgrade` / `validate` /
  `lint` / `shrink`.
- `compare`, `dashboard`, `stats`, and `summarize` are also migrated through the new `clap` shell
  and cutover path, with canonical help surfaces aligned to the real execution contexts.
- `layout-perf-summary`, `memory-summary`, `inspect`, `pick`, `pick-arm`, `pick-script`, and
  `pick-apply` are also migrated through the new `clap` shell and cutover path, with old snake_case
  aliases explicitly rejected.
- `trace`, `resolve`, `pack`, `triage`, `lint`, `artifact lint`, `meta`, `index`, `test-ids`,
  `test-ids-index`, `frames-index`, `windows`, `dock-routing`, `dock-graph`, `screenshots`,
  `hotspots`, `bundle-v2`, `layout-sidecar`, `extensions`, `ai-packet`, `query`, and `slice` are
  also migrated through the new `clap` shell and cutover path.
- `agent`, `path`, `poke`, `latest`, `sessions clean`, `perf-baseline-from-bundles`, `matrix`,
  `registry`, and `config` are now also migrated through the new `clap` shell and cutover path.
- `crates/fret-diag/src/lib.rs` no longer carries parser-v1; `diag_cmd` now delegates directly to
  the canonical typed dispatcher, and the old `diag_simple_dispatch` helper is deleted.
- Migrated execution modules no longer ship their own duplicate usage/help branches; help ownership
  is centralized in the `clap` contract layer.
- `apps/fretboard/src/cli.rs` no longer duplicates the full `diag` usage surface in prose; callers
  are pointed to the generated `fretboard diag --help` contract instead.

Deliverables:

- Full migration of the remaining `diag` command family.
- Hard deletion of parser-v1 code paths.
- Removal of duplicated help text and parser-only state blobs tied to the deleted parser.

Exit criteria:

- There is no merged legacy `diag` parser path left in `main`.

## M4 — Repo migration and hardening

Outcome:

- The repository teaches only the new `diag` CLI surface and locks it with tests.

Deliverables:

- Updated first-party docs, examples, scripts, and maintainer notes.
- Updated repo command snippets for any documents that teach `fretboard diag` usage.
- Parser regression tests for representative valid and invalid invocations.
- Help snapshots or equivalent output guards for user-facing usage text.

Exit criteria:

- Repo-owned callers and repo-owned command examples no longer rely on deleted syntax, and future drift is caught by tests.
- This lane no longer owns those hardening tasks directly; they are split into:
  - `docs/workstreams/diag-cli-first-party-migration-v1/README.md`
  - `docs/workstreams/diag-cli-help-and-gates-v1/README.md`

## M5 — Closeout

Outcome:

- The workstream ends with a smaller, clearer CLI surface and no retained parser debt.

Deliverables:

- A closeout note or status update in this folder.
- A short ownership summary for future maintainers.
- Residual follow-ups, if any, split into narrower workstreams instead of reopening compatibility debt.

Exit criteria:

- `fretboard diag` is on the new model only.
- This lane has explicit closeout, ownership, and follow-up handoff docs.
