# fret-ui Layout Architecture Audit v1 - Closeout Audit

Date: 2026-05-18
Status: Closed

## Objective

Decide whether the current `fret-ui` layout/node classification and clean-geometry model should be
kept, reorganized, or redesigned before more performance work lands.

## Verdict

Close this audit lane.

The current clean-geometry model should not be redesigned now. Its axes are conceptually sound:

- layout effect,
- child-bounds strategy,
- width-delta size stability,
- explicit rejection attribution.

The problem was organization, not the model. FLA-040 landed the smallest behavior-preserving split:
clean-geometry proof code now lives in `crates/fret-ui/src/tree/layout/clean_geometry.rs`, while
ordinary per-node layout/measure execution remains in `crates/fret-ui/src/tree/layout/node.rs`.

## Evidence

- Source inventory:
  `docs/workstreams/fret-ui-layout-architecture-audit-v1/ARCHITECTURE_INVENTORY_2026-05-18.md`
- Architecture decision:
  `docs/workstreams/fret-ui-layout-architecture-audit-v1/ARCHITECTURE_DECISION_2026-05-18.md`
- Clean-geometry module:
  `crates/fret-ui/src/tree/layout/clean_geometry.rs`
- Node layout execution:
  `crates/fret-ui/src/tree/layout/node.rs`
- Local perf bundle:
  `target/fret-diag/layout-architecture-audit-v1-baseline-r1/1779077560550/bundle.schema2.json`
- Local perf stats:
  `target/fret-diag/layout-architecture-audit-v1-baseline-r1/worst.stats.json`

## Gates

Recorded passed gates:

- `cargo nextest run -p fret-ui layout_engine --no-fail-fast`
- `cargo nextest run -p fret-ui clean_geometry_small_resize_rejects_auto_height_text_reflow --no-fail-fast`
- `python3 tools/check_layering.py`
- `cargo fmt --check`
- `python3 -m json.tool docs/workstreams/fret-ui-layout-architecture-audit-v1/WORKSTREAM.json`
- `python3 tools/check_workstream_catalog.py`
- `git diff --check`

## Follow-On

The next runtime performance owner is split to:

- `docs/workstreams/retained-layout-orchestration-v1/`

That lane owns fresh evidence and possible fixes around retained-tree/barrier orchestration,
`Semantics` layout hotspots, root `Scroll` side-effect boundaries, and `ViewCache` scheduling.

Do not reopen this audit lane for clean-geometry expansion, wrapped text proof work, tiny `Canvas`
proofs, or measured-size model migration. Those are separate lanes only if fresh evidence makes them
dominant.
