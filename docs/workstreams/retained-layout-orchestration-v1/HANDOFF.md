# Retained Layout Orchestration v1 - Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

This lane is closed. RLO-030 landed the first smallest safe implementation slice from the
resize-jitter attribution: `Semantics` is now part of the clean-geometry propagation fast path,
matching its existing pure wrapper contract.

## Active Task

- Task ID: RLO-040
- Owner: codex
- Status: complete
- Files:
  - `crates/fret-ui/src/tree/layout/clean_geometry.rs`
  - `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`
  - `docs/workstreams/retained-layout-orchestration-v1/{TODO.md,EVIDENCE_AND_GATES.md,MILESTONES.md,HANDOFF.md,WORKSTREAM.json,CLOSEOUT_AUDIT_2026-05-18.md}`
- Validation:
  - `cargo nextest run -p fret-ui clean_geometry_small_resize_propagates_through_semantics_wrapper --no-fail-fast`
  - `cargo nextest run -p fret-ui layout_engine scroll view_cache --no-fail-fast`
  - `python3 tools/check_layering.py`
  - `cargo fmt --check`
  - `git diff --check`
  - `python3 -m json.tool docs/workstreams/retained-layout-orchestration-v1/WORKSTREAM.json`
  - `python3 tools/check_workstream_catalog.py`
  - after `fretboard-dev diag perf` resize-jitter bundle

## Decisions Since Last Update

- Do not redesign the clean-geometry model now.
- Do not widen clean-geometry to wrapped text, root `Scroll`, or `Canvas` by default.
- Treat root `Scroll` as a side-effect boundary until a dedicated proof says otherwise.
- Treat `ViewCache` as a retained/cache boundary with contained relayout semantics, not a pure
  geometry wrapper.
- The dominant owner in the fresh baseline is retained root solve scheduling / root orchestration
  around the `Semantics` root, not `ViewCache` or `Scroll`.
- RLO-030 fixed the mismatch where `Semantics` was already a pure wrapper in the clean-geometry
  contract but was missing from the execution-side supported-element matrix.
- The after perf win is layout/root orchestration work, not Taffy solve work: `p95.layout_time_us`
  moved from `2479` to `885`, while `p95.layout_engine_solve_time_us` stayed roughly flat
  (`220` to `214`).

## Blockers

- None.

## Next Recommended Action

Open a new lane if continuing. The after layout summary still shows possible owners, but they
should not be bundled into this closed workstream:

1. `Pressable` wrapper propagation: after-sample top layout hotspot was a `Pressable` wrapper, but
   this needs its own proof because `Pressable` owns hit/focus/interaction side effects.
2. `Scroll` boundary cost: still secondary and must preserve scroll extent/handle side effects.
3. `ViewCache` boundary cost: still stable in reuse, but any optimization must preserve contained
   relayout and explicit-root solve semantics.
