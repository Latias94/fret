# Retained Layout Orchestration v1

Status: Closed
Last updated: 2026-05-18

Status note (2026-05-18): this lane closed after RLO-030 landed the narrow Semantics wrapper
clean-geometry propagation fix and RLO-040 recorded closeout evidence. Remaining after-sample
owners (`Pressable`, `Scroll`, `ViewCache`) should start as narrower follow-ons rather than
reopening this lane.

## Why This Lane Exists

`fret-ui-layout-architecture-audit-v1` closed with a clear result: do not redesign the
clean-geometry model now. The local resize-jitter evidence instead points at retained
tree/barrier orchestration around `Semantics`, root `Scroll`, and `ViewCache`.

This lane owns that next performance question without reopening clean-geometry expansion.

## Relevant Authority

- ADRs:
  - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
  - `docs/adr/0076-declarative-layout-performance-hardening.md`
  - `docs/adr/0213-cache-roots-and-cached-subtree-semantics-v1.md`
  - `docs/adr/0224-view-cache-subtree-reuse-and-state-retention.md`
- Existing docs:
  - `docs/workstreams/fret-ui-layout-architecture-audit-v1/CLOSEOUT_AUDIT_2026-05-18.md`
  - `docs/workstreams/scroll-optimization-v1/HANDOFF.md`
- Code anchors:
  - `crates/fret-ui/src/tree/layout/entrypoints.rs`
  - `crates/fret-ui/src/tree/layout/solve.rs`
  - `crates/fret-ui/src/tree/layout/node.rs`
  - `crates/fret-ui/src/tree/layout/clean_geometry.rs`
  - `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`
  - `crates/fret-ui/src/layout/engine.rs`

## Problem

Current local evidence says the remaining resize-jitter cost is layout-heavy, but not dominated by
Taffy solve or text measurement. The worst sample showed:

- `total=2803us`
- `layout=2304us`
- `layout_roots=2181us`
- `layout_engine_solve=202us`
- top retained/layout hotspots around `Semantics`, `Scroll`, and `ViewCache`

That shape needs a retained orchestration lane: attribution first, then a narrowly proven fix.

## Target State

This lane closed after:

- fresh diagnostics identified whether the current owner was `Semantics`, root `Scroll`, `ViewCache`,
  or a different retained layout scheduling edge;
- the first implementation slice preserved `Scroll` side effects and `ViewCache`
  containment semantics;
- clean-geometry remained a bounded proof module, not the catch-all owner for retained layout cost;
- gates proved correctness before perf claims.

## In Scope

- Frame/layout orchestration in `tree/layout/entrypoints.rs` and `tree/layout/solve.rs`.
- Root `Scroll` side-effect boundary attribution and scheduling behavior.
- `ViewCache` retained layout scheduling, contained relayout, and boundary repair.
- Diagnostics that explain retained layout hot spots and root solve reasons.
- A smallest behavior-preserving implementation slice if fresh evidence identifies one.

## Out Of Scope

- Widening clean-geometry proofs by default.
- Wrapped text computed-box/line-break stability proofs.
- Tiny `Canvas` proof work unless fresh evidence makes it non-trivial.
- Broad `measured_size: Option<Size>` migration.
- Component policy or shadcn recipe changes.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Taffy solve is not the dominant current owner | Medium | FLA-020 `layout_engine_solve=202us` in a layout-heavy worst frame | Reopen layout engine attribution before touching orchestration |
| Root `Scroll` must remain a side-effect boundary | High | `scroll-optimization-v1/HANDOFF.md`; clean-geometry rejection guards | Do not skip `Scroll` layout by name |
| `ViewCache` participation is boundary-specific, not pure geometry | High | FLA-040 gates and `scroll-optimization-v1` ViewCache boundary notes | Preserve explicit root solves and contained relayout semantics |
| `Semantics` hotspot may be inclusive orchestration cost | Medium | FLA-020 top hotspot attribution | First task must separate inclusive vs exclusive/root owner cost |

## Architecture Direction

Keep ownership in `crates/fret-ui` retained-tree orchestration. This is a mechanism lane. Policy
layers should not be changed to hide runtime scheduling cost.

The first task is evidence-first. A fix should only land after the diagnostics show which step is
avoidable: root collection, solve scheduling, contained relayout, post-layout side effects, or
debug/hotspot accounting.

## Closeout Condition

This lane can close when:

- fresh diag evidence is recorded,
- one smallest safe owner is either implemented or explicitly deferred,
- correctness gates pass,
- and any remaining owner is split to a narrower follow-on.
