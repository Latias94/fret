# Diag CLI First-party Migration v1

Status: In progress
Last updated: 2026-03-26

Source lane:

- `docs/workstreams/diag-cli-fearless-refactor-v1/FOLLOWUPS.md`

## Scope

Finish the repo-owned caller migration after the parser reset landed.

This lane covers docs, helper scripts, parser-sensitive tests, and repo-wide grep cleanup for
deleted command spellings.

## Landed slice

The first migration slice updates repo-owned `diag perf` callers and command snippets to the
current prewarm/prelude surface:

- `tools/perf/diag_perf_baseline_select.py` now forwards `--prewarm-script`,
  `--prelude-script`, and `--prelude-each-run`
- current maintainer workstream docs now teach the renamed `diag perf` flags instead of the deleted
  suite-prefixed prewarm/prelude spellings
- historical logs were intentionally left unchanged in this slice

## Carries

- `DCR-repo-050`
- `DCR-repo-051`
- `DCR-repo-052`
- `DCR-repo-055`
- `DCR-repo-056`

## Exit criteria

- repo-owned docs teach only the intended CLI surface
- helper scripts and maintainer notes no longer rely on deleted spellings
- parser-sensitive tests are updated to the intended surface
- repo grep no longer finds stale deleted syntax except where explicitly labeled historical

## Repro, Gate, Evidence

Gate commands:

- `python3 tools/perf/diag_perf_baseline_select.py --help`
- `cargo nextest run -p fret-diag cli::contracts::tests:: cli::cutover::tests::`
- `cargo build -p fretboard --message-format short`

Evidence anchors:

- `tools/perf/diag_perf_baseline_select.py`
- `docs/workstreams/diag-v2-hardening-and-switches-v1/README.md`
- `docs/workstreams/diag-v2-hardening-and-switches-v1/todo.md`
- `docs/workstreams/ui-perf-windows-rtx4090-smoothness-v1/ui-perf-windows-rtx4090-smoothness-v1.md`
- `docs/workstreams/standalone/ui-perf-resize-path-v1.md`
