# Diag CLI Fearless Refactor v1 — TODO

Status: Closeout-ready
Last updated: 2026-03-26

Related:

- Main note: `docs/workstreams/diag-cli-fearless-refactor-v1/README.md`
- Closeout: `docs/workstreams/diag-cli-fearless-refactor-v1/CLOSEOUT.md`
- Milestones: `docs/workstreams/diag-cli-fearless-refactor-v1/MILESTONES.md`
- Ownership: `docs/workstreams/diag-cli-fearless-refactor-v1/OWNERSHIP.md`
- Parser model: `docs/workstreams/diag-cli-fearless-refactor-v1/PARSER_MODEL.md`
- Follow-ups: `docs/workstreams/diag-cli-fearless-refactor-v1/FOLLOWUPS.md`
- Current top-level CLI help shell: `apps/fretboard/src/cli.rs`
- Current diagnostics parser blob: `crates/fret-diag/src/lib.rs`
- Current diagnostics command modules: `crates/fret-diag/src/`

Tracking legend:

- `[ ]` open
- `[~]` in progress
- `[x]` done
- `[>]` moved to a narrower follow-up lane
- `[!]` blocked

ID format:

- `DCR-{area}-{nnn}`

## M0 — Scope freeze and break policy

- [x] DCR-docs-001 Create a dedicated workstream folder for the CLI reset.
- [x] DCR-docs-002 Write the main workstream note with an explicit no-compatibility decision.
- [x] DCR-docs-003 Write milestones and task tracker docs.
- [x] DCR-docs-004 Lock `clap` as the parser/help framework in the workstream docs.
- [x] DCR-inventory-004 Inventory current `diag` subcommands and group them into:
  - main execution lanes,
  - utility/reporting lanes,
  - script authoring lanes,
  - live inspect lanes.
- [x] DCR-inventory-005 Inventory shared flag families and mark which are currently duplicated.
- [x] DCR-inventory-006 Inventory repo-owned callers:
  - docs examples,
  - tests,
  - scripts,
  - maintainer notes,
  - CI or gate helpers.
- [x] DCR-inventory-008 Inventory repo documents that contain `fretboard-dev diag` command examples and will need command rewrites during the reset.
- [x] DCR-policy-007 Write a concise migration rule for first-party callers:
  - delete old syntax,
  - update callers atomically,
  - do not leave fallback parsing behind.

## M1 — Command model and ownership map

- [x] DCR-model-010 Decide the final ownership split for parser types:
  - `apps/fretboard`,
  - `crates/fret-diag`,
  - command-local modules,
  - shared arg-family modules.
- [x] DCR-model-011 Draft the top-level typed `clap` command tree for `fretboard-dev diag`.
- [x] DCR-model-012 Define shared structs for launch-related flags.
- [x] DCR-model-013 Define shared structs for output/artifact flags.
- [x] DCR-model-014 Define shared structs for diagnostics check flags.
- [x] DCR-model-015 Define shared structs for session/transport/devtools flags.
- [x] DCR-model-016 Decide where command-local semantic validation lives after parse.
- [x] DCR-model-017 Decide the final representation for `--launch -- <cmd...>`.
- [x] DCR-model-018 Decide which current aliases survive because they are intentional, not because they are old.
- [x] DCR-model-019 Write a parser module map so `clap` types, shared arg families, and execution contexts do not collapse into one new blob.

## M2 — Parser reset for the main execution lanes

- [x] DCR-core-020 Introduce `clap` scaffolding for `diag`.
- [>] DCR-core-021 Remaining `diag run` hardening moved to `docs/workstreams/diag-cli-main-lanes-hardening-v1/README.md`.
- [>] DCR-core-022 Remaining `diag suite` hardening moved to `docs/workstreams/diag-cli-main-lanes-hardening-v1/README.md`.
- [>] DCR-core-023 Remaining `diag repro` hardening moved to `docs/workstreams/diag-cli-main-lanes-hardening-v1/README.md`.
- [>] DCR-core-024 Remaining `diag repeat` hardening moved to `docs/workstreams/diag-cli-main-lanes-hardening-v1/README.md`.
- [x] DCR-core-025 Implement the new typed parser for `diag perf`.
- [x] DCR-core-026 Implement the new typed parser for `diag campaign`.
- [>] DCR-core-027 Remaining parser-local validation extraction for the main execution lanes moved to `docs/workstreams/diag-cli-main-lanes-hardening-v1/README.md`.
- [>] DCR-core-028 Remaining generated help/example hardening for the main execution lanes moved to `docs/workstreams/diag-cli-main-lanes-hardening-v1/README.md`.
- [>] DCR-core-029 Remaining parser coverage for valid main-lane invocations moved to `docs/workstreams/diag-cli-main-lanes-hardening-v1/README.md`.
- [>] DCR-core-030 Remaining parser coverage for invalid main-lane combinations moved to `docs/workstreams/diag-cli-main-lanes-hardening-v1/README.md`:
  - missing required values,
  - conflicting flags,
  - wrong subcommand-only flags,
  - malformed trailing launch command capture.

