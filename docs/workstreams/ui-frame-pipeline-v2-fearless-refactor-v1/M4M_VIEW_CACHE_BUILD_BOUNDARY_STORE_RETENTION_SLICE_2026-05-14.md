# M4M View-Cache Build-Boundary Store Retention Slice

Date: 2026-05-14
Status: Landed as explicit retention decision

## Why

M4D consolidated element-runtime view-cache build-time rendered/next side maps into
`ViewCacheBuildBoundaryStore`, but intentionally left one final ownership question open:

- should the store migrate directly into `ViewBoundaryState`;
- should it become a different boundary-owned build store;
- or should it remain a separate mechanism with an explicit retention reason?

The correct decision for the current runtime is to retain `ViewCacheBuildBoundaryStore` inside
`WindowElementState`.

The store owns declarative build-boundary identity records keyed by `GlobalElementId`: cache keys,
state keys, authoring identities, subtree membership, action-route fallback roots, reuse roots,
key-mismatch roots, last-reused frame tracking, transitioned reuse roots, and the active
view-cache scope stack. Those records exist before retained-node runtime boundary state is
refreshed for the frame.

`ViewBoundaryState` is keyed by retained `NodeId`. That makes it the correct owner for runtime
layout/prepaint/paint boundary state, but not the current owner for rendered/next declarative build
identity. Cache-hit subtree liveness is revalidated during mount and can rebind recorded global
element membership to the current live retained nodes.

## Change

- Added a code-level retention note on `ViewCacheBuildBoundaryStore`.
- Added a focused runtime test proving that recorded `GlobalElementId` subtree membership can be
  revalidated and rebound to current `NodeId` entries on a cache-hit frame.
- Kept `ViewCacheBuildBoundaryStore` private to `WindowElementState`.
- Kept `ViewBoundaryState` focused on retained-node runtime boundary state:
  - layout dependency and dirty state,
  - typed prepaint outputs,
  - scene-fragment state,
  - boundary-node paint-cache entry metadata.

## Contract Decision

For Frame Pipeline v2, view-cache build-boundary ownership is now explicit:

- `ViewCacheBuildBoundaryStore` is the retained declarative build-boundary mechanism keyed by
  `GlobalElementId`;
- `ViewBoundaryState` remains the retained-node runtime boundary mechanism keyed by `NodeId`;
- mount-time live-node revalidation is the bridge between recorded build membership and current
  retained nodes;
- this store should only move into `ViewBoundaryState` if the runtime introduces a stable boundary
  identity that can truthfully represent both declarative build identity and retained-node runtime
  ownership without losing cache-hit revalidation semantics.

This is an accepted retention decision for the current architecture, not a compatibility path.

## What This Deletes Or Avoids

Deleted:

- no live runtime path is deleted in this slice; M4D already deleted the parallel flat side maps.

Avoided:

- forcing declarative rendered/next build identity into a `NodeId`-keyed runtime boundary store;
- duplicating build-boundary frames inside every `ViewBoundaryState`;
- weakening cache-hit subtree liveness by assuming previous retained nodes are still authoritative.

Retained intentionally:

- `WindowElementState::view_cache_build_boundaries` as the final build-boundary store for the
  current `GlobalElementId`-keyed declarative runtime.

## Evidence

Implementation anchors:

- `crates/fret-ui/src/elements/runtime.rs`
- `crates/fret-ui/src/declarative/mount.rs`
- `crates/fret-ui/src/elements/cx.rs`
- `crates/fret-ui/src/tree/view_boundary.rs`

Correctness gates:

```bash
cargo fmt --check
cargo check -p fret-ui --all-targets
cargo check -p fret-ui --features diagnostics --all-targets
cargo nextest run -p fret-ui \
  elements::runtime::tests::view_cache_build_boundary_store_advances_rendered_next_and_clears_frame_local_flags \
  elements::runtime::tests::view_cache_build_boundary_store_rebinds_global_membership_to_current_nodes \
  --no-fail-fast
cargo nextest run -p fret-ui \
  declarative::tests::core::view_cache_subtree_membership_includes_nested_cache_roots \
  declarative::tests::view_cache::view_cache_keep_alive_revalidates_recorded_membership_before_touching_stale_detached_elements \
  declarative::tests::view_cache::view_cache_inherits_model_observations_on_cache_hit_layout \
  --no-fail-fast
python3 tools/check_layering.py
python3 tools/check_workstream_catalog.py
python3 -m json.tool docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/WORKSTREAM.json >/dev/null
git diff --check
```

Current-state wording check:

```bash
rg -n "ViewCacheBuildBoundaryStore.*(still open|remains open)|build-boundary store still needs|final ViewBoundaryState ownership remains open" \
  docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/PROGRESS.md \
  docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/TODO.md \
  docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/MILESTONES.md \
  docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/EVIDENCE_AND_GATES.md \
  docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/WORKSTREAM.json \
  docs/adr/0327-frame-pipeline-v2-and-view-boundaries.md \
  docs/adr/IMPLEMENTATION_ALIGNMENT.md
```

Observed results:

- `cargo nextest run -p fret-ui ...view_cache_build_boundary_store...`: `2 passed, 940 skipped`.
- view-cache behavior gates: `3 passed, 939 skipped`.
- `cargo fmt --check`: passed.
- `cargo check -p fret-ui --all-targets`: passed.
- `cargo check -p fret-ui --features diagnostics --all-targets`: passed.
- `python3 tools/check_layering.py`: passed.
- `python3 tools/check_workstream_catalog.py`: passed.
- `WORKSTREAM.json` JSON validation: passed.
- `git diff --check`: passed.
- current-state wording check: no stale current-state references to unresolved
  `ViewCacheBuildBoundaryStore` ownership remain in first-open workstream/ADR docs.

## Remaining Work

- Decide layout aggregation/sweep env knobs in their owning workstreams.
- Consolidate or explicitly retain remaining internal low-level `contained_layout` flags/debug
  fields.
- Add a second non-code-editor proof surface before global closeout.
