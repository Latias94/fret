# fret-ui Layout Architecture Audit v1 - Handoff

Status: Closed
Updated: 2026-05-18

## Current state

This audit lane is closed.

Final decision:

- The clean-geometry model is conceptually on the right track.
- The organization risk is now reduced: clean-geometry proof code is isolated from ordinary
  per-node layout/measure execution.
- The next runtime performance owner is split to
  `docs/workstreams/retained-layout-orchestration-v1/`.

## Completed

FLA-010 is complete:

- `docs/workstreams/fret-ui-layout-architecture-audit-v1/ARCHITECTURE_INVENTORY_2026-05-18.md`

FLA-020 and FLA-030 are complete:

- Bundle:
  `target/fret-diag/layout-architecture-audit-v1-baseline-r1/1779077560550/bundle.schema2.json`
- Stats:
  `target/fret-diag/layout-architecture-audit-v1-baseline-r1/worst.stats.json`
- Decision:
  `docs/workstreams/fret-ui-layout-architecture-audit-v1/ARCHITECTURE_DECISION_2026-05-18.md`

Verdict: the current model axes are conceptually sound. The main refactor candidate is a
behavior-preserving extraction of clean-geometry proof code from `tree/layout/node.rs`, not a full
layout model redesign. The local perf owner, if pursued, is retained layout orchestration around
`Semantics` / `Scroll` / `ViewCache`, not Taffy solve or text prepare.

FLA-040 is complete:

- `crates/fret-ui/src/tree/layout/clean_geometry.rs`
- `crates/fret-ui/src/tree/layout/node.rs`
- `crates/fret-ui/src/tree/layout/mod.rs`

Validation passed:

- `cargo nextest run -p fret-ui layout_engine --no-fail-fast`
- `cargo nextest run -p fret-ui clean_geometry_small_resize_rejects_auto_height_text_reflow --no-fail-fast`
- `python3 tools/check_layering.py`
- `cargo fmt --check`
- `python3 -m json.tool docs/workstreams/fret-ui-layout-architecture-audit-v1/WORKSTREAM.json`
- `python3 tools/check_workstream_catalog.py`
- `git diff --check`

## Next task

Resume `docs/workstreams/retained-layout-orchestration-v1/` if continuing runtime performance.
Start with fresh diag attribution before redesigning root `Scroll` or retained layout scheduling.

Useful anchors:

- `crates/fret-ui/src/tree/layout/node.rs`
- `crates/fret-ui/src/tree/layout/clean_geometry.rs`
- `crates/fret-ui/src/tree/layout/solve.rs`
- `crates/fret-ui/src/tree/layout/entrypoints.rs`
- `crates/fret-ui/src/layout/engine.rs`
- `docs/workstreams/scroll-optimization-v1/HANDOFF.md`
- `docs/workstreams/retained-layout-orchestration-v1/`

## Do not do yet

- Do not reopen `scroll-optimization-v1` for clean-geometry expansion.
- Do not treat wrapped text as a fast-path candidate without a dedicated line-break/computed-box
  proof.
- Do not start a layout model rewrite from this lane without fresh evidence that the current
  classification axes are insufficient.
