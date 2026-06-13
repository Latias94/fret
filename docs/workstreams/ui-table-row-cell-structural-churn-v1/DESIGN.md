# Design: UI Table Row/Cell Structural Churn v1

Status: Active
Last updated: 2026-06-13

## Problem

The prior `ui-layout-dirty-breadth-data-table-v1` lane closed after removing avoidable page-cache
breadth, input chrome motion, and a redundant runtime structural walk. The remaining data-table
cost is now inside the contained table subtree.

Fresh general-app perf evidence shows that this is still the main gap for shadcn-heavy application
surfaces:

- data-table view-cache/filter/vlist script: `top_total p95/max=17068/17068us`,
  `layout p95=15336us`, renderer encode/text p95 only `178/121us`;
- virtual-list torture script: `top_total p95/max=9311/9311us`, `layout p95=8232us`,
  renderer encode/text p95 only `395/125us`;
- context menu and overlay pointer probes are closer to budget, so the next high-leverage owner is
  list/table layout churn, not GPU throughput.

This lane owns the narrow question: can the table row/cell subtree reduce structural churn on
filter/sort/window-membership changes, or is the current cost required for correctness?

## Assumptions-first pass

- Area: lane status
  - Assumption: this is a new narrow follow-on, not a reopen of
    `ui-layout-dirty-breadth-data-table-v1`.
  - Evidence: `docs/workstreams/ui-layout-dirty-breadth-data-table-v1/CLOSEOUT_AUDIT_2026-05-15.md`
    explicitly says future row/cell structural churn work should use a narrower follow-on.
  - Confidence: Confident.
  - Consequence if wrong: new work could blur the closed page-cache/input-chrome/runtime fastpath
    evidence.

- Area: primary bottleneck
  - Assumption: the current data-table and virtual-list perf gap is layout-root breadth and row
    subtree solve work, not renderer encode, text prepare, or GPU upload.
  - Evidence: the 2026-06-13 general-app bundles report data-table layout p95 `15336us` versus
    renderer encode/text p95 `178/121us`, and virtual-list layout p95 `8232us` versus
    renderer encode/text p95 `395/125us`.
  - Confidence: Confident.
  - Consequence if wrong: the first optimization slice would target the wrong phase.

- Area: ownership
  - Assumption: row/cell rendering and table policy belong in `ecosystem/fret-ui-kit`, while
    runtime-level VirtualList/cache semantics stay mechanism-owned in `crates/fret-ui`.
  - Evidence: ADR 0066 keeps `fret-ui` as mechanism/contract surface; table rendering hotspots are
    in `ecosystem/fret-ui-kit/src/declarative/table.rs`.
  - Confidence: Confident.
  - Consequence if wrong: a policy fix could leak into runtime or a mechanism change could be hidden
    in table code.

- Area: first slice
  - Assumption: the first implementation should be attribution-first and reversible: identify the
    row/cell subtree shape that creates the `first_solve` / `batch_roots` tail, then remove one
    unnecessary wrapper, key instability, or per-row rebuild path only if the bundle proves it.
  - Evidence: data-table top solve shows `Pressable` at `table.rs:8058` with `batch_roots=33`,
    `subtree_nodes=297`, and `first_solve`; virtual-list shows a similar `batch_roots=35`,
    `subtree_nodes=455`, `measure_calls=210` pattern.
  - Confidence: Likely.
  - Consequence if wrong: the lane should instead split to VirtualList retained reconciliation or
    command availability.

## Goals

1. Attribute the remaining data-table row/cell layout churn with node-level layout evidence.
2. Reduce one measured row/cell structural-churn source, or record a clear no-change verdict if the
   structure is required.
3. Preserve the mechanism/policy split:
   - `crates/fret-ui`: VirtualList/cache/layout mechanisms only when proven table-agnostic;
   - `ecosystem/fret-ui-kit`: table row/cell rendering, grouping, pinning, row selection, keyboard
     policy;
   - `ecosystem/fret-ui-shadcn`: data-table recipes and app-facing shadcn composition.
4. Keep retained/view-cache table correctness gates green.
5. Leave before/after bundles and `diag stats` summaries in `EVIDENCE_AND_GATES.md`.

## Non-goals

- No renderer rewrite.
- No new public runtime API unless an ADR update explicitly justifies it.
- No reopening page-cache containment or shadcn Input chrome motion.
- No broad "make all lists faster" refactor without row/cell evidence.
- No checked-in baseline relaxation from local noisy runs.

## Target architecture

The desired end state is:

```text
table interaction changes row/window membership
  -> row keys and row layout contracts remain stable
  -> only changed visible rows/cells rebuild or solve
  -> table subtree avoids redundant per-row wrapper roots
  -> cache/VirtualList diagnostics explain remaining solves
  -> general-app data-table p95 moves toward the 120Hz budget
```

The important distinction is between legitimate row/window membership work and avoidable structural
work. Filtering a 50k-row data set to 111 rows can legitimately change visible row membership, but
it should not force avoidable wrapper roots, unstable keys, or repeated first-solve work for rows
whose geometry contract is fixed.

## ADR trigger conditions

No new ADR is required for:

- private table rendering cleanup;
- new focused table/VirtualList tests;
- documentation of perf evidence;
- diagnostics-only use of existing bundle fields.

Add or update an ADR if the lane changes:

- public `fret-ui` runtime APIs;
- VirtualList cache semantics;
- ViewCache invalidation semantics;
- diagnostics schema consumed outside first-party tooling.

## Completion criteria

- The lane has a recorded baseline bundle and node-level layout attribution.
- At least one implementation slice has before/after evidence, or the no-change verdict is explicit.
- Focused table correctness gates pass.
- `python tools/check_workstream_catalog.py`, `python -m json.tool WORKSTREAM.json`, and
  `git diff --check` pass.
- A closeout audit records the final owner decision or splits the next narrower follow-on.
