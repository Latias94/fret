# Retained Layout Orchestration v1 - Milestones

Status: Closed
Last updated: 2026-05-18

## M0 - Scope And Evidence Freeze

Exit criteria:

- [x] Current audit lane is closed.
- [x] Follow-on scope is retained layout orchestration, not clean-geometry expansion.
- [x] First attribution target and stop conditions are explicit.

Primary evidence:

- `docs/workstreams/fret-ui-layout-architecture-audit-v1/CLOSEOUT_AUDIT_2026-05-18.md`
- `docs/workstreams/retained-layout-orchestration-v1/DESIGN.md`

## M1 - Fresh Attribution

Exit criteria:

- [x] Fresh diag bundle and stats are recorded.
- [x] The dominant owner is classified before implementation starts.
- [x] The lane explicitly records that no local implementation landed in RLO-020.

Primary gates:

- `fretboard-dev diag perf` resize-jitter baseline with ViewCache enabled.
- `target/release/fretboard-dev diag stats <bundle.schema2.json> --sort time --top 20`

Primary evidence:

- `target/fret-diag/retained-layout-orchestration-v1-baseline/1779080825844/bundle.schema2.json`
- `target/fret-diag/retained-layout-orchestration-v1-baseline/1779080825844/worst.stats.json`
- `target/fret-diag/retained-layout-orchestration-v1-baseline/1779080825844/layout-perf-summary.json`

## M2 - Smallest Safe Slice

Exit criteria:

- [x] One behavior-preserving orchestration fix lands, or the lane records why no fix should land.
- [x] `Scroll` side effects remain authoritative.
- [x] `ViewCache` contained relayout and explicit-root solve semantics remain intact.

Primary gates:

- `cargo nextest run -p fret-ui layout_engine scroll view_cache --no-fail-fast`
- `python3 tools/check_layering.py`
- `cargo fmt --check`

Primary evidence:

- `crates/fret-ui/src/tree/layout/clean_geometry.rs`
- `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`
- `target/fret-diag/retained-layout-orchestration-v1-rlo030-after/1779083266980/bundle.schema2.json`

## M3 - Closeout

Exit criteria:

- [x] Gate set is recorded.
- [x] Remaining work is completed, deferred, or split.
- [x] `WORKSTREAM.json` status and `HANDOFF.md` are updated.

Primary evidence:

- `docs/workstreams/retained-layout-orchestration-v1/CLOSEOUT_AUDIT_2026-05-18.md`
- `docs/workstreams/retained-layout-orchestration-v1/WORKSTREAM.json`
- `docs/workstreams/retained-layout-orchestration-v1/HANDOFF.md`
