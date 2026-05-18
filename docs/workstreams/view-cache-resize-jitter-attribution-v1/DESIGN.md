# ViewCache Resize-Jitter Attribution v1

Status: Active
Last updated: 2026-05-18

## Why This Lane Exists

`pressable-clean-geometry-propagation-v1` closed after proving that `Pressable` can safely use the
clean-geometry propagation fast path. Fresh UI Gallery resize-jitter evidence moved `Pressable` off
the worst-frame layout hotspot list, but it surfaced `ViewCache` as the largest remaining retained
layout owner in that same local scenario.

`ViewCache` is not a pure wrapper today. It owns cache-boundary identity, reuse, liveness,
contained-layout repair, boundary dirty tracking, and scroll follow-up scheduling. That makes it a
poor candidate for a direct allowlist-style clean-geometry change without a separate attribution and
source audit.

## Relevant Authority

- ADRs:
  - `docs/adr/0076-declarative-layout-performance-hardening.md`
  - `docs/adr/0176-view-cache-root-liveness-contract.md`
  - `docs/adr/0216-cache-root-tracing-contract-v1.md`
  - `docs/adr/0224-view-cache-subtree-reuse-and-state-retention.md`
- Existing docs:
  - `docs/runtime-contract-matrix.md`
  - `docs/workstreams/fret-ui-layout-architecture-audit-v1/CLOSEOUT_AUDIT_2026-05-18.md`
  - `docs/workstreams/retained-layout-orchestration-v1/CLOSEOUT_AUDIT_2026-05-18.md`
  - `docs/workstreams/pressable-clean-geometry-propagation-v1/CLOSEOUT_AUDIT_2026-05-18.md`
  - `docs/workstreams/pressable-clean-geometry-propagation-v1/EVIDENCE_AND_GATES.md`
- Related workstreams:
  - `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/`
  - `docs/workstreams/ui-gallery-view-cache-web-perf-stabilization-v1/`
  - `docs/workstreams/subtree-layout-dirty-fearless-refactor-v1/`

## Problem

The current local resize-jitter summary reports:

```text
ViewCache layout_us=380 inclusive_us=723
Scroll layout_us=205 inclusive_us=331
Flex layout_us=83 inclusive_us=122
```

This makes `ViewCache` the top remaining retained layout hotspot after the `Pressable` slice, but
the metric alone does not explain whether the cost comes from:

- legitimate cache-root contained relayout and follow-up scheduling,
- unnecessary layout invalidation breadth under cache-root reuse,
- missing propagation for a bounded wrapper-like case,
- diagnostics attribution collapsing multiple phases onto the `ViewCache` host node,
- or a demo-specific composition issue in the UI Gallery shell.

## Target State

This lane should produce a source-backed owner verdict before any runtime optimization:

- `ViewCache` remains a side-effect boundary, with evidence showing the current cost is legitimate
  or belongs to another owner.
- Or a narrow runtime slice is identified with a precise invariant, focused RED/GREEN proof, and no
  cache-liveness or scroll-range regression.
- Or the lane records that current diagnostics cannot distinguish the owner cleanly enough, and a
  diagnostics attribution slice must come first.

The target is attribution quality and safe next-step selection, not a broad view-cache rewrite.

## In Scope

- `crates/fret-ui/src/element.rs`
  - `ViewCacheProps`
- `crates/fret-ui/src/elements/runtime.rs`
  - `ViewCacheBuildBoundaryStore`
  - cache-root liveness, membership, key, and reuse bookkeeping
- `crates/fret-ui/src/tree/view_boundary.rs`
  - `ViewBoundaryKind::ViewCacheRoot`
  - layout dependency metadata
- `crates/fret-ui/src/tree/layout/entrypoints.rs`
  - `expand_view_cache_layout_invalidations_if_needed`
  - `repair_view_cache_root_bounds_from_engine_if_needed`
  - `layout_contained_view_cache_roots_if_needed`
  - `collapse_layout_observations_to_view_cache_roots_if_needed`
- `crates/fret-ui/src/tree/layout/clean_geometry.rs`
  - current `ElementInstance::ViewCache(_)` side-effect boundary classification
- UI Gallery resize-jitter diagnostics evidence.

## Out Of Scope

- Directly adding `ViewCache` to the clean-geometry execution allowlist.
- Combining `Scroll` optimization with `ViewCache` attribution.
- Replacing the view-cache model with a new retained/declarative architecture in this lane.
- Changing shadcn recipe policy, code-editor row replay policy, renderer batching, or GPU paint.
- Universal performance claims across machines or GPUs.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| This should be a narrow follow-on, not a reopening of the closed retained-layout lanes. | Confident | `pressable-clean-geometry-propagation-v1` closeout says future `ViewCache` and `Scroll` work need separate owner-specific lanes. | Reopening the old lane would mix shipped wrapper proofs with cache-boundary semantics. |
| `ViewCache` is currently a side-effect boundary in clean geometry. | Confident | `crates/fret-ui/src/tree/layout/clean_geometry.rs` maps `ElementInstance::ViewCache(_)` to `side_effect_boundary()`. | Any allowlist-style change would need a stronger proof than `Semantics` or `Pressable`. |
| The fresh `Pressable` after bundle is enough to justify attribution work, not enough to justify a runtime change. | Confident | `target/fret-diag/pressable-clean-geometry-propagation-v1-pgp050-after/layout.perf.summary.v1.json` shows `ViewCache` as the top layout hotspot but does not split view-cache phases. | A runtime change without phase attribution could optimize the wrong owner. |
| `Scroll` must stay out of this lane. | Likely | The same evidence lists `Scroll` as a separate second hotspot, and `entrypoints.rs` schedules scroll follow-up relayout after contained view-cache relayout. | Combining them would blur whether stale extents, cache-root repair, or scroll policy owns the cost. |
| The first executable slice should freeze evidence and source risk before modifying runtime code. | Confident | `ViewCache` owns reuse/liveness/key bookkeeping and contained relayout, unlike pure visual wrappers. | Skipping the audit could regress cache-root reuse or state retention while appearing to improve one frame. |

## Architecture Direction

Treat `ViewCache` as a boundary primitive first and a performance owner second:

1. Separate phase attribution from optimization. A `ViewCache` hotspot could mean root-bound repair,
   contained relayout, invalidation breadth collapse, follow-up scheduling, or incomplete diagnostics.
2. Preserve `clean_geometry_node_contract(...)` as the safety model. Do not move `ViewCache` out of
   `side_effect_boundary()` until a narrower invariant is proven.
3. Prefer a small diagnostics or contract proof if the current evidence cannot distinguish owner
   phases.
4. Keep runtime policy in `fret-ui` limited to mechanism-level cache-boundary behavior. Demo
   composition, recipe sizing, and scroll-area policy stay outside this lane unless evidence proves
   they are the actual owner.

## Closeout Condition

This lane can close when it leaves one of these explicit verdicts:

- no runtime change, with source-backed explanation of why `ViewCache` cost is legitimate or belongs
  to another owner;
- a minimal runtime change, with focused regression tests and fresh perf evidence;
- a diagnostics-first follow-on, with a clear missing attribution field or phase split.

In all cases the lane must preserve cache-root liveness, state retention, boundary tracing, and
scroll extent correctness.
