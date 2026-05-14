# M4D View-Cache Build-Boundary Store Slice - 2026-05-14

Status: landed as a runtime ownership-consolidation slice; no new perf claim.

## Summary

This slice consolidates view-cache build-time bookkeeping inside `WindowElementState`.

Before this slice, the element runtime carried separate rendered/next maps and side sets for:

- view-cache cache keys,
- state keys touched by cached subtrees,
- authoring identities touched by cached subtrees,
- recorded subtree membership,
- key mismatch roots,
- reuse roots,
- last-reused frame tracking,
- transitioned reuse roots,
- and the active view-cache scope stack.

After this slice, those maps are grouped behind a single internal
`ViewCacheBuildBoundaryStore` with per-root `ViewCacheBuildBoundaryFrame` records. Existing
runtime methods still expose the same mechanism surface to `ElementContext` and declarative mount,
so this is an ownership consolidation step rather than an authoring or behavior change.

## Code Changes

- `crates/fret-ui/src/elements/runtime.rs`
  - adds `ViewCacheBuildBoundaryStore` and `ViewCacheBuildBoundaryFrame`,
  - replaces the flat `WindowElementState` view-cache rendered/next maps and frame-local reuse
    sets with `view_cache_build_boundaries`,
  - preserves existing method names such as `begin_view_cache_scope(...)`,
    `touch_view_cache_state_keys_if_recorded(...)`, `view_cache_reuse_roots(...)`, and
    `view_cache_elements_for_root(...)`,
  - keeps diagnostics counters semantically stable by deriving them from the consolidated store,
  - adds a focused runtime test for rendered/next advancement and frame-local flag clearing.

## Correctness Gates

Direct runtime store gate:

```bash
cargo nextest run -p fret-ui \
  elements::runtime::tests::view_cache_build_boundary_store_advances_rendered_next_and_clears_frame_local_flags \
  --no-fail-fast
```

Observed result:

- `1 passed, 933 skipped`.

View-cache behavior gates:

```bash
cargo nextest run -p fret-ui \
  declarative::tests::core::view_cache_subtree_membership_includes_nested_cache_roots \
  declarative::tests::view_cache::view_cache_keep_alive_revalidates_recorded_membership_before_touching_stale_detached_elements \
  declarative::tests::view_cache::view_cache_inherits_model_observations_on_cache_hit_layout \
  --no-fail-fast
```

Observed result:

- `3 passed, 931 skipped`.

Compile gate:

```bash
cargo check -p fret-ui --all-targets
cargo check -p fret-ui --features diagnostics --all-targets
```

Observed result:

- both passed.

## Deletion Note

Deleted or retired in this slice:

- `WindowElementState::view_cache_state_keys_rendered`,
- `WindowElementState::view_cache_state_keys_next`,
- `WindowElementState::view_cache_authoring_identities_rendered`,
- `WindowElementState::view_cache_authoring_identities_next`,
- `WindowElementState::view_cache_keys_rendered`,
- `WindowElementState::view_cache_keys_next`,
- `WindowElementState::view_cache_key_mismatch_roots`,
- `WindowElementState::view_cache_elements_rendered`,
- `WindowElementState::view_cache_elements_next`,
- `WindowElementState::view_cache_reuse_roots`,
- `WindowElementState::view_cache_last_reused_frame`,
- `WindowElementState::view_cache_transitioned_reuse_roots`,
- `WindowElementState::view_cache_stack`.

Still intentionally not deleted:

- the public/internal method surface used by `ElementContext` and declarative mount;
- internal `ViewCacheFlags::contained_layout`;
- node-owned paint-cache replay state;
- the existing mount/GC path that revalidates cached subtree membership before touching retained
  members.

The important design boundary is that subtree membership still requires mount-time live-node
revalidation through `touch_view_cache_subtree_elements_if_recorded(...)`; the build-boundary store
only owns the recorded build/cache-root frame data.

## Perf Evidence

This slice is structural ownership work. It does not make a new optimization claim.

The current code-editor closeout perf evidence remains the latest perf evidence:

- `target/fret-diag-code-editor-resize-probes-frame-v2-closeout-final-20260514/check.perf_thresholds.json`
- `target/fret-diag-code-editor-resize-probes-frame-v2-closeout-final-20260514/1778700500609/bundle.schema2.json`

## Remaining Gaps

- `ViewCacheBuildBoundaryStore` is not yet merged into `ViewBoundaryState`; it is a narrower
  build-boundary owner used to remove parallel flat maps before the final boundary model migration.
- Broader paint-cache replay stores are still node/paint-cache owned.
- Internal low-level `contained_layout` flags and diagnostics remain until layout containment is
  fully expressed through the final boundary dependency contract.
- A second non-code-editor proof surface is still required before global closeout.