## M3 — Utility lane migration and parser-v1 deletion

- [x] DCR-util-040 Implement the new typed parser for list/reporting commands.
  `list`, `sessions`, `compare`, `dashboard`, `stats`, `summarize`, `layout-perf-summary`,
  `memory-summary`, `perf-baseline-from-bundles`, and `matrix` are now migrated.
- [x] DCR-util-040a Implement the new typed parser for `diag doctor`.
- [x] DCR-util-042 Implement the new typed parser for inspect/pick/script helper commands.
  `script`, `inspect`, `pick`, `pick-arm`, `pick-script`, and `pick-apply` are now migrated.
- [x] DCR-util-041 Implement the new typed parser for artifact/reporting commands.
  `trace` / `resolve` / `agent` / `path` / `poke` / `latest` / `compare` / `dashboard` / `stats` / `summarize` / `pack` / `triage` /
  `lint` / `artifact lint` / `meta` / `index` / `test-ids` / `test-ids-index` / `frames-index` /
  `windows` / `dock-routing` / `dock-graph` / `screenshots` / `hotspots` / `bundle-v2` /
  `layout-sidecar` / `layout-perf-summary` / `memory-summary` / `extensions` / `ai-packet` /
  `query` / `slice` / `registry` / `config` are now migrated through the `clap` cutover path.
- [x] DCR-util-043 Delete the old manual `diag` parse loop from `crates/fret-diag/src/lib.rs`.
- [x] DCR-util-044 Delete parser-only mutable state that existed solely to support parser-v1.
- [x] DCR-util-045 Remove duplicated post-parse arity/ownership checks that the new parser now declares structurally.
- [x] DCR-util-046 Remove stale hand-written usage text that can no longer drift from the executable surface.
  Root `fretboard` help now points to generated `diag --help`, and migrated execution modules no
  longer keep their own redundant `--help`/usage branches.

## M4 — Repo migration and hardening

- [>] DCR-repo-050 Remaining repo-owned docs migration moved to `docs/workstreams/diag-cli-first-party-migration-v1/README.md`.
- [>] DCR-repo-051 Remaining helper script or gate-doc migration moved to `docs/workstreams/diag-cli-first-party-migration-v1/README.md`.
- [>] DCR-repo-052 Remaining parser-sensitive test migration moved to `docs/workstreams/diag-cli-first-party-migration-v1/README.md`.
- [>] DCR-repo-053 Help snapshot coverage moved to `docs/workstreams/diag-cli-help-and-gates-v1/README.md`.
- [>] DCR-repo-054 Focused smoke coverage for the highest-risk command families moved to `docs/workstreams/diag-cli-help-and-gates-v1/README.md`:
  - `run`,
  - `suite`,
  - `repro`,
  - `perf`,
  - `campaign`.
- [>] DCR-repo-055 Remaining repo grep cleanup for deleted syntax moved to `docs/workstreams/diag-cli-first-party-migration-v1/README.md`.
- [>] DCR-repo-056 Remaining command-snippet migration moved to `docs/workstreams/diag-cli-first-party-migration-v1/README.md`.
  `script`, `trace`, `resolve latest`, `compare`, `dashboard`, `stats`, `summarize`,
  `layout-perf-summary`, `memory-summary`, `inspect`, `pick`, `pick-script`, `pick-apply`,
  `test-ids-index`, `frames-index`, `windows`, `dock-routing`, `dock-graph`, `screenshots`,
  `hotspots`, and `bundle-v2` docs/help were updated for the migrated surface; broader repo grep is
  still pending.

## M5 — Closeout

- [x] DCR-close-060 Write a short closeout status note once parser-v1 is deleted.
- [x] DCR-close-061 Record the final ownership map for future maintainers.
- [x] DCR-close-062 Split any remaining follow-up work into narrow lanes instead of reintroducing compatibility debt.
