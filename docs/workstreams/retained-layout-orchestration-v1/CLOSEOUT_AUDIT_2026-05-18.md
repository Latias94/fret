# Retained Layout Orchestration v1 - Closeout Audit

Date: 2026-05-18
Status: Closed

## Objective

Use fresh resize-jitter diagnostics to decide whether retained layout orchestration had a narrow,
behavior-preserving implementation slice after the layout architecture audit split. If a safe slice
existed, land it with correctness gates and perf evidence; otherwise record why no fix should land.

## Verdict

Close this lane.

RLO-020 identified retained root orchestration around a `Semantics` root as the dominant local
layout owner. RLO-030 found a precise contract/execution mismatch:

- `clean_geometry_node_contract(...)` already classified `ElementInstance::Semantics(_)` as a pure
  `PreserveLocalOrigins` wrapper.
- `clean_engine_geometry_propagation_supported_element(...)` did not allow `Semantics` to execute
  the propagation fast path, causing wrapper/subtree layout work after the root solve was skipped.

The fix was intentionally small: add `ElementInstance::Semantics(_)` to the supported-element
matrix and lock it with a targeted resize test.

## Evidence

Implementation and tests:

- `crates/fret-ui/src/tree/layout/clean_geometry.rs`
- `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`
  (`clean_geometry_small_resize_propagates_through_semantics_wrapper`)

Perf evidence:

- Baseline bundle:
  `target/fret-diag/retained-layout-orchestration-v1-baseline/1779080825844/bundle.schema2.json`
- After bundle:
  `target/fret-diag/retained-layout-orchestration-v1-rlo030-after/1779083266980/bundle.schema2.json`
- `diag stats --diff` highlights:
  - `p95.total_time_us`: `3050 -> 1442` (`-52.7%`)
  - `p95.layout_time_us`: `2479 -> 885` (`-64.3%`)
  - `p95.layout_roots_time_us`: `2349 -> 747` (`-68.2%`)
  - `p95.layout_engine_solve_time_us`: `220 -> 214` (`-2.7%`)

The win is retained wrapper/subtree layout avoidance, not a material Taffy solve-time change.

## Gates

Recorded passed gates:

- `cargo nextest run -p fret-ui clean_geometry_small_resize_propagates_through_semantics_wrapper --no-fail-fast`
- `cargo nextest run -p fret-ui layout_engine scroll view_cache --no-fail-fast`
- `python3 tools/check_layering.py`
- `cargo fmt --check`
- `git diff --check`
- `python3 -m json.tool docs/workstreams/retained-layout-orchestration-v1/WORKSTREAM.json`
- `python3 tools/check_workstream_catalog.py`

## Boundary Decision

No component policy changed. The implementation stayed in `crates/fret-ui` mechanism code and did
not treat `Scroll` or `ViewCache` as pure wrappers.

`Scroll` remains a side-effect boundary. `ViewCache` remains a retained/cache boundary with
contained relayout and explicit-root solve semantics.

## Follow-On Policy

Do not reopen this lane for broad clean-geometry expansion. Open a new, narrower lane only when the
next owner has its own proof:

- `Pressable` wrapper propagation: promising after-sample hotspot, but it must separately prove hit,
  focus, and interaction side effects remain authoritative.
- `Scroll` boundary cost: may be worth attributing, but do not skip scroll layout by name.
- `ViewCache` boundary cost: only optimize with explicit evidence that contained relayout and
  explicit-root solve semantics stay intact.
