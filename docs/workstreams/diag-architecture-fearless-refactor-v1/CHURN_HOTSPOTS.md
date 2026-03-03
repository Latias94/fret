# Diagnostics Architecture (Fearless Refactor v1) — Churn Hotspots

Status: Draft (workstream note)

This note lists “high churn” parts of `crates/fret-diag` so we can target refactors where they buy
the most long-term stability.

Data source: `git log -n 200 --name-only -- crates/fret-diag` (counts are “commits in the last 200
touching the file”, not LoC or author time).

## Top churn files (last 200 commits)

1. `crates/fret-diag/src/lib.rs` (40)
2. `crates/fret-diag/src/diag_perf.rs` (29)
3. `crates/fret-diag/src/diag_suite.rs` (25)
4. `crates/fret-diag/src/diag_stats.rs` (20)
5. `crates/fret-diag/src/commands/config.rs` (17)
6. `crates/fret-diag/src/stats.rs` (16)
7. `crates/fret-diag/src/script_tooling.rs` (16)
8. `crates/fret-diag/src/compare.rs` (14)
9. `crates/fret-diag/src/diag_stats/check_support.rs` (13)
10. `crates/fret-diag/src/diag_run.rs` (12)

## Why these churn

### `crates/fret-diag/src/lib.rs`

- It is both a CLI parser and a command router, so almost any new feature touches it.
- The long list of flags (especially check flags) makes “small additions” create large diffs.

Refactor direction:

- Keep parsing/dispatch thin in `lib.rs`; push command-specific parsing to `commands/*`.
- Introduce registries (suites/checks) so adding new behaviors does not widen the central dispatch.

### `crates/fret-diag/src/diag_perf.rs` and `crates/fret-diag/src/diag_suite.rs`

- These are “orchestrators” that combine suite resolution, launch wiring, artifact resolution,
  and post-run checks.
- Suite-specific policy (UI gallery, docking, etc.) tends to accrete here, even when it is not a
  general diagnostics concern.

Refactor direction:

- Extract suite resolution behind `registry/*` seams.
- Move suite-specific policy into explicit tables / policy modules, and eventually allow ecosystem
  contributions via an extension contract (M2).

### `crates/fret-diag/src/diag_run.rs` and `crates/fret-diag/src/commands/config.rs`

- This is where “check flags” and script meta defaults accumulate.
- The `RunChecks` struct is large and grows with every new gate.

Refactor direction:

- Define a `CheckRegistry` that owns:
  - check identifiers,
  - check kinds (lint/triage/perf/hotspots),
  - and the mapping from “requested checks” → “runner implementation”.

## Non-goals (for this workstream)

- We do not need to make churn “zero”; we need to concentrate churn into the correct layer and
  keep contract surfaces stable.
- We do not need to split crates prematurely; prefer clean module boundaries first.

